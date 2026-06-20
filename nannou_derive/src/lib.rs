use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprLit, ItemStruct, Lit, Meta, Token, parse_macro_input};

#[proc_macro_attribute]
pub fn shader_model(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let attrs = parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);

    let (vertex_shader, fragment_shader) = parse_shader_attributes(&attrs);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let vertex_shader_impl = vertex_shader
        .map(|path| quote! { #path.into() })
        .unwrap_or_else(|| quote! { ::nannou::prelude::ShaderRef::Default });

    let fragment_shader_impl = fragment_shader
        .map(|path| quote! { #path.into() })
        .unwrap_or_else(|| quote! { ::nannou::prelude::ShaderRef::Default });

    let expanded = quote! {
        #[derive(::bevy::asset::Asset, ::bevy::prelude::TypePath, ::nannou::prelude::AsBindGroup, Debug, Clone, Default)]
        #input

        impl #impl_generics ::nannou::prelude::render::ShaderModel for #name #ty_generics #where_clause {
            fn vertex_shader() -> ::nannou::prelude::ShaderRef {
                #vertex_shader_impl
            }

            fn fragment_shader() -> ::nannou::prelude::ShaderRef {
                #fragment_shader_impl
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_shader_attributes(
    attrs: &Punctuated<Meta, Token![,]>,
) -> (Option<String>, Option<String>) {
    let mut vertex_shader = None;
    let mut fragment_shader = None;

    for meta in attrs {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("vertex") {
                if let Some(value) = lit_str_value(&nv.value) {
                    vertex_shader = Some(value);
                }
            } else if nv.path.is_ident("fragment") {
                if let Some(value) = lit_str_value(&nv.value) {
                    fragment_shader = Some(value);
                }
            }
        }
    }

    (vertex_shader, fragment_shader)
}

/// Extract the value of a string-literal expression (`syn 2` stores attribute
/// `name = value` values as an `Expr` rather than a `Lit`).
fn lit_str_value(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(lit), ..
        }) => Some(lit.value()),
        _ => None,
    }
}
