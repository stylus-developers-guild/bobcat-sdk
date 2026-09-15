#![crate_type = "proc-macro"]

extern crate proc_macro;

use heck::ToLowerCamelCase;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields, Generics, Type, parse_macro_input};

#[proc_macro_derive(EvmCdSerialise, attributes(evm_values, evm_entrypoint))]
pub fn derive_evm_cd_serialise(input: TokenStream) -> TokenStream {
    expand(
        parse_macro_input!(input as DeriveInput),
        Direction::Serialise,
    )
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

#[proc_macro_derive(EvmCdDeserialise, attributes(evm_values, evm_entrypoint))]
pub fn derive_evm_cd_deserialise(input: TokenStream) -> TokenStream {
    expand(
        parse_macro_input!(input as DeriveInput),
        Direction::Deserialise,
    )
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

#[derive(Clone, Copy)]
enum Direction {
    Serialise,
    Deserialise,
}

fn expand(input: DeriveInput, direction: Direction) -> syn::Result<TokenStream2> {
    let cd = bobcat_cd_path()?;
    let name = &input.ident;
    let evm_values = has_evm_values(&input)?;
    let evm_entrypoint = has_evm_entrypoint(&input)?;
    if evm_values && evm_entrypoint {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`evm_values` and `evm_entrypoint` cannot be used together",
        ));
    }
    let io_ident = fresh_type_ident(
        &input.generics,
        match direction {
            Direction::Serialise => "__EvmCdWriter",
            Direction::Deserialise => "__EvmCdReader",
        },
    );
    let slice_lifetime = fresh_lifetime_ident(&input.generics, "__evm_cd_slice");
    let field_types = all_field_types(&input.data);
    let trait_path = trait_path(direction, &cd);
    let generics = add_field_bounds(input.generics.clone(), &field_types, &trait_path);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = match (&input.data, direction) {
        (Data::Struct(data), Direction::Serialise) => serialise_struct(data, &cd),
        (Data::Struct(data), Direction::Deserialise) => deserialise_struct(data, &cd),
        (Data::Enum(data), Direction::Serialise) if evm_entrypoint => {
            serialise_enum(name, data, &cd)?
        }
        (Data::Enum(data), Direction::Deserialise) if evm_entrypoint => {
            deserialise_enum(name, data, &cd)?
        }
        (Data::Enum(data), Direction::Serialise) => serialise_enum_value(name, data, &cd, true)?,
        (Data::Enum(data), Direction::Deserialise) => {
            deserialise_enum_value(name, data, &cd, true)?
        }
        (Data::Union(data), _) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "EVM calldata derives do not support unions",
            ));
        }
    };
    let abi_type = abi_type_body(&input.data, &trait_path);
    let abi_methods = abi_methods(&input.data, direction, &trait_path, &cd, &io_ident);
    let value_method = match (&input.data, direction) {
        (Data::Enum(data), Direction::Serialise) => {
            let body = serialise_enum_value(name, data, &cd, !evm_entrypoint)?;
            quote! {
                fn serialise_value<#io_ident: #cd::serialisation::Write>(
                    &self,
                    writer: &mut #io_ident,
                ) -> ::core::result::Result<(), #cd::serialisation::Error> {
                    #body
                }
            }
        }
        (Data::Enum(data), Direction::Deserialise) => {
            let body = deserialise_enum_value(name, data, &cd, !evm_entrypoint)?;
            quote! {
                fn deserialise_value<#io_ident: #cd::serialisation::Read>(
                    reader: &mut #io_ident,
                ) -> ::core::result::Result<Self, #cd::serialisation::Error> {
                    #body
                }
            }
        }
        _ => TokenStream2::new(),
    };

    let buffer_type = match direction {
        Direction::Serialise => TokenStream2::new(),
        Direction::Deserialise => {
            deserialise_buffer_type(&input.data, evm_entrypoint, &trait_path, &cd)
        }
    };
    let to_evm_array_method = match (&input.data, direction) {
        (Data::Struct(data), Direction::Serialise) => static_struct_serialised_size(data)
            .map(|size| {
                quote! {
                    pub fn to_evm_array(
                        &self,
                    ) -> ::core::result::Result<[u8; #size], #cd::serialisation::Error> {
                        let mut output = [0u8; #size];
                        let mut writer = output.as_mut_slice();
                        <Self as #cd::serialisation::EvmCdSerialise>::serialise_writer(
                            self,
                            &mut writer,
                        )?;
                        debug_assert!(writer.is_empty());
                        ::core::result::Result::Ok(output)
                    }
                }
            })
            .unwrap_or_default(),
        _ => TokenStream2::new(),
    };

    let output = match direction {
        Direction::Serialise => quote! {
            #[automatically_derived]
            impl #impl_generics #cd::serialisation::EvmCdSerialise for #name #ty_generics #where_clause {
                fn serialise_writer<#io_ident: #cd::serialisation::Write>(
                    &self,
                    writer: &mut #io_ident,
                ) -> ::core::result::Result<(), #cd::serialisation::Error> {
                    #body
                }

                #value_method

                #abi_methods

                fn append_abi_type(
                    hasher: #cd::serialisation::SelectorHasher,
                ) -> #cd::serialisation::SelectorHasher {
                    #abi_type
                }
            }

            #[automatically_derived]
            impl #impl_generics #name #ty_generics #where_clause {
                /// Serialises into `output` and returns its written prefix.
                pub fn write_slice<#slice_lifetime>(
                    &self,
                    output: &#slice_lifetime mut [u8],
                ) -> ::core::result::Result<&#slice_lifetime mut [u8], #cd::serialisation::Error> {
                    let output_len = output.len();
                    let mut writer = &mut *output;
                    <Self as #cd::serialisation::EvmCdSerialise>::serialise_writer(
                        self,
                        &mut writer,
                    )?;
                    let written = output_len - writer.len();
                    ::core::result::Result::Ok(&mut output[..written])
                }

                #to_evm_array_method
            }
        },
        Direction::Deserialise => quote! {
            #[automatically_derived]
            impl #impl_generics #cd::serialisation::EvmCdDeserialise for #name #ty_generics #where_clause {
                #buffer_type

                fn deserialise_reader<#io_ident: #cd::serialisation::Read>(
                    reader: &mut #io_ident,
                ) -> ::core::result::Result<Self, #cd::serialisation::Error> {
                    #body
                }

                #value_method

                #abi_methods

                fn append_abi_type(
                    hasher: #cd::serialisation::SelectorHasher,
                ) -> #cd::serialisation::SelectorHasher {
                    #abi_type
                }
            }
        },
    };
    Ok(output)
}

