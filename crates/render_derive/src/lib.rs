mod enum_variant_meta;
mod modules;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Field, Fields};

#[proc_macro_derive(EnumVariantMeta)]
pub fn derive_enum_variant_meta(input: TokenStream) -> TokenStream {
    enum_variant_meta::derive_enum_variant_meta(input)
}

#[proc_macro_derive(NodeParam, attributes(field_slots))]
pub fn derive_node_param(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let field_slots = ast
        .attrs
        .iter()
        .find(|attr| attr.path.segments[0].ident == "field_slots")
        .map(|_| true);

    let r = if field_slots.unwrap_or(false) {
        // 含了 field_slots 每个字段 单独展开
        let named_fields = match &ast.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(fields),
                ..
            }) => &fields.named,
            _ => panic!("Expected a struct with named fields."),
        };

        // 只对 共有 字段 展开
        let pub_fileds = named_fields
            .iter()
            .filter(|field| matches!(&field.vis, syn::Visibility::Public(_)))
            .collect::<Vec<&Field>>();

        if pub_fileds.is_empty() {
            panic!("{:?} isn't no pub field", name);
        }

        let inputs = pub_fileds
            .iter()
            .map(|field| {
                let name = field.ident.as_ref().unwrap();
                quote! {
                    let r = pi_render::graph_new::param::InParam::fill_from(&mut self.#name, pre_id, out_param);
                    println!("input field name = {}, result = {}", #name, r);
                    if r {
                        return true;
					}
                }
            })
            .collect::<Vec<TokenStream2>>();

        let outputs = pub_fileds
            .iter()
            .map(|field| {
                let name = field.ident.as_ref().unwrap();
                quote! {
                    let r = pi_render::graph_new::param::OutParam::get_content(&self.#name, ty);
                    println!("output field name = {}, ty = {:?}, r = {}", #name, ty, r);
                    if r != 0 {
                        return r;
                    }
                }
            })
            .collect::<Vec<TokenStream2>>();

        quote! {
            impl #impl_generics pi_render::graph_new::param::OutParam for #name #ty_generics #where_clause {
                fn get_content(&self, ty: std::any::TypeId) -> usize {
                    #(#outputs)*
                    
                    0
                }
            }

            impl #impl_generics pi_render::graph_new::param::InParam for #name #ty_generics #where_clause {
                fn fill_from<T: pi_render::graph_new::param::OutParam + ?Sized>(&mut self, pre_id: pi_render::graph_new::node::NodeId, out_param: &T) -> bool {
                    #(#inputs;)*
                    
                    println!("failed input");
                        
                    false
                }
            }
        }
    } else {
        // 整体 作为 输入 输入 参数
        quote! {
            impl #impl_generics pi_render::graph_new::param::OutParam for #name #ty_generics #where_clause {
                fn get_content(&self, ty: std::any::TypeId) -> usize {
                    if ty == std::any::TypeId::of::<Self>() {
                        let c = Clone::clone(self);
                        
                        // 隐藏条件，必须 为 Self 实现 Clone
                        let p = &c as *const Self as usize;

                        // 注: Copy 和 Drop 不能 共存
                        // 不能 释放放这个 c，因为 c 是要拿去 填充 输入的
                        std::mem::forget(c);

                        p
                    } else {
                        0
                    }
                }
            }

            impl #impl_generics pi_render::graph_new::param::InParam for #name #ty_generics #where_clause {
                fn fill_from<T: pi_render::graph_new::param::OutParam + ?Sized>(
                    &mut self,
                    pre_id: pi_render::graph_new::node::NodeId,
                    out_param: &T,
                ) -> bool {
                    let v = out_param.get_content(std::any::TypeId::of::<Self>());

                    println!("Input fill_from, type = {:?}, v = {}", std::any::TypeId::of::<Self>(), v);
                    
                    if v != 0 {
                        *self = unsafe {
                            std::ptr::read(v as *const Self)
                        };
                    }

                    v != 0
                }
            }
        }
    };
    r.into()
}
