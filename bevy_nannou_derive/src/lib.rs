use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, Lit, Meta, NestedMeta, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn shader_model(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let attrs = parse_macro_input!(attr as syn::AttributeArgs);

    let (vertex_shader, fragment_shader) = parse_shader_attributes(&attrs);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let vertex_shader_impl = vertex_shader
        .map(|path| quote! { #path.into() })
        .unwrap_or_else(|| quote! { ::bevy_nannou::prelude::ShaderRef::Default });

    let fragment_shader_impl = fragment_shader
        .map(|path| quote! { #path.into() })
        .unwrap_or_else(|| quote! { ::bevy_nannou::prelude::ShaderRef::Default });

    // Add derive attributes
    input
        .attrs
        .push(parse_quote!(#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]));

    let expanded = quote! {
        #input

        impl #impl_generics ::bevy_nannou::prelude::render::ShaderModel for #name #ty_generics #where_clause {
            fn vertex_shader() -> ::bevy_nannou::prelude::ShaderRef {
                #vertex_shader_impl
            }

            fn fragment_shader() -> ::bevy_nannou::prelude::ShaderRef {
                #fragment_shader_impl
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_shader_attributes(attrs: &[syn::NestedMeta]) -> (Option<String>, Option<String>) {
    let mut vertex_shader = None;
    let mut fragment_shader = None;

    for attr in attrs {
        if let NestedMeta::Meta(Meta::NameValue(nv)) = attr {
            if nv.path.is_ident("vertex") {
                if let Lit::Str(lit) = &nv.lit {
                    vertex_shader = Some(lit.value());
                }
            } else if nv.path.is_ident("fragment") {
                if let Lit::Str(lit) = &nv.lit {
                    fragment_shader = Some(lit.value());
                }
            }
        }
    }

    (vertex_shader, fragment_shader)
}