fn has_evm_values(input: &DeriveInput) -> syn::Result<bool> {
    let mut attributes = input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("evm_values"));
    let Some(attribute) = attributes.next() else {
        return Ok(false);
    };
    if !matches!(attribute.meta, syn::Meta::Path(_)) {
        return Err(syn::Error::new_spanned(
            attribute,
            "`evm_values` does not accept arguments",
        ));
    }
    if let Some(duplicate) = attributes.next() {
        return Err(syn::Error::new_spanned(
            duplicate,
            "duplicate `evm_values` attribute",
        ));
    }
    if matches!(input.data, Data::Union(_)) {
        return Err(syn::Error::new_spanned(
            attribute,
            "`evm_values` is only supported on structs and enums",
        ));
    }
    Ok(true)
}

fn has_evm_entrypoint(input: &DeriveInput) -> syn::Result<bool> {
    let mut attributes = input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("evm_entrypoint"));
    let Some(attribute) = attributes.next() else {
        return Ok(false);
    };
    if !matches!(attribute.meta, syn::Meta::Path(_)) {
        return Err(syn::Error::new_spanned(
            attribute,
            "`evm_entrypoint` does not accept arguments",
        ));
    }
    if let Some(duplicate) = attributes.next() {
        return Err(syn::Error::new_spanned(
            duplicate,
            "duplicate `evm_entrypoint` attribute",
        ));
    }
    if !matches!(input.data, Data::Enum(_)) {
        return Err(syn::Error::new_spanned(
            attribute,
            "`evm_entrypoint` is only supported on enums",
        ));
    }
    Ok(true)
}

fn fresh_type_ident(generics: &Generics, base: &str) -> syn::Ident {
    let existing: ::std::collections::HashSet<_> = generics
        .params
        .iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(param) => Some(param.ident.to_string()),
            syn::GenericParam::Const(param) => Some(param.ident.to_string()),
            syn::GenericParam::Lifetime(_) => None,
        })
        .collect();
    let mut candidate = base.to_owned();
    let mut suffix = 0usize;
    while existing.contains(&candidate) {
        suffix += 1;
        candidate = format!("{base}{suffix}");
    }
    format_ident!("{candidate}")
}

fn fresh_lifetime_ident(generics: &Generics, base: &str) -> syn::Lifetime {
    let existing: ::std::collections::HashSet<_> = generics
        .lifetimes()
        .map(|param| param.lifetime.ident.to_string())
        .collect();
    let mut candidate = base.to_owned();
    let mut suffix = 0usize;
    while existing.contains(&candidate) {
        suffix += 1;
        candidate = format!("{base}{suffix}");
    }
    syn::Lifetime::new(&format!("'{candidate}"), proc_macro2::Span::call_site())
}

