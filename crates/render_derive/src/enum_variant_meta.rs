use render_macro_utils::RenderManifest;
use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

pub fn derive_enum_variant_meta(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let variants = match &ast.data {
        Data::Enum(v) => &v.variants,
        _ => {
            return syn::Error::new(Span::call_site().into(), "Only enums are supported")
                .into_compile_error()
                .into()
        }
    };

    let render_util_path = RenderManifest::default().get_path(crate::modules::RENDER_UTILS);

    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let struct_name = &ast.ident;
    let idents = variants.iter().map(|v| &v.ident);
    let names = variants.iter().map(|v| v.ident.to_string());
    let indices = 0..names.len();

    TokenStream::from(quote! {
        impl #impl_generics #render_util_path::EnumVariantMeta for #struct_name #ty_generics #where_clause {
            fn enum_variant_index(&self) -> usize {
                match self {
                    #(#struct_name::#idents {..} => #indices,)*
                }
            }
            fn enum_variant_name(&self) -> &'static str {
                static variants: &[&str] = &[
                    #(#names,)*
                ];
                let index = self.enum_variant_index();
                variants[index]
            }
        }
    })
}
