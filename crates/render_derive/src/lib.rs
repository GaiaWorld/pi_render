mod enum_variant_meta;
mod modules;

use std::collections::VecDeque;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use syn::{DeriveInput, Fields, DataStruct, Data, Field, Type};
use quote::{quote};

#[proc_macro_derive(EnumVariantMeta)]
pub fn derive_enum_variant_meta(input: TokenStream) -> TokenStream {
    enum_variant_meta::derive_enum_variant_meta(input)
}

#[proc_macro_derive(Param, attributes(unpack))]
pub fn derive_param(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let unpack = ast
        .attrs
        .iter()
        .find(|attr| attr.path.segments[0].ident == "unpack")
        .map(|attr| {true});
	// 如果没有指定存储容器，没有必要重载实现
	let unpack = match unpack {
		Some(r) => r,
		None => false
	};

	let r = if unpack {
		let named_fields = match &ast.data {
			Data::Struct(DataStruct {
				fields: Fields::Named(fields),
				..
			}) => &fields.named,
			_ => panic!("Expected a struct with named fields."),
		};
		let fileds =  named_fields
        .iter()
		.filter(|field| {if let syn::Visibility::Public(_) = &field.vis {true} else {false}})
		.collect::<Vec<&Field>>();

		let mut i = 0;
        let fills = fileds
        .iter()
		.map(|field| {
			let name = field.ident.as_ref().unwrap();
			let ty = &field.ty;
			i += 1;
			if i == 1 {
				quote! {if ty == std::any::TypeId::of::<#ty>() { self.#name = std::ptr::read(src_ptr as *const #ty)}}
			} else {
				quote! {else if ty == std::any::TypeId::of::<#ty>() { self.#name = std::ptr::read(src_ptr as *const #ty)}}
			}
			
		})
		.collect::<Vec<TokenStream2>>();

		let mut tys =  fileds
        .iter()
		.map(|field| {&field.ty})
		.collect::<Vec<&Type>>();
		if tys.len() == 0 {
			panic!("pub filed count is 0");
		}
		let last_ty = tys.pop().unwrap();

		let fill_to = fileds
        .iter()
		.map(|field| {
			let name = field.ident.as_ref().unwrap();
			let ty = &field.ty;
			quote! {unsafe { target.fill(src_id, &self.#name as *const #ty as usize, std::any::TypeId::of::<#ty>()) }}
		})
		.collect::<Vec<TokenStream2>>();

		// let mut fileds = Vec::new();
		quote! {
			impl #impl_generics pi_render::graph_new::FillTarget for #name #ty_generics #where_clause {
				unsafe fn fill(&mut self, src_id: pi_render::graph_new::NodeId, src_ptr: usize, ty: std::any::TypeId) {
					#(#fills)*
				}
			
				fn check_macth(&self, ty: std::any::TypeId) -> Result<(), pi_render::graph_new::RenderGraphError> {
					if #(ty == std::any::TypeId::of::<#tys>() ||)* ty == std::any::TypeId::of::<#last_ty>() {
						Ok(())
					} else {
						Err(pi_render::graph_new::RenderGraphError::MismatchedParam)
					}
				}
			}
			
			
			impl #impl_generics pi_render::graph_new::FillSrc for #name #ty_generics #where_clause {
				fn fill_to<T: pi_render::graph_new::FillTarget + ?Sized>(self, src_id: pi_render::graph_new::NodeId, target: &mut T) {
					#(#fill_to;)*
					// 忘记self
					std::mem::forget(self);
				}
			
				fn check_macths<T: pi_render::graph_new::FillTarget + ?Sized>(target: &T) -> Result<(), pi_render::graph_new::RenderGraphError> {
					#(target.check_macth(std::any::TypeId::of::<#tys>())?;)*
					target.check_macth(std::any::TypeId::of::<#last_ty>())
				}
			}

			// impl #impl_generics pi_render::graph_new::Param for #name #ty_generics #where_clause {}
		}
	} else {
		quote! {
			impl #impl_generics pi_render::graph_new::FillTarget for #name #ty_generics #where_clause {
				unsafe fn fill(&mut self, src_id: pi_render::graph_new::NodeId, src_ptr: usize, ty: std::any::TypeId) {
					if ty == std::any::TypeId::of::<#name>() {
						*self = std::ptr::read(src_ptr as *const #name #ty_generics);
					}
				}
			
				fn check_macth(&self, ty: std::any::TypeId) -> Result<(), pi_render::graph_new::RenderGraphError> {
					if ty == std::any::TypeId::of::<#name>() {
						return Ok(());
					}
					Err(pi_render::graph_new::RenderGraphError::MismatchedParam)
				}
			}
			
			
			impl #impl_generics pi_render::graph_new::FillSrc for #name #ty_generics #where_clause {
				fn fill_to<T: pi_render::graph_new::FillTarget + ?Sized>(self, src_id: pi_render::graph_new::NodeId, target: &mut T) {
					unsafe { target.fill(src_id, &self as *const #name #ty_generics as usize, std::any::TypeId::of::<Self>()) };
					// 忘记self
					std::mem::forget(self);
				}
			
				fn check_macths<T: pi_render::graph_new::FillTarget + ?Sized>(target: &T) -> Result<(), pi_render::graph_new::RenderGraphError> {
					target.check_macth(std::any::TypeId::of::<Self>())
				}
			}

			impl #impl_generics pi_render::graph_new::Param for #name #ty_generics #where_clause {}
		}
	};
	r.into()

    
}