fn trait_path(direction: Direction, cd: &TokenStream2) -> TokenStream2 {
    match direction {
        Direction::Serialise => quote!(#cd::serialisation::EvmCdSerialise),
        Direction::Deserialise => quote!(#cd::serialisation::EvmCdDeserialise),
    }
}

fn bobcat_cd_path() -> syn::Result<TokenStream2> {
    match crate_name("bobcat-cd") {
        Ok(FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(FoundCrate::Name(name)) => {
            let ident = format_ident!("{}", name);
            Ok(quote!(::#ident))
        }
        Err(cd_error) => match crate_name("bobcat-sdk") {
            Ok(FoundCrate::Itself) => Ok(quote!(crate::cd)),
            Ok(FoundCrate::Name(name)) => {
                let ident = format_ident!("{}", name);
                Ok(quote!(::#ident::cd))
            }
            Err(sdk_error) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "could not find bobcat-cd or bobcat-sdk in dependencies: {cd_error}; {sdk_error}"
                ),
            )),
        },
    }
}

fn all_field_types(data: &Data) -> Vec<Type> {
    match data {
        Data::Struct(data) => data.fields.iter().map(|field| field.ty.clone()).collect(),
        Data::Enum(data) => data
            .variants
            .iter()
            .flat_map(|variant| variant.fields.iter().map(|field| field.ty.clone()))
            .collect(),
        Data::Union(_) => Vec::new(),
    }
}

fn add_field_bounds(
    mut generics: Generics,
    field_types: &[Type],
    trait_path: &TokenStream2,
) -> Generics {
    let where_clause = generics.make_where_clause();
    for ty in field_types {
        where_clause
            .predicates
            .push(syn::parse_quote!(#ty: #trait_path));
    }
    generics
}

fn field_accesses(fields: &Fields) -> Vec<syn::Member> {
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            field.ident.as_ref().map_or_else(
                || syn::Member::Unnamed(index.into()),
                |ident| syn::Member::Named(ident.clone()),
            )
        })
        .collect()
}

fn deserialise_buffer_type(
    data: &Data,
    evm_entrypoint: bool,
    trait_path: &TokenStream2,
    cd: &TokenStream2,
) -> TokenStream2 {
    if matches!(data, Data::Enum(_)) && !evm_entrypoint {
        return quote! {
            type Buffer = #cd::serialisation::EvmCdBuffer<[u8; 32]>;
        };
    }

    let mut storage = if evm_entrypoint {
        quote!([u8; 4])
    } else {
        quote!(())
    };
    let mut kind = quote!(#cd::serialisation::EvmCdStaticBufferKind);
    for ty in all_field_types(data).iter().rev() {
        storage = quote!((<#ty as #trait_path>::Buffer, #storage));
        kind = quote! {
            <<<#ty as #trait_path>::Buffer as #cd::serialisation::EvmCdDecodeBuffer>::Kind
                as #cd::serialisation::EvmCdBufferKind>::Combined<#kind>
        };
    }
    quote! {
        type Buffer = <#kind as #cd::serialisation::EvmCdBufferKind>::Buffer<#storage>;
    }
}

fn abi_methods(
    data: &Data,
    direction: Direction,
    trait_path: &TokenStream2,
    cd: &TokenStream2,
    io_ident: &syn::Ident,
) -> TokenStream2 {
    let Data::Struct(data) = data else {
        return TokenStream2::new();
    };
    let types: Vec<_> = data.fields.iter().map(|field| &field.ty).collect();
    let accesses = field_accesses(&data.fields);
    let dynamic = quote!(false #(|| <#types as #trait_path>::is_abi_dynamic())*);
    let static_head_size = quote!(0usize #(+ <#types as #trait_path>::abi_head_size())*);
    let tail_sizes = accesses.iter().zip(types.iter()).map(|(access, ty)| {
        quote! {
            size = size.saturating_add(<#ty as #trait_path>::abi_tail_size(&self.#access));
        }
    });

    match direction {
        Direction::Serialise => {
            let tuple = serialise_tuple(&data.fields, cd);
            quote! {
                fn is_abi_dynamic() -> bool {
                    #dynamic
                }

                fn abi_head_size() -> usize {
                    if <Self as #trait_path>::is_abi_dynamic() { 32 } else { #static_head_size }
                }

                fn abi_tail_size(&self) -> usize {
                    if !<Self as #trait_path>::is_abi_dynamic() {
                        return 0;
                    }
                    let mut size = #static_head_size;
                    #(#tail_sizes)*
                    size
                }

                fn serialise_abi_head<#io_ident: #cd::serialisation::Write>(
                    &self,
                    tail_offset: usize,
                    writer: &mut #io_ident,
                ) -> ::core::result::Result<(), #cd::serialisation::Error> {
                    if <Self as #trait_path>::is_abi_dynamic() {
                        <usize as #trait_path>::serialise_value(&tail_offset, writer)
                    } else {
                        #tuple
                    }
                }

                fn serialise_abi_tail<#io_ident: #cd::serialisation::Write>(
                    &self,
                    writer: &mut #io_ident,
                ) -> ::core::result::Result<(), #cd::serialisation::Error> {
                    if <Self as #trait_path>::is_abi_dynamic() {
                        #tuple
                    } else {
                        ::core::result::Result::Ok(())
                    }
                }
            }
        }
        Direction::Deserialise => {
            let tail_sizes = accesses.iter().zip(types.iter()).map(|(access, ty)| {
                quote! {
                    size = size.saturating_add(<#ty as #trait_path>::abi_tail_size(&self.#access));
                }
            });
            quote! {
                fn is_abi_dynamic() -> bool {
                    #dynamic
                }

                fn abi_head_size() -> usize {
                    if <Self as #trait_path>::is_abi_dynamic() { 32 } else { #static_head_size }
                }

                fn abi_tail_size(&self) -> usize {
                    if !<Self as #trait_path>::is_abi_dynamic() {
                        return 0;
                    }
                    let mut size = #static_head_size;
                    #(#tail_sizes)*
                    size
                }

                fn deserialise_abi_head<#io_ident: #cd::serialisation::Read>(
                    reader: &mut #io_ident,
                ) -> ::core::result::Result<#cd::serialisation::EvmCdHead<Self>, #cd::serialisation::Error> {
                    if <Self as #trait_path>::is_abi_dynamic() {
                        ::core::result::Result::Ok(#cd::serialisation::EvmCdHead::Offset(
                            <usize as #trait_path>::deserialise_value(reader)?,
                        ))
                    } else {
                        ::core::result::Result::Ok(#cd::serialisation::EvmCdHead::Value(
                            <Self as #trait_path>::deserialise_reader(reader)?,
                        ))
                    }
                }

                fn deserialise_abi_finish<#io_ident: #cd::serialisation::Read>(
                    head: #cd::serialisation::EvmCdHead<Self>,
                    expected_tail_offset: usize,
                    reader: &mut #io_ident,
                ) -> ::core::result::Result<Self, #cd::serialisation::Error> {
                    match head {
                        #cd::serialisation::EvmCdHead::Offset(offset)
                            if <Self as #trait_path>::is_abi_dynamic() && offset == expected_tail_offset =>
                        {
                            <Self as #trait_path>::deserialise_reader(reader)
                        }
                        #cd::serialisation::EvmCdHead::Value(value)
                            if !<Self as #trait_path>::is_abi_dynamic() =>
                        {
                            ::core::result::Result::Ok(value)
                        }
                        _ => ::core::result::Result::Err(#cd::serialisation::invalid_data()),
                    }
                }
            }
        }
    }
}

