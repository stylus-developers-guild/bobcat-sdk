#![crate_type = "proc-macro"]

extern crate proc_macro;

use heck::ToLowerCamelCase;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields, Generics, Type, parse_macro_input};

#[proc_macro_derive(EvmCdSerialise)]
pub fn derive_evm_cd_serialise(input: TokenStream) -> TokenStream {
    expand(
        parse_macro_input!(input as DeriveInput),
        Direction::Serialise,
    )
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

#[proc_macro_derive(EvmCdDeserialise)]
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
    let io_ident = fresh_type_ident(
        &input.generics,
        match direction {
            Direction::Serialise => "__EvmCdWriter",
            Direction::Deserialise => "__EvmCdReader",
        },
    );
    let field_types = all_field_types(&input.data);
    let trait_path = trait_path(direction, &cd);
    let generics = add_field_bounds(input.generics.clone(), &field_types, &trait_path);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = match (&input.data, direction) {
        (Data::Struct(data), Direction::Serialise) => serialise_struct(data, &cd),
        (Data::Struct(data), Direction::Deserialise) => deserialise_struct(data, &cd),
        (Data::Enum(data), Direction::Serialise) => serialise_enum(name, data, &cd)?,
        (Data::Enum(data), Direction::Deserialise) => deserialise_enum(name, data, &cd)?,
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
            let body = serialise_enum_value(name, data, &cd)?;
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
            let body = deserialise_enum_value(name, data, &cd)?;
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

    let output = match direction {
        Direction::Serialise => quote! {
            #[automatically_derived]
            impl #impl_generics #cd::serialisation::EvmCdSerialise for #name #ty_generics #where_clause {
                fn serialise<#io_ident: #cd::serialisation::Write>(
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
        },
        Direction::Deserialise => quote! {
            #[automatically_derived]
            impl #impl_generics #cd::serialisation::EvmCdDeserialise for #name #ty_generics #where_clause {
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

fn selector_expr(
    variant: &syn::Variant,
    trait_path: &TokenStream2,
    cd: &TokenStream2,
) -> TokenStream2 {
    let function_name = variant.ident.to_string().to_lower_camel_case();
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

fn validate_enum(data: &DataEnum) -> syn::Result<()> {
    if data.variants.len() > 256 {
        return Err(syn::Error::new_spanned(
            &data.variants[256],
            "EVM calldata enums support at most 256 variants",
        ));
    }
    for variant in &data.variants {
        if variant.discriminant.is_some() {
            return Err(syn::Error::new_spanned(
                variant,
                "explicit enum discriminants are not supported; variants are encoded by declaration order when nested",
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
    validate_enum(data)?;
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
) -> syn::Result<TokenStream2> {
    validate_enum(data)?;
    let arms = data.variants.iter().enumerate().map(|(index, variant)| {
        let (pattern, _) = variant_pattern(name, variant);
        let discriminant = index as u8;
        if matches!(variant.fields, Fields::Unit) {
            quote! {
                #pattern => #cd::serialisation::EvmCdSerialise::serialise_value(
                    &#discriminant,
                    writer,
                )
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
    validate_enum(data)?;
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
) -> syn::Result<TokenStream2> {
    validate_enum(data)?;
    let arms = data.variants.iter().enumerate().map(|(index, variant)| {
        let discriminant = index as u8;
        if matches!(variant.fields, Fields::Unit) {
            let value = deserialise_variant(name, variant, cd);
            quote!(#discriminant => #value)
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
