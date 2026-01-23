//! Proc macro helpers for working with `bobcat-maths` using Python-style integer snippets.
//! Accepts small arithmetic strings and turns them into `bobcat_maths::U` constants.

use bobcat_maths::U;
use proc_macro::TokenStream;
use quote::quote;
use std::str::FromStr;
use syn::{LitStr, parse_macro_input};

/// Evaluate a Python-style integer expression at compile time.
///
/// Supported:
/// - Decimal literals
/// - Underscores in literals (`1_000_000`)
/// - Operators: +, -, *, //, %, **, <<, >>, &, |, ^ (bitwise xor)
/// - Unary operators and parentheses
///
/// Notes:
/// - `/` is not supported (use `//`).
/// - Expression must evaluate to a non-negative integer fitting in 256 bits.
#[proc_macro]
pub fn maths_zone(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as LitStr);
    match expand(expr) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand(expr: LitStr) -> Result<proc_macro2::TokenStream, syn::Error> {
    let value =
        evaluate_expression(&expr.value()).map_err(|msg| syn::Error::new(expr.span(), msg))?;
    let bytes = value.0;
    let rendered = bytes.iter().map(|b| quote! { #b });
    Ok(quote! {{
        ::bobcat_maths::U([#(#rendered),*])
    }})
}

fn evaluate_expression(src: &str) -> Result<U, String> {
    let mut tokens = lex(src)?;
    tokens.push(Token::Eof);
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.parse_expression()?;
    let trailing = parser.peek();
    if !matches!(trailing, Token::Eof) {
        return Err(format!("unexpected trailing input: {:?}", trailing));
    }
    eval(&expr)
}

fn eval(expr: &Expr) -> Result<U, String> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Ident(_) => Err("identifiers cannot be evaluated at compile time".into()),
        Expr::Unary(UnaryOp::Plus, inner) => eval(inner),
        Expr::Unary(UnaryOp::Minus, _inner) => {
            Err("negative numbers are not supported for U type".into())
        }
        Expr::Binary(lhs, op, rhs) => {
            let l = eval(lhs)?;
            let r = eval(rhs)?;
            match op {
                BinaryOp::Add => bobcat_maths::checked_add_opt(&l, &r)
                    .ok_or_else(|| "addition overflowed 256 bits".into()),
                BinaryOp::Sub => bobcat_maths::checked_sub_opt(&l, &r)
                    .ok_or_else(|| "subtraction underflow: result would be negative".into()),
                BinaryOp::Mul => bobcat_maths::checked_mul_opt(&l, &r)
                    .ok_or_else(|| "multiplication overflowed 256 bits".into()),
                BinaryOp::FloorDiv => {
                    bobcat_maths::checked_div_opt(&l, &r).ok_or_else(|| "division by zero".into())
                }
                BinaryOp::Mod => {
                    if r.is_zero() {
                        Err("modulo by zero".into())
                    } else {
                        Ok(bobcat_maths::modd(&l, &r))
                    }
                }
                BinaryOp::Pow => {
                    if l.is_zero() && r.is_zero() {
                        return Err("0^0 is undefined".into());
                    }
                    let pow_val = to_u32_checked(&r, "exponent")?;
                    checked_pow_u32(&l, pow_val)
                }
                BinaryOp::Shl => {
                    let shift = to_u32_checked(&r, "shift count")?;
                    if shift > 256 {
                        return Err("shift count exceeds 256 bits".into());
                    }
                    Ok(l << shift as usize)
                }
                BinaryOp::Shr => {
                    let shift = to_u32_checked(&r, "shift count")?;
                    if shift > 256 {
                        return Err("shift count exceeds 256 bits".into());
                    }
                    Ok(l >> shift as usize)
                }
                BinaryOp::BitAnd => Ok(l & r),
                BinaryOp::BitOr => Ok(l | r),
                BinaryOp::BitXor => Ok(l ^ r),
            }
        }
    }
}

fn to_u32_checked(value: &U, context: &str) -> Result<u32, String> {
    if value.0[..28].iter().any(|&b| b != 0) {
        return Err(format!("{context} does not fit in a u32"));
    }
    Ok(u32::from_be_bytes([
        value.0[28],
        value.0[29],
        value.0[30],
        value.0[31],
    ]))
}

fn checked_pow_u32(base: &U, exp: u32) -> Result<U, String> {
    if exp == 0 {
        return Ok(U::ONE);
    }

    let mut result = U::ONE;
    let mut power = *base;
    let mut e = exp;

    while e > 0 {
        if e & 1 == 1 {
            result = bobcat_maths::checked_mul_opt(&result, &power)
                .ok_or_else(|| "power operation overflowed 256 bits".to_string())?;
        }
        e >>= 1;
        if e > 0 {
            power = bobcat_maths::checked_mul_opt(&power, &power)
                .ok_or_else(|| "power operation overflowed 256 bits".to_string())?;
        }
    }

    Ok(result)
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(U),
    Ident(String),
    Plus,
    Minus,
    Star,
    DoubleStar,
    DoubleSlash,
    Percent,
    LParen,
    RParen,
    LtLt,
    GtGt,
    Amp,
    Pipe,
    Caret,
    Eof,
}

fn lex(src: &str) -> Result<Vec<Token>, String> {
    let mut chars = src.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '0'..='9' => tokens.push(read_number(&mut chars)?),
            'a'..='z' | 'A'..='Z' | '_' => tokens.push(read_ident(&mut chars)?),
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                chars.next();
                if matches!(chars.peek(), Some('*')) {
                    chars.next();
                    tokens.push(Token::DoubleStar);
                } else {
                    tokens.push(Token::Star);
                }
            }
            '/' => {
                chars.next();
                if matches!(chars.peek(), Some('/')) {
                    chars.next();
                    tokens.push(Token::DoubleSlash);
                } else {
                    return Err("`/` is not supported; use `//` for integer floor-division".into());
                }
            }
            '%' => {
                chars.next();
                tokens.push(Token::Percent);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            '<' => {
                chars.next();
                if matches!(chars.peek(), Some('<')) {
                    chars.next();
                    tokens.push(Token::LtLt);
                } else {
                    return Err("expected '<' after '<' for a shift expression".into());
                }
            }
            '>' => {
                chars.next();
                if matches!(chars.peek(), Some('>')) {
                    chars.next();
                    tokens.push(Token::GtGt);
                } else {
                    return Err("expected '>' after '>' for a shift expression".into());
                }
            }
            '&' => {
                chars.next();
                tokens.push(Token::Amp);
            }
            '|' => {
                chars.next();
                tokens.push(Token::Pipe);
            }
            '^' => {
                chars.next();
                tokens.push(Token::Caret);
            }
            _ => {
                return Err(format!("unsupported character '{c}'"));
            }
        }
    }
    Ok(tokens)
}

