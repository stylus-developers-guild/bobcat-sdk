#![crate_type = "proc-macro"]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{Arm, Attribute, Expr, ExprClosure, ItemFn, Path, Stmt, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn bobcat_trace(attr: TokenStream, input: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(Span::call_site(), "`bobcat_trace` takes no arguments")
            .into_compile_error()
            .into();
    }
    let function = parse_macro_input!(input as ItemFn);
    let sdk = match bobcat_sdk_path() {
        Ok(path) => path,
        Err(error) => return error.into_compile_error().into(),
    };
    expand(function, sdk)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(mut function: ItemFn, sdk: Path) -> syn::Result<TokenStream2> {
    if let Some(constness) = function.sig.constness {
        return Err(syn::Error::new_spanned(
            constness,
            "`bobcat_trace` does not support `const fn`",
        ));
    }
    StatementTracer { sdk }.visit_block_mut(&mut function.block);
    Ok(quote!(#function))
}

struct StatementTracer {
    sdk: Path,
}

impl StatementTracer {
    fn trace_statement(&self, statement: &Stmt) -> Option<Stmt> {
        if matches!(statement, Stmt::Item(_)) {
            return None;
        }
        Some(self.trace_for_tokens(statement.span(), statement, conditional_attrs(statement)))
    }

    fn trace_for_tokens(
        &self,
        span: Span,
        tokens: &impl ToTokens,
        conditional_attrs: Vec<Attribute>,
    ) -> Stmt {
        let source = span
            .source_text()
            .unwrap_or_else(|| tokens.to_token_stream().to_string());
        let sdk = &self.sdk;

        parse_quote! {
            #(#conditional_attrs)*
            #sdk::__bobcat_trace_statement!(#source);
        }
    }

    fn trace_expression(&self, expression: &Expr) -> Stmt {
        self.trace_for_tokens(expression.span(), expression, Vec::new())
    }
}

fn conditional_attrs(statement: &Stmt) -> Vec<Attribute> {
    let attrs = match statement {
        Stmt::Local(local) => local.attrs.clone(),
        Stmt::Item(_) => return Vec::new(),
        Stmt::Expr(expression, _) => {
            let parser = |input: syn::parse::ParseStream<'_>| {
                let attrs = Attribute::parse_outer(input)?;
                let _rest: TokenStream2 = input.parse()?;
                Ok(attrs)
            };
            parser
                .parse2(expression.to_token_stream())
                .unwrap_or_default()
        }
        Stmt::Macro(statement_macro) => statement_macro.attrs.clone(),
    };

    attrs
        .into_iter()
        .filter(|attr| attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr"))
        .collect()
}

impl VisitMut for StatementTracer {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let mut traced = Vec::with_capacity(block.stmts.len() * 2);

        for mut statement in core::mem::take(&mut block.stmts) {
            let trace = self.trace_statement(&statement);
            if !matches!(statement, Stmt::Item(_)) {
                visit_mut::visit_stmt_mut(self, &mut statement);
            }
            if let Some(trace) = trace {
                traced.push(trace);
            }
            traced.push(statement);
        }
        block.stmts = traced;
    }

    fn visit_arm_mut(&mut self, arm: &mut Arm) {
        visit_mut::visit_arm_mut(self, arm);

        if !matches!(&*arm.body, Expr::Block(_)) {
            let trace = self.trace_expression(&arm.body);
            let body = &arm.body;
            arm.body = Box::new(parse_quote!({ #trace #body }));
        }
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut ExprClosure) {
        visit_mut::visit_expr_closure_mut(self, closure);
        if !matches!(&*closure.body, Expr::Block(_)) {
            let trace = self.trace_expression(&closure.body);
            let body = &closure.body;
            closure.body = Box::new(parse_quote!({ #trace #body }));
        }
    }
}

fn bobcat_sdk_path() -> syn::Result<Path> {
    match crate_name("bobcat-sdk") {
        Ok(FoundCrate::Itself) => Ok(parse_quote!(crate)),
        Ok(FoundCrate::Name(name)) => {
            let ident = format_ident!("{}", name);
            Ok(parse_quote!(::#ident))
        }
        Err(error) => Err(syn::Error::new(
            Span::call_site(),
            format!("`bobcat_trace` could not find the `bobcat-sdk` dependency: {error}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn expanded(function: ItemFn) -> String {
        expand(function, parse_quote!(::bobcat_sdk))
            .unwrap()
            .to_string()
    }

    #[test]
    fn traces_each_executable_statement_in_order() {
        let output = expanded(parse_quote! {
            fn example(x: u64) -> u64 {
                let y = x + 1;
                consume(y);
                y
            }
        });

        assert!(output.contains("\"let y = x + 1 ;\""));
        assert!(output.contains("\"consume (y) ;\""));
        assert!(output.contains("\"y\""));
        assert_eq!(output.matches("__bobcat_trace_statement !").count(), 3);
    }

    #[test]
    fn traces_statements_inside_nested_blocks() {
        let output = expanded(parse_quote! {
            fn example(flag: bool) {
                if flag {
                    work();
                } else {
                    fallback();
                }
            }
        });

        assert!(output.contains("\"work () ;\""));
        assert!(output.contains("\"fallback () ;\""));
        assert_eq!(output.matches("__bobcat_trace_statement !").count(), 3);
    }

    #[test]
    fn traces_bare_match_arms_and_closure_bodies() {
        let output = expanded(parse_quote! {
            fn example(value: Option<u64>) {
                let f = |x| x + 1;
                match value {
                    Some(x) => consume(f(x)),
                    None => fallback(),
                }
            }
        });

        assert!(output.contains("\"x + 1\""));
        assert!(output.contains("\"consume (f (x))\""));
        assert!(output.contains("\"fallback ()\""));
    }

    #[test]
    fn does_not_trace_nested_item_declarations() {
        let output = expanded(parse_quote! {
            fn example() {
                fn helper() {
                    nested_work();
                }
                helper();
            }
        });

        assert_eq!(output.matches("__bobcat_trace_statement !").count(), 1);
        assert!(!output.contains("\"fn helper"));
        assert!(!output.contains("\"nested_work () ;\""));
    }

    #[test]
    fn trace_inherits_statement_cfg_attributes() {
        let output = expanded(parse_quote! {
            fn selected() -> u32 {
                #[cfg(feature = "x")]
                { 1 }
                #[cfg(not(feature = "x"))]
                { 2 }
            }
        });

        assert_eq!(output.matches("cfg (feature = \"x\")").count(), 2);
        assert_eq!(output.matches("cfg (not (feature = \"x\"))").count(), 2);
    }

    #[test]
    fn generated_traces_delegate_feature_selection_to_the_sdk() {
        let output = expanded(parse_quote! {
            fn example() {
                work();
            }
        });

        assert!(output.contains("bobcat_sdk :: __bobcat_trace_statement !"));
        assert!(!output.contains("cfg (feature = \"console\")"));
    }

    #[test]
    fn rejects_const_functions() {
        let result = expand(
            parse_quote! {
                const fn example() -> u64 {
                    1
                }
            },
            parse_quote!(::bobcat_sdk),
        );

        assert_eq!(
            result.unwrap_err().to_string(),
            "`bobcat_trace` does not support `const fn`"
        );
    }
}