fn static_struct_serialised_size(data: &DataStruct) -> Option<usize> {
    data.fields.iter().try_fold(0usize, |size, field| {
        size.checked_add(static_abi_value_size(&field.ty)?)
    })
}

fn static_abi_value_size(ty: &Type) -> Option<usize> {
    match ty {
        Type::Path(path) if path.qself.is_none() => {
            let segment = path.path.segments.last()?;
            if !matches!(segment.arguments, syn::PathArguments::None) {
                return None;
            }
            matches!(
                segment.ident.to_string().as_str(),
                "U" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "EvmCdAddress" | "Address"
            )
            .then_some(32)
        }
        Type::Array(array)
            if matches!(
                array.elem.as_ref(),
                Type::Path(elem) if elem.qself.is_none() && elem.path.is_ident("u8")
            ) =>
        {
            Some(32)
        }
        Type::Group(group) => static_abi_value_size(&group.elem),
        Type::Paren(paren) => static_abi_value_size(&paren.elem),
        _ => None,
    }
}

fn abi_type_body(data: &Data, trait_path: &TokenStream2) -> TokenStream2 {
    match data {
        Data::Enum(_) => quote!(hasher.update(b"uint8")),
        Data::Struct(data) => {
            let types: Vec<_> = data.fields.iter().map(|field| &field.ty).collect();
            append_types(&types, trait_path, quote!(hasher.update(b"(")), b")")
        }
        Data::Union(_) => TokenStream2::new(),
    }
}