fn read_number<I>(chars: &mut core::iter::Peekable<I>) -> Result<Token, String>
where
    I: Iterator<Item = char>,
{
    let mut buf = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || c == '_' {
            buf.push(c);
            chars.next();
        } else {
            break;
        }
    }

    let parsed = parse_number_literal(&buf)?;
    Ok(Token::Number(parsed))
}

fn read_ident<I>(chars: &mut core::iter::Peekable<I>) -> Result<Token, String>
where
    I: Iterator<Item = char>,
{
    let mut buf = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphanumeric() || c == '_' {
            buf.push(c);
            chars.next();
        } else {
            break;
        }
    }
    Ok(Token::Ident(buf))
}

fn parse_number_literal(s: &str) -> Result<U, String> {
    let cleaned: String = s.chars().filter(|c| *c != '_').collect();
    if cleaned.is_empty() {
        return Err("empty number literal".into());
    }
    // Parse decimal using from_str
    U::from_str(cleaned.as_str()).map_err(|e| format!("invalid decimal literal '{}': {:?}", s, e))
}

#[derive(Debug, Clone)]
enum Expr {
    Number(U),
    Ident(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
}

#[derive(Debug, Copy, Clone)]
enum UnaryOp {
    Plus,
    Minus,
}

#[derive(Debug, Copy, Clone)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    FloorDiv,
    Mod,
    Pow,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_bit_or()
    }

    fn parse_bit_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_bit_xor()?;
        while matches!(self.peek(), Token::Pipe) {
            self.next();
            let rhs = self.parse_bit_xor()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::BitOr, Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_bit_xor(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_bit_and()?;
        while matches!(self.peek(), Token::Caret) {
            self.next();
            let rhs = self.parse_bit_and()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::BitXor, Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_bit_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_shift()?;
        while matches!(self.peek(), Token::Amp) {
            self.next();
            let rhs = self.parse_shift()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::BitAnd, Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_shift(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_add_sub()?;
        loop {
            match self.peek() {
                Token::LtLt => {
                    self.next();
                    let rhs = self.parse_add_sub()?;
                    expr = Expr::Binary(Box::new(expr), BinaryOp::Shl, Box::new(rhs));
                }
                Token::GtGt => {
                    self.next();
                    let rhs = self.parse_add_sub()?;
                    expr = Expr::Binary(Box::new(expr), BinaryOp::Shr, Box::new(rhs));
                }
                _ => return Ok(expr),
            }
        }
    }

    fn parse_add_sub(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_mul_div()?;
        loop {
            match self.peek() {
                Token::Plus => {
                    self.next();
                    let rhs = self.parse_mul_div()?;
                    expr = Expr::Binary(Box::new(expr), BinaryOp::Add, Box::new(rhs));
                }
                Token::Minus => {
                    self.next();
                    let rhs = self.parse_mul_div()?;
                    expr = Expr::Binary(Box::new(expr), BinaryOp::Sub, Box::new(rhs));
                }
                _ => return Ok(expr),
            }
        }
    }

    fn parse_mul_div(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_power()?;
        loop {
            match self.peek() {
                Token::Star => {
                    self.next();
                    let rhs = self.parse_power()?;
                    expr = Expr::Binary(Box::new(expr), BinaryOp::Mul, Box::new(rhs));
                }
                Token::DoubleSlash => {
                    self.next();
                    let rhs = self.parse_power()?;
                    expr = Expr::Binary(Box::new(expr), BinaryOp::FloorDiv, Box::new(rhs));
                }
                Token::Percent => {
                    self.next();
                    let rhs = self.parse_power()?;
                    expr = Expr::Binary(Box::new(expr), BinaryOp::Mod, Box::new(rhs));
                }
                _ => return Ok(expr),
            }
        }
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_unary()?;
        if matches!(self.peek(), Token::DoubleStar) {
            self.next();
            let rhs = self.parse_power()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::Pow, Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Token::Plus => {
                self.next();
                Ok(Expr::Unary(UnaryOp::Plus, Box::new(self.parse_unary()?)))
            }
            Token::Minus => {
                self.next();
                Ok(Expr::Unary(UnaryOp::Minus, Box::new(self.parse_unary()?)))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.next() {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::Ident(name) => Ok(Expr::Ident(name)),
            Token::LParen => {
                let expr = self.parse_expression()?;
                match self.next() {
                    Token::RParen => Ok(expr),
                    _ => Err("expected ')'".into()),
                }
            }
            Token::Eof => Err("unexpected end of expression".into()),
            other => Err(format!(
                "unexpected token in primary expression: {:?}",
                other
            )),
        }
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof)
    }

    fn next(&mut self) -> Token {
        let tok = self.peek();
        self.pos += 1;
        tok
    }
}

/// Generate checked math operations from Python-like code provided as a string literal.
///
/// Syntax:
/// ```rust,ignore
/// bobcat_math!(r#"
///     [label = "optional label", unwrap]  // unwrap is optional
///     a = 123 + 12831 * x ** 12
///     b = a // 2
///     return b
/// "#);
/// ```
///
/// Supported operators: `+`, `-`, `*`, `//` (floor div), `%`, `**` (power), `<<`, `>>`, `&`, `|`, `^`
/// With `unwrap`: uses `.expect()` for errors
/// Without `unwrap`: returns `Option<U>`
#[proc_macro]
pub fn bobcat_math(input: TokenStream) -> TokenStream {
    let source = parse_macro_input!(input as LitStr);
    match parse_bobcat_math(&source) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[derive(Debug)]
struct MathBlock {
    label: Option<String>,
    unwrap: bool,
    statements: Vec<Statement>,
}

#[derive(Debug)]
enum Statement {
    Assign { var: String, expr: Expr },
    Return(Expr),
}

fn parse_bobcat_math(src: &LitStr) -> Result<proc_macro2::TokenStream, syn::Error> {
    let block = parse_math_block(src)?;
    generate_code(&block, src.span())
}

fn parse_math_block(src: &LitStr) -> Result<MathBlock, syn::Error> {
    let span = src.span();
    let mut label = None;
    let mut unwrap = false;
    let mut statements = Vec::new();
    let mut saw_return = false;

    for raw_line in src.value().lines() {
        let mut line = raw_line.trim();
        if let Some(stripped) = line.strip_suffix(';') {
            line = stripped.trim_end();
        }
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if statements.is_empty() && label.is_none() && !unwrap && line.starts_with('[') {
            let attrs = parse_attributes_line(line, span)?;
            label = attrs.label;
            unwrap = attrs.unwrap;
            continue;
        }

        if saw_return {
            return Err(syn::Error::new(
                span,
                "return must be the final statement in bobcat_math!",
            ));
        }

        if line.starts_with("return") {
            let expr_str = line["return".len()..].trim();
            let expr = parse_expression_string(expr_str, span)?;
            statements.push(Statement::Return(expr));
            saw_return = true;
        } else if let Some(eq_pos) = line.find('=') {
            let var = line[..eq_pos].trim();
            if var.is_empty() {
                return Err(syn::Error::new(span, "missing variable name before '='"));
            }
            let ident: syn::Ident = syn::parse_str(var).map_err(|_| {
                syn::Error::new(span, format!("invalid identifier `{var}` in assignment"))
            })?;
            let expr_str = line[eq_pos + 1..].trim();
            let expr = parse_expression_string(expr_str, span)?;
            statements.push(Statement::Assign {
                var: ident.to_string(),
                expr,
            });
        } else {
            return Err(syn::Error::new(
                span,
                format!("could not parse statement `{line}`"),
            ));
        }
    }

    if !saw_return {
        return Err(syn::Error::new(
            span,
            "bobcat_math! requires a `return` statement",
        ));
    }

    Ok(MathBlock {
        label,
        unwrap,
        statements,
    })
}

fn parse_attributes_line(
    line: &str,
    span: proc_macro2::Span,
) -> Result<BlockAttributes, syn::Error> {
    if !line.ends_with(']') {
        return Err(syn::Error::new(
            span,
            "attribute line must be closed with ']'",
        ));
    }

    let inner = &line[1..line.len() - 1];
    let mut label = None;
    let mut unwrap = false;

    for part in inner.split(',').map(|p| p.trim()).filter(|p| !p.is_empty()) {
        if part == "unwrap" {
            unwrap = true;
            continue;
        }

        if let Some(rest) = part.strip_prefix("label") {
            let rest = rest.trim();
            let rest = rest
                .strip_prefix('=')
                .ok_or_else(|| syn::Error::new(span, "expected `=` after label attribute name"))?;
            let lit: LitStr = syn::parse_str(rest.trim()).map_err(|_| {
                syn::Error::new(span, "label must be a string literal, e.g. \"my label\"")
            })?;
            if label.replace(lit.value()).is_some() {
                return Err(syn::Error::new(
                    span,
                    "label attribute specified more than once",
                ));
            }
            continue;
        }

        return Err(syn::Error::new(
            span,
            format!("unknown attribute `{part}` in bobcat_math! header"),
        ));
    }

    Ok(BlockAttributes { label, unwrap })
}

struct BlockAttributes {
    label: Option<String>,
    unwrap: bool,
}

fn parse_expression_string(s: &str, span: proc_macro2::Span) -> Result<Expr, syn::Error> {
    let mut tokens = lex(s).map_err(|e| syn::Error::new(span, e))?;
    tokens.push(Token::Eof);
    let mut parser = Parser { tokens, pos: 0 };
    parser
        .parse_expression()
        .map_err(|e| syn::Error::new(span, e))
}

fn generate_code(
    block: &MathBlock,
    span: proc_macro2::Span,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut stmts = proc_macro2::TokenStream::new();
    let mut var_map = std::collections::HashMap::new();
    let mut return_expr = None;

    for (idx, stmt) in block.statements.iter().enumerate() {
        match stmt {
            Statement::Assign { var, expr } => {
                let var_ident = syn::Ident::new(var, proc_macro2::Span::call_site());
                let expr_code =
                    generate_expr_code(expr, &var_map, block, &format!("{}.{}", var, idx))?;

                stmts.extend(quote! {
                    let #var_ident = #expr_code;
                });

                var_map.insert(var.clone(), var_ident);
            }
            Statement::Return(expr) => {
                let expr_code =
                    generate_expr_code(expr, &var_map, block, &format!("return.{idx}"))?;
                return_expr = Some(expr_code);
            }
        }
    }

    let Some(result) = return_expr else {
        return Err(syn::Error::new(
            span,
            "bobcat_math! requires a `return` statement",
        ));
    };

    let body = if block.unwrap {
        quote! {
            #stmts
            #result
        }
    } else {
        quote! {
            #stmts
            Some(#result)
        }
    };

    if block.unwrap {
        Ok(quote! {{ #body }})
    } else {
        Ok(quote! {{
            (|| -> ::core::option::Option<::bobcat_maths::U> {
                #body
            })()
        }})
    }
}

fn generate_expr_code(
    expr: &Expr,
    var_map: &std::collections::HashMap<String, syn::Ident>,
    block: &MathBlock,
    path: &str,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    match expr {
        Expr::Number(n) => {
            let bytes = n.0;
            let rendered = bytes.iter().map(|b| quote! { #b });
            Ok(quote! {
                ::bobcat_maths::U([#(#rendered),*])
            })
        }
        Expr::Ident(name) => {
            // Check if it's a variable we've defined
            if let Some(var_ident) = var_map.get(name) {
                Ok(quote! { #var_ident })
            } else {
                // It's an external variable - lift it with U::from()
                let ext_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                Ok(quote! {
                    ::bobcat_maths::U::from(#ext_ident)
                })
            }
        }
        Expr::Unary(op, inner) => {
            let inner_code = generate_expr_code(inner, var_map, block, &format!("{}.unary", path))?;
            match op {
                UnaryOp::Plus => Ok(inner_code),
                UnaryOp::Minus => Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "unary minus not supported for U type",
                )),
            }
        }
        Expr::Binary(lhs, op, rhs) => {
            let lhs_code = generate_expr_code(lhs, var_map, block, &format!("{}.lhs", path))?;
            let rhs_code = generate_expr_code(rhs, var_map, block, &format!("{}.rhs", path))?;

            let error_msg = if let Some(ref lbl) = block.label {
                format!("{lbl} overflowed at {path}")
            } else {
                format!("overflow or invalid operation at {path}")
            };

            match op {
                BinaryOp::Add => {
                    if block.unwrap {
                        Ok(quote! {
                            ::bobcat_panic::panic_on_err_overflow!(
                                (#lhs_code).checked_add_opt(&#rhs_code),
                                #error_msg
                            )
                        })
                    } else {
                        Ok(quote! {
                            (#lhs_code).checked_add_opt(&#rhs_code)?
                        })
                    }
                }
                BinaryOp::Sub => {
                    if block.unwrap {
                        Ok(quote! {
                            ::bobcat_panic::panic_on_err_overflow!(
                                (#lhs_code).checked_sub_opt(&#rhs_code),
                                #error_msg
                            )
                        })
                    } else {
                        Ok(quote! {
                            (#lhs_code).checked_sub_opt(&#rhs_code)?
                        })
                    }
                }
                BinaryOp::Mul => {
                    if block.unwrap {
                        Ok(quote! {
                            ::bobcat_panic::panic_on_err_overflow!(
                                (#lhs_code).checked_mul_opt(&#rhs_code),
                                #error_msg
                            )
                        })
                    } else {
                        Ok(quote! {
                            (#lhs_code).checked_mul_opt(&#rhs_code)?
                        })
                    }
                }
                BinaryOp::FloorDiv => {
                    if block.unwrap {
                        Ok(quote! {
                            ::bobcat_panic::panic_on_err_div_by_zero!(
                                (#lhs_code).checked_div_opt(&#rhs_code),
                                #error_msg
                            )
                        })
                    } else {
                        Ok(quote! {
                            (#lhs_code).checked_div_opt(&#rhs_code)?
                        })
                    }
                }
                BinaryOp::Mod => {
                    if block.unwrap {
                        Ok(quote! {{
                            let lhs_val = #lhs_code;
                            let rhs_val = #rhs_code;
                            ::bobcat_panic::panic_on_err_div_by_zero!(
                                if rhs_val.is_zero() { None } else { Some(()) },
                                #error_msg
                            );
                            ::bobcat_maths::modd(&lhs_val, &rhs_val)
                        }})
                    } else {
                        Ok(quote! {{
                            let lhs_val = #lhs_code;
                            let rhs_val = #rhs_code;
                            if rhs_val.is_zero() {
                                return None;
                            }
                            ::bobcat_maths::modd(&lhs_val, &rhs_val)
                        }})
                    }
                }
                BinaryOp::Pow => {
                    if block.unwrap {
                        Ok(quote! {{
                            let lhs_val = #lhs_code;
                            let rhs_val = #rhs_code;
                            if lhs_val.is_zero() && rhs_val.is_zero() {
                                ::bobcat_panic::panic_on_err_overflow!(None, #error_msg);
                            }
                            ::bobcat_panic::panic_on_err_overflow!(
                                lhs_val.checked_pow(&rhs_val),
                                #error_msg
                            )
                        }})
                    } else {
                        Ok(quote! {{
                            let lhs_val = #lhs_code;
                            let rhs_val = #rhs_code;
                            if lhs_val.is_zero() && rhs_val.is_zero() {
                                return None;
                            }
                            lhs_val.checked_pow(&rhs_val)?
                        }})
                    }
                }
                BinaryOp::Shl => {
                    let shift_tokens = if block.unwrap {
                        quote! {{
                            let rhs_val = #rhs_code;
                            let rhs_bytes: [u8; 32] = rhs_val.into();
                            let shift = if rhs_bytes[..28].iter().any(|&b| b != 0) {
                                None
                            } else {
                                let shift = u32::from_be_bytes([rhs_bytes[28], rhs_bytes[29], rhs_bytes[30], rhs_bytes[31]]);
                                if shift > 256 {
                                    None
                                } else {
                                    Some(shift as usize)
                                }
                            };
                            ::bobcat_panic::panic_on_err_overflow!(shift, #error_msg)
                        }}
                    } else {
                        quote! {{
                            let rhs_val = #rhs_code;
                            let rhs_bytes: [u8; 32] = rhs_val.into();
                            if rhs_bytes[..28].iter().any(|&b| b != 0) {
                                return None;
                            }
                            let shift = u32::from_be_bytes([rhs_bytes[28], rhs_bytes[29], rhs_bytes[30], rhs_bytes[31]]);
                            if shift > 256 {
                                return None;
                            }
                            shift as usize
                        }}
                    };

                    Ok(quote! {{
                        let lhs_val = #lhs_code;
                        let shift = #shift_tokens;
                        lhs_val << shift
                    }})
                }
                BinaryOp::Shr => {
                    let shift_tokens = if block.unwrap {
                        quote! {{
                            let rhs_val = #rhs_code;
                            let rhs_bytes: [u8; 32] = rhs_val.into();
                            let shift = if rhs_bytes[..28].iter().any(|&b| b != 0) {
                                None
                            } else {
                                let shift = u32::from_be_bytes([rhs_bytes[28], rhs_bytes[29], rhs_bytes[30], rhs_bytes[31]]);
                                if shift > 256 {
                                    None
                                } else {
                                    Some(shift as usize)
                                }
                            };
                            ::bobcat_panic::panic_on_err_overflow!(shift, #error_msg)
                        }}
                    } else {
                        quote! {{
                            let rhs_val = #rhs_code;
                            let rhs_bytes: [u8; 32] = rhs_val.into();
                            if rhs_bytes[..28].iter().any(|&b| b != 0) {
                                return None;
                            }
                            let shift = u32::from_be_bytes([rhs_bytes[28], rhs_bytes[29], rhs_bytes[30], rhs_bytes[31]]);
                            if shift > 256 {
                                return None;
                            }
                            shift as usize
                        }}
                    };

                    Ok(quote! {{
                        let lhs_val = #lhs_code;
                        let shift = #shift_tokens;
                        lhs_val >> shift
                    }})
                }
                BinaryOp::BitAnd => Ok(quote! { (#lhs_code & #rhs_code) }),
                BinaryOp::BitOr => Ok(quote! { (#lhs_code | #rhs_code) }),
                BinaryOp::BitXor => Ok(quote! { (#lhs_code ^ #rhs_code) }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_bytes(expr: &str) -> [u8; 32] {
        let value = evaluate_expression(expr).expect("expression should parse");
        value.0
    }

    #[test]
    fn parses_simple_expression() {
        assert_eq!(
            eval_bytes("2**8 + 5"),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 1, 5
            ]
        );
    }

    #[test]
    fn handles_floor_div_and_mod() {
        assert_eq!(
            eval_bytes("((5 // 2) << 4) | (11 % 3)"),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 34
            ]
        );
    }

    #[test]
    fn rejects_negative_results() {
        let err = evaluate_expression("-1").unwrap_err();
        assert!(err.contains("negative"), "got: {}", err);
    }

    #[test]
    fn rejects_single_slash() {
        let err = evaluate_expression("1/2").unwrap_err();
        assert!(
            err.contains("not supported") && err.contains("//"),
            "got: {}",
            err
        );
    }

    #[test]
    fn parses_underscores() {
        assert_eq!(eval_bytes("1_000_000"), eval_bytes("1000000"));
        assert_eq!(eval_bytes("255 + 2"), eval_bytes("257"));
    }

    #[test]
    fn caret_is_bitwise_xor() {
        assert_eq!(eval_bytes("10 ^ 3"), eval_bytes("9"));
    }

    #[test]
    fn rejects_large_shifts() {
        let err = evaluate_expression("1 << 300").unwrap_err();
        assert!(
            err.contains("shift count"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn rejects_overflowing_pow() {
        let err = evaluate_expression("2 ** 256").unwrap_err();
        assert!(err.contains("overflow"), "unexpected error message: {err}");
    }

    #[test]
    fn parses_header_attributes_and_unwrap() {
        let lit = LitStr::new(
            "[label = \"demo\", unwrap]\na = 1\nreturn a",
            proc_macro2::Span::call_site(),
        );
        let block = parse_math_block(&lit).expect("should parse");
        assert_eq!(block.label.as_deref(), Some("demo"));
        assert!(block.unwrap);
    }

    #[test]
    fn bobcat_math_wraps_option_when_not_unwrapped() {
        let lit = LitStr::new("a = 1 + 2\nreturn a", proc_macro2::Span::call_site());
        let tokens = parse_bobcat_math(&lit).expect("should parse");
        let rendered = tokens.to_string();
        assert!(
            rendered.contains("Option"),
            "rendered tokens missing Option: {rendered}"
        );
        assert!(
            rendered.contains("Some"),
            "rendered tokens missing Some(): {rendered}"
        );
    }

    #[test]
    fn return_must_be_last_statement() {
        let lit = LitStr::new("return 1\na = 2", proc_macro2::Span::call_site());
        let err = parse_math_block(&lit).unwrap_err();
        assert!(
            err.to_string()
                .contains("return must be the final statement"),
            "unexpected error: {err}"
        );
    }
}
