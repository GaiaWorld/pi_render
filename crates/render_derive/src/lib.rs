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

        // 如果 所有的 字段 都不是 pub 字段，报错
        if pub_fileds.is_empty() {
            panic!("Type {:?} isn't no pub field", name);
        }

        let can_inputs = pub_fileds
            .iter()
            .map(|field| {
                let name = field.ident.as_ref().unwrap();
                quote! {
                    r |= pi_render::graph::param::InParam::can_fill(&self.#name, map, pre_id, out_param);
                }
            })
            .collect::<Vec<TokenStream2>>();

        let input_fills = pub_fileds
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            quote! {
                r |= pi_render::graph::param::InParam::fill_from(&mut self.#name, pre_id, out_param);
            }
        })
        .collect::<Vec<TokenStream2>>();

        let output_fills = pub_fileds
            .iter()
            .map(|field| {
                let name = field.ident.as_ref().unwrap();
                quote! {
                    if pi_render::graph::param::OutParam::fill_to(&self.#name, this_id, to, ty) {
                        return true;
                    }
                }
            })
            .collect::<Vec<TokenStream2>>();

        let can_outputs = pub_fileds
            .iter()
            .map(|field| {
                let name = field.ident.as_ref().unwrap();
                quote! {
                    if pi_render::graph::param::OutParam::can_fill(&self.#name, set, ty) {
                        return true;
                    }
                }
            })
            .collect::<Vec<TokenStream2>>();
        quote! {
            impl #impl_generics pi_render::graph::param::OutParam for #name #ty_generics #where_clause {

                fn can_fill(&self, set: &mut Option<&mut pi_hash::XHashSet<std::any::TypeId>>, ty: std::any::TypeId) -> bool {
                    #(#can_outputs)*

                    false
                }

                fn fill_to(&self, this_id: pi_render::graph::node::NodeId, to: &mut dyn pi_render::graph::param::Assign, ty: std::any::TypeId) -> bool {
                    #(#output_fills)*

                    false
                }
            }

            impl #impl_generics pi_render::graph::param::InParam for #name #ty_generics #where_clause {
                fn can_fill<O: pi_render::graph::param::OutParam + ?Sized>(
                    &self,
                    map: &mut pi_hash::XHashMap<std::any::TypeId, Vec<pi_render::graph::NodeId>>,
                    pre_id: pi_render::graph::node::NodeId,
                    out_param: &O,
                ) -> bool {
                    let mut r = false;

                    #(#can_inputs;)*

                    r
                }

                fn fill_from<O: pi_render::graph::param::OutParam + ?Sized>(&mut self, pre_id: pi_render::graph::node::NodeId, out_param: &O) -> bool {
                    let mut r = false;

                    #(#input_fills;)*

                    r
                }
            }
        }
    } else {
        // 整体 作为 输入 输入 参数
        quote! {
            impl #impl_generics pi_render::graph::param::OutParam for #name #ty_generics #where_clause {
                fn can_fill(&self, set: &mut Option<&mut pi_hash::XHashSet<std::any::TypeId>>, ty: std::any::TypeId) -> bool {
                    let r = ty == std::any::TypeId::of::<Self>();
                    if r && set.is_some() {
                        match set {
                            None => {}
                            Some(s) => {
                                s.insert(ty);
                            }
                        }

                    }
                    r
                }

                fn fill_to(&self, this_id: pi_render::graph::node::NodeId, to: &mut dyn pi_render::graph::param::Assign, ty: std::any::TypeId) -> bool {
                    let r = ty == std::any::TypeId::of::<Self>();
                    if r {
                        let c = Clone::clone(self);

                        // 隐藏条件，必须 为 Self 实现 Clone
                        let p = &c as *const Self as usize;
                        to.assign(this_id, p);

                        // 注: Copy 和 Drop 不能 共存
                        // 不能 释放放这个 c，因为 c 是要拿去 填充 输入的
                        std::mem::forget(c);
                    }
                    r
                }
            }

            impl #impl_generics pi_render::graph::param::InParam for #name #ty_generics #where_clause {

                fn can_fill<O: pi_render::graph::param::OutParam + ?Sized>(
                    &self,
                    map: &mut pi_hash::XHashMap<std::any::TypeId, Vec<pi_render::graph::node::NodeId>>,
                    pre_id: pi_render::graph::node::NodeId,
                    out_param: &O,
                ) -> bool {
                    let ty = std::any::TypeId::of::<Self>();
                    let r = out_param.can_fill(&mut None, ty.clone());
                    if r {
                        let v = map.entry(ty).or_insert(vec![]);
                        v.push(pre_id);
                    }
                    r
                }

                fn fill_from<O: pi_render::graph::param::OutParam + ?Sized>(
                    &mut self,
                    pre_id: pi_render::graph::node::NodeId,
                    out_param: &O,
                ) -> bool {
                    out_param.fill_to(pre_id, self, std::any::TypeId::of::<Self>())
                }
            }
        }
    };
    r.into()
}