fn append_types(
    types: &[&Type],
    trait_path: &TokenStream2,
    initial: TokenStream2,
    suffix: &'static [u8],
) -> TokenStream2 {
    let updates = types.iter().enumerate().map(|(index, ty)| {
        let comma = (index > 0).then(|| quote!(hasher = hasher.update(b",");));
        quote! {
            #comma
            hasher = <#ty as #trait_path>::append_abi_type(hasher);
        }
    });
    let suffix = syn::LitByteStr::new(suffix, proc_macro2::Span::call_site());
    quote! {
        let mut hasher = #initial;
        #(#updates)*
        hasher.update(#suffix)
    }
}

/// Try to compute the canonical Solidity ABI type name for a type the derive
/// can resolve syntactically (bobcat primitives and SDK container types only).
///
/// Returns `None` for anything that would need compile-time trait-based name
/// resolution (type aliases, derived structs/enums, generic parameters, user
/// types). The caller then falls back to the runtime `SelectorHasher` path, so
/// behaviour is unchanged for unresolvable field types.
fn abi_type_name(ty: &Type) -> Option<Vec<u8>> {
    match ty {
        Type::Path(path) => {
            if path.qself.is_some() {
                return None;
            }
            path_abi_type_name(&path.path)
        }
        Type::Array(arr) => {
            // The SDK implements `EvmCdSerialise for [u8; N]` only.
            let is_u8 = matches!(
                arr.elem.as_ref(),
                Type::Path(elem) if elem.qself.is_none() && elem.path.is_ident("u8")
            );
            if !is_u8 {
                return None;
            }
            // The length must be a plain integer literal to resolve here.
            let n: u64 = match &arr.len {
                syn::Expr::Lit(lit) => match &lit.lit {
                    syn::Lit::Int(int) => int.base10_parse().ok()?,
                    _ => return None,
                },
                _ => return None,
            };
            Some(format!("bytes{n}").into_bytes())
        }
        _ => None,
    }
}

fn path_abi_type_name(path: &syn::Path) -> Option<Vec<u8>> {
    let segment = path.segments.last()?;
    let name = segment.ident.to_string();
    // First generic type argument, if the path is generic (e.g. `Vec<T>`).
    let first_type_arg = || -> Option<&Type> {
        let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
            return None;
        };
        args.args.iter().find_map(|arg| match arg {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
    };
    let is_u8_ty = |ty: &Type| {
        matches!(
            ty,
            Type::Path(elem) if elem.qself.is_none() && elem.path.is_ident("u8")
        )
    };
    match name.as_str() {
        "U" => Some(b"uint256".to_vec()),
        "u8" => Some(b"uint8".to_vec()),
        "u16" => Some(b"uint16".to_vec()),
        "u32" => Some(b"uint32".to_vec()),
        "u64" => Some(b"uint64".to_vec()),
        "u128" => Some(b"uint128".to_vec()),
        "usize" => Some(b"uint32".to_vec()),
        "EvmCdAddress" | "Address" => Some(b"address".to_vec()),
        "EvmCdString" => Some(b"string".to_vec()),
        "Vec" => {
            let elem = first_type_arg()?;
            if is_u8_ty(elem) {
                Some(b"bytes".to_vec())
            } else {
                let mut out = abi_type_name(elem)?;
                out.extend_from_slice(b"[]");
                Some(out)
            }
        }
        "EvmCdArray" => {
            let elem = first_type_arg()?;
            let mut out = abi_type_name(elem)?;
            out.extend_from_slice(b"[]");
            Some(out)
        }
        _ => None,
    }
}

/// Build the canonical selector signature `"name(type1,...,typeN)"` for a
/// variant if every field type can be resolved to an ABI name at expansion
/// time; otherwise `None` (the caller uses the runtime hasher).
fn try_selector_signature(variant: &syn::Variant, function_name: &str) -> Option<Vec<u8>> {
    let mut sig = function_name.as_bytes().to_vec();
    sig.extend_from_slice(b"(");
    for (index, field) in variant.fields.iter().enumerate() {
        if index > 0 {
            sig.extend_from_slice(b",");
        }
        sig.extend_from_slice(&abi_type_name(&field.ty)?);
    }
    sig.push(b')');
    Some(sig)
}

/// Keccak-256 of the signature, truncated to the EVM 4-byte selector.
fn selector_literal(signature: &[u8]) -> [u8; 4] {
    let digest = keccak_const::Keccak256::new().update(signature).finalize();
    [digest[0], digest[1], digest[2], digest[3]]
}

fn selector_expr(
    variant: &syn::Variant,
    trait_path: &TokenStream2,
    cd: &TokenStream2,
) -> TokenStream2 {
    let function_name = variant.ident.to_string().to_lower_camel_case();

    // Preferred path: precompute the 4-byte selector at macro-expansion time so
    // no keccak code is emitted into (or linked by) the contract wasm.
    if let Some(signature) = try_selector_signature(variant, &function_name) {
        let [a, b, c, d] = selector_literal(&signature);
        return quote!([#a, #b, #c, #d]);
    }

    // Fallback: runtime trait-based hashing for types the derive can't resolve
    // syntactically (aliases, derived structs/enums, generic parameters).
    let prefix = format!("{function_name}(");
    let prefix = syn::LitByteStr::new(prefix.as_bytes(), variant.ident.span());
    let types: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
    let hasher = append_types(
        &types,
        trait_path,
        quote!(#cd::serialisation::SelectorHasher::new().update(#prefix)),
        b")",
    );
    quote!({ { #hasher }.selector() })
}

fn serialise_tuple(fields: &Fields, cd: &TokenStream2) -> TokenStream2 {
    let accesses = field_accesses(fields);
    let types: Vec<_> = fields.iter().map(|field| &field.ty).collect();
    let head_sizes = types
        .iter()
        .map(|ty| quote!(<#ty as #cd::serialisation::EvmCdSerialise>::abi_head_size()));
    let heads = accesses.iter().zip(types.iter()).map(|(access, ty)| quote! {
        <#ty as #cd::serialisation::EvmCdSerialise>::serialise_abi_head(
            &self.#access,
            __evm_cd_tail_offset,
            writer,
        )?;
        __evm_cd_tail_offset = __evm_cd_tail_offset
            .checked_add(<#ty as #cd::serialisation::EvmCdSerialise>::abi_tail_size(&self.#access))
            .ok_or_else(#cd::serialisation::invalid_data)?;
    });
    let tails = accesses.iter().zip(types.iter()).map(|(access, ty)| {
        quote! {
            <#ty as #cd::serialisation::EvmCdSerialise>::serialise_abi_tail(
                &self.#access,
                writer,
            )?;
        }
    });
    quote! {
        let mut __evm_cd_tail_offset = 0usize #(
            .checked_add(#head_sizes)
            .ok_or_else(#cd::serialisation::invalid_data)?
        )*;
        #(#heads)*
        #(#tails)*
        ::core::result::Result::Ok(())
    }
}

fn serialise_struct(data: &DataStruct, cd: &TokenStream2) -> TokenStream2 {
    serialise_tuple(&data.fields, cd)
}

fn deserialise_struct(data: &DataStruct, cd: &TokenStream2) -> TokenStream2 {
    let types: Vec<_> = data.fields.iter().map(|field| &field.ty).collect();
    let heads: Vec<_> = (0..types.len())
        .map(|index| format_ident!("__evm_cd_head_{index}"))
        .collect();
    let values: Vec<_> = (0..types.len())
        .map(|index| format_ident!("__evm_cd_value_{index}"))
        .collect();
    let read_heads = heads.iter().zip(types.iter()).map(|(head, ty)| quote! {
        let #head = <#ty as #cd::serialisation::EvmCdDeserialise>::deserialise_abi_head(reader)?;
    });
    let head_sizes = types
        .iter()
        .map(|ty| quote!(<#ty as #cd::serialisation::EvmCdDeserialise>::abi_head_size()));
    let finish = values.iter().zip(heads.iter()).zip(types.iter()).map(
        |((value, head), ty)| quote! {
            let #value = <#ty as #cd::serialisation::EvmCdDeserialise>::deserialise_abi_finish(
                #head,
                __evm_cd_tail_offset,
                reader,
            )?;
            __evm_cd_tail_offset = __evm_cd_tail_offset
                .checked_add(<#ty as #cd::serialisation::EvmCdDeserialise>::abi_tail_size(&#value))
                .ok_or_else(#cd::serialisation::invalid_data)?;
        },
    );
    let construct = match &data.fields {
        Fields::Unit => quote!(Self),
        Fields::Unnamed(_) => quote!(Self(#(#values),*)),
        Fields::Named(fields) => {
            let names = fields
                .named
                .iter()
                .map(|field| field.ident.as_ref().unwrap());
            quote!(Self { #(#names: #values),* })
        }
    };
    quote! {
        #(#read_heads)*
        let mut __evm_cd_tail_offset = 0usize #(
            .checked_add(#head_sizes)
            .ok_or_else(#cd::serialisation::invalid_data)?
        )*;
        #(#finish)*
        ::core::result::Result::Ok(#construct)
    }
}

fn validate_enum(data: &DataEnum, evm_values: bool) -> syn::Result<()> {
    if data.variants.len() > 256 {
        return Err(syn::Error::new_spanned(
            &data.variants[256],
            "EVM calldata enums support at most 256 variants",
        ));
    }
    for variant in &data.variants {
        if evm_values && !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "EVM value enums may only contain fieldless variants; add #[evm_entrypoint] if this enum represents function calls",
            ));
        }
        if !evm_values && variant.discriminant.is_some() {
            return Err(syn::Error::new_spanned(
                variant,
                "explicit enum discriminants are not supported on #[evm_entrypoint] enums; entrypoint variants are encoded by declaration order when nested",
            ));
        }
    }
    Ok(())
}

fn variant_pattern(name: &syn::Ident, variant: &syn::Variant) -> (TokenStream2, Vec<syn::Ident>) {
    let variant_name = &variant.ident;
    match &variant.fields {
        Fields::Unit => (quote!(#name::#variant_name), Vec::new()),
        Fields::Unnamed(fields) => {
            let bindings: Vec<_> = (0..fields.unnamed.len())
                .map(|field| format_ident!("__evm_cd_field_{field}"))
                .collect();
            (quote!(#name::#variant_name(#(#bindings),*)), bindings)
        }
        Fields::Named(fields) => {
            let names: Vec<_> = fields
                .named
                .iter()
                .map(|field| field.ident.clone().unwrap())
                .collect();
            let bindings: Vec<_> = (0..names.len())
                .map(|field| format_ident!("__evm_cd_field_{field}"))
                .collect();
            (
                quote!(#name::#variant_name { #(#names: #bindings),* }),
                bindings,
            )
        }
    }
}

fn serialise_enum(
    name: &syn::Ident,
    data: &DataEnum,
    cd: &TokenStream2,
) -> syn::Result<TokenStream2> {
    validate_enum(data, false)?;
    let trait_path = trait_path(Direction::Serialise, cd);
    let selectors: Vec<_> = data
        .variants
        .iter()
        .map(|variant| selector_expr(variant, &trait_path, cd))
        .collect();
    let arms = data.variants.iter().map(|variant| {
        let (pattern, bindings) = variant_pattern(name, variant);
        let selector = selector_expr(variant, &trait_path, cd);
        let types: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
        let head_sizes = types.iter().map(
            |ty| quote!(<#ty as #trait_path>::abi_head_size()),
        );
        let heads = bindings.iter().zip(types.iter()).map(|(binding, ty)| quote! {
            <#ty as #trait_path>::serialise_abi_head(#binding, tail_offset, writer)?;
            tail_offset = tail_offset
                .checked_add(<#ty as #trait_path>::abi_tail_size(#binding))
                .ok_or_else(#cd::serialisation::invalid_data)?;
        });
        let tails = bindings.iter().zip(types.iter()).map(|(binding, ty)| quote! {
            <#ty as #trait_path>::serialise_abi_tail(#binding, writer)?;
        });
        quote! {
            #pattern => {
                let selector = #selector;
                let selector_matches = 0usize #(+ (selector == #selectors) as usize)*;
                if selector_matches != 1 {
                    return ::core::result::Result::Err(#cd::serialisation::invalid_data());
                }
                writer.write_all(&selector)?;
                let mut tail_offset = 0usize #(.checked_add(#head_sizes).ok_or_else(#cd::serialisation::invalid_data)?)*;
                #(#heads)*
                #(#tails)*
            }
        }
    });
    Ok(quote! {
        match self { #(#arms),* }
        ::core::result::Result::Ok(())
    })
}

fn serialise_enum_value(
    name: &syn::Ident,
    data: &DataEnum,
    cd: &TokenStream2,
    evm_values: bool,
) -> syn::Result<TokenStream2> {
    validate_enum(data, evm_values)?;
    let arms = data.variants.iter().enumerate().map(|(index, variant)| {
        let (pattern, _) = variant_pattern(name, variant);
        let discriminant = index as u8;
        if matches!(variant.fields, Fields::Unit) {
            if evm_values {
                let variant_name = &variant.ident;
                quote! {
                    #pattern => match <u8 as ::core::convert::TryFrom<i128>>::try_from(
                        #name::#variant_name as i128,
                    ) {
                        ::core::result::Result::Ok(discriminant) => {
                            #cd::serialisation::EvmCdSerialise::serialise_value(
                                &discriminant,
                                writer,
                            )
                        }
                        ::core::result::Result::Err(_) => {
                            ::core::result::Result::Err(#cd::serialisation::invalid_data())
                        }
                    }
                }
            } else {
                quote! {
                    #pattern => #cd::serialisation::EvmCdSerialise::serialise_value(
                        &#discriminant,
                        writer,
                    )
                }
            }
        } else {
            quote! {
                #pattern => ::core::result::Result::Err(#cd::serialisation::invalid_data())
            }
        }
    });
    Ok(quote!(match self { #(#arms),* }))
}

fn deserialise_variant(
    name: &syn::Ident,
    variant: &syn::Variant,
    cd: &TokenStream2,
) -> TokenStream2 {
    let variant_name = &variant.ident;
    match &variant.fields {
        Fields::Unit => quote!(::core::result::Result::Ok(#name::#variant_name)),
        Fields::Unnamed(fields) => {
            let values = fields
                .unnamed
                .iter()
                .map(|_| quote!(#cd::serialisation::EvmCdDeserialise::deserialise_value(reader)?));
            quote!(::core::result::Result::Ok(#name::#variant_name(#(#values),*)))
        }
        Fields::Named(fields) => {
            let names = fields
                .named
                .iter()
                .map(|field| field.ident.as_ref().unwrap());
            quote! {
                ::core::result::Result::Ok(#name::#variant_name {
                    #(#names: #cd::serialisation::EvmCdDeserialise::deserialise_value(reader)?),*
                })
            }
        }
    }
}

fn deserialise_enum(
    name: &syn::Ident,
    data: &DataEnum,
    cd: &TokenStream2,
) -> syn::Result<TokenStream2> {
    validate_enum(data, false)?;
    let trait_path = trait_path(Direction::Deserialise, cd);
    let selectors: Vec<_> = data
        .variants
        .iter()
        .map(|variant| selector_expr(variant, &trait_path, cd))
        .collect();
    let branches = data.variants.iter().map(|variant| {
        let selector = selector_expr(variant, &trait_path, cd);
        let variant_name = &variant.ident;
        let types: Vec<_> = variant.fields.iter().map(|field| &field.ty).collect();
        let heads: Vec<_> = (0..types.len())
            .map(|index| format_ident!("__evm_cd_head_{index}"))
            .collect();
        let values: Vec<_> = (0..types.len())
            .map(|index| format_ident!("__evm_cd_value_{index}"))
            .collect();
        let read_heads = heads.iter().zip(types.iter()).map(|(head, ty)| quote! {
            let #head = <#ty as #trait_path>::deserialise_abi_head(reader)?;
        });
        let head_sizes = types.iter().map(
            |ty| quote!(<#ty as #trait_path>::abi_head_size()),
        );
        let finish = values.iter().zip(heads.iter()).zip(types.iter()).map(
            |((value, head), ty)| quote! {
                let #value = <#ty as #trait_path>::deserialise_abi_finish(
                    #head,
                    tail_offset,
                    reader,
                )?;
                tail_offset = tail_offset
                    .checked_add(<#ty as #trait_path>::abi_tail_size(&#value))
                    .ok_or_else(#cd::serialisation::invalid_data)?;
            },
        );
        let construct = match &variant.fields {
            Fields::Unit => quote!(#name::#variant_name),
            Fields::Unnamed(_) => quote!(#name::#variant_name(#(#values),*)),
            Fields::Named(fields) => {
                let names = fields.named.iter().map(|field| field.ident.as_ref().unwrap());
                quote!(#name::#variant_name { #(#names: #values),* })
            }
        };
        quote! {
            if selector == #selector {
                #(#read_heads)*
                let mut tail_offset = 0usize #(.checked_add(#head_sizes).ok_or_else(#cd::serialisation::invalid_data)?)*;
                #(#finish)*
                return ::core::result::Result::Ok(#construct);
            }
        }
    });
    Ok(quote! {
        let mut selector = [0u8; 4];
        reader.read_exact(&mut selector)?;
        let selector_matches = 0usize #(+ (selector == #selectors) as usize)*;
        if selector_matches != 1 {
            return ::core::result::Result::Err(#cd::serialisation::invalid_data());
        }
        #(#branches)*
        ::core::result::Result::Err(#cd::serialisation::invalid_data())
    })
}

fn deserialise_enum_value(
    name: &syn::Ident,
    data: &DataEnum,
    cd: &TokenStream2,
    evm_values: bool,
) -> syn::Result<TokenStream2> {
    validate_enum(data, evm_values)?;
    let arms = data.variants.iter().enumerate().map(|(index, variant)| {
        let discriminant = index as u8;
        if matches!(variant.fields, Fields::Unit) {
            let value = deserialise_variant(name, variant, cd);
            if evm_values {
                let variant_name = &variant.ident;
                quote! {
                    discriminant if discriminant as i128 == #name::#variant_name as i128 => #value
                }
            } else {
                quote!(#discriminant => #value)
            }
        } else {
            quote!(#discriminant => ::core::result::Result::Err(#cd::serialisation::invalid_data()))
        }
    });
    Ok(quote! {
        let discriminant: u8 = #cd::serialisation::EvmCdDeserialise::deserialise_value(reader)?;
        match discriminant {
            #(#arms,)*
            _ => ::core::result::Result::Err(#cd::serialisation::invalid_data()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use keccak_const::Keccak256;

    fn sel(signature: &str) -> [u8; 4] {
        let digest = Keccak256::new().update(signature.as_bytes()).finalize();
        [digest[0], digest[1], digest[2], digest[3]]
    }

    fn abi(ty: &str) -> String {
        let ty: Type = syn::parse_str(ty).unwrap();
        String::from_utf8(abi_type_name(&ty).expect("should resolve")).unwrap()
    }

    #[test]
    fn abi_type_names_match_the_sdk_trait_impls() {
        assert_eq!(abi("U"), "uint256");
        assert_eq!(abi("bobcat_maths::U"), "uint256");
        assert_eq!(abi("u8"), "uint8");
        assert_eq!(abi("u16"), "uint16");
        assert_eq!(abi("u32"), "uint32");
        assert_eq!(abi("u64"), "uint64");
        assert_eq!(abi("u128"), "uint128");
        assert_eq!(abi("usize"), "uint32");
        assert_eq!(abi("EvmCdAddress"), "address");
        assert_eq!(abi("Address"), "address");
        assert_eq!(abi("[u8; 4]"), "bytes4");
        assert_eq!(abi("[u8; 20]"), "bytes20");
        assert_eq!(abi("Vec<u8>"), "bytes");
        assert_eq!(abi("Vec<U>"), "uint256[]");
        assert_eq!(abi("Vec<EvmCdAddress>"), "address[]");
        assert_eq!(abi("EvmCdArray<u8, 0, 4>"), "uint8[]");
        assert_eq!(abi("EvmCdArray<U, 0, 4>"), "uint256[]");
        assert_eq!(abi("EvmCdString<0, 32>"), "string");
    }

    #[test]
    fn unresolvable_types_fall_back_to_runtime_hashing() {
        for ty_str in [
            "Name",      // type alias
            "Asset",     // derived enum -> uint8 (only known via its impl)
            "DogRecord", // derived struct -> tuple (only known via its impl)
            "T",         // generic parameter
            "&[u8]",     // not a supported ABI type
            "[u32; 4]",  // SDK implements [u8; N] only
        ] {
            let ty: Type = syn::parse_str(ty_str).unwrap();
            assert!(
                abi_type_name(&ty).is_none(),
                "{ty_str} should be unresolvable"
            );
        }
    }

    #[test]
    fn precomputed_variant_selectors_match_the_reference_keccak() {
        let variants = [
            ("setNumber(uint256)", "SetNumber(U)"),
            ("fixedBytes(bytes4,uint8)", "FixedBytes([u8; 4], u8)"),
            ("addNumber(uint256)", "AddNumber(U)"),
            ("number()", "Number"),
        ];
        for (expected_sig, variant_text) in variants {
            let variant: syn::Variant = syn::parse_str(variant_text).unwrap();
            let function_name = variant.ident.to_string().to_lower_camel_case();
            let signature = try_selector_signature(&variant, &function_name).unwrap();
            assert_eq!(signature, expected_sig.as_bytes());
            assert_eq!(selector_literal(&signature), sel(expected_sig));
        }
    }
}
