use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseBuffer, ParseStream},
    parse_macro_input,
    token::Comma,
    DeriveInput,
};

pub fn derive_bind_layout(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let mut layout = None;
    // let mut entry = None;
    for attr in ast.attrs.iter() {
        // pub pound_token: Token![#],
        // pub style: AttrStyle,
        // pub bracket_token: token::Bracket,
        // pub path: Path,
        // pub tokens: TokenStream,
        let t = attr.tokens.to_string();
        let t = TokenStream::from_str(t.as_str()).unwrap();
        if attr.path.segments[0].ident == "layout" {
            layout = Some(parse_macro_input!(t as Layout));
        }
    }
    let layout = match layout {
        Some(r) => r,
        _ => panic!("lost 'set' or 'binding'"),
    };

    if let (Some(set), Some(binding)) = (layout.set, layout.binding) {
        let count = match layout.count {
            Some(r) => quote! {std::num::NonZeroU32::new(#r)},
            None => quote! {None},
        };
        // let a = Kind::Depth;
        let gen = quote! {
            impl #impl_generics pi_render::rhi::shader::BindLayout for #name #ty_generics #where_clause {
                #[inline]
                fn set() -> u32 {
                    #set
                }

                #[inline]
                fn binding() -> u32 {
                    #binding
                }

                #[inline]
                fn count() -> Option<std::num::NonZeroU32> {
                    #count
                }
            }
        };
        gen.into()
    } else {
        panic!("lost 'set' or 'bind'")
    }
}

pub fn derive_input(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let mut location = None;
    // let mut entry = None;
    for attr in ast.attrs.iter() {
        // pub pound_token: Token![#],
        // pub style: AttrStyle,
        // pub bracket_token: token::Bracket,
        // pub path: Path,
        // pub tokens: TokenStream,
        let t = attr.tokens.to_string();
        let t = TokenStream::from_str(t.as_str()).unwrap();
        if attr.path.segments[0].ident == "location" {
            location = Some(parse_macro_input!(t as Input));
        }
    }
    let location = match location {
        Some(r) => r,
        _ => panic!("lost 'set' or 'binding'"),
    };

    if let Some(location) = location.location {
        // let a = Kind::Depth;
        let gen = quote! {
            impl #impl_generics pi_render::rhi::shader::Input for #name #ty_generics #where_clause {
                #[inline]
                fn location() -> u32 {
                    #location
                }
            }
        };
        gen.into()
    } else {
        panic!("lost 'set' or 'bind'")
    }
}

pub fn derive_binding_type(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let mut entry_ty = None;
    // let mut entry = None;
    for attr in ast.attrs.iter() {
        // pub pound_token: Token![#],
        // pub style: AttrStyle,
        // pub bracket_token: token::Bracket,
        // pub path: Path,
        // pub tokens: TokenStream,
        let t = attr.tokens.to_string();
        let t = TokenStream::from_str(t.as_str()).unwrap();
        if attr.path.segments[0].ident == "uniformbuffer" {
            entry_ty = Some(BindType::UniformBuffer);
        } else if attr.path.segments[0].ident == "storagebuffer" {
            entry_ty = Some(BindType::StorageBuffer);
        } else if attr.path.segments[0].ident == "texture" {
            entry_ty = Some(BindType::Tetxure(parse_macro_input!(t as TextureDesc)));
        } else if attr.path.segments[0].ident == "storagetexture" {
        } else if attr.path.segments[0].ident == "sampler" {
            entry_ty = Some(BindType::Sampler(parse_macro_input!(t as Sampler)));
        }
    }

    let entry_ty = match entry_ty {
        Some(r) => r,
        _ => panic!("lost 'set' or 'binding'"),
    };

    // let a = Kind::Depth;
    let gen = quote! {
        impl #impl_generics pi_render::rhi::shader::BindingType for #name #ty_generics #where_clause {
            #[inline]
            fn binding_type() -> wgpu::BindingType {
                #entry_ty
            }
        }
    };
    gen.into()
}

pub fn derive_uniform(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let mut layout = None;
    for attr in ast.attrs.iter() {
        // pub pound_token: Token![#],
        // pub style: AttrStyle,
        // pub bracket_token: token::Bracket,
        // pub path: Path,
        // pub tokens: TokenStream,
        let t = attr.tokens.to_string();
        let t = TokenStream::from_str(t.as_str()).unwrap();
        if attr.path.segments[0].ident == "uniform" {
            layout = Some(parse_macro_input!(t as UniformSpan));
        }
    }
    let layout = match layout {
        Some(r) => r,
        _ => panic!("lost 'offset' or 'len'"),
    };

    if let (Some(offset), Some(len), Some(bind)) = (&layout.offset, layout.len, &layout.bind) {
        let gen = quote! {
            impl #impl_generics pi_render::rhi::shader::WriteBuffer for #name #ty_generics #where_clause {
                fn write_into(&self, index: u32, buffer: &mut [u8]) {
                    unsafe { std::ptr::copy_nonoverlapping(
                        self.0.as_ptr() as usize as *const u8,
                        buffer.as_mut_ptr().add(index as usize + #offset),
                        #len,
                    ) };
                }
                #[inline]
                fn byte_len(&self) -> u32 {
                    #len
                }

                #[inline]
                fn offset(&self) -> u32 {
                    #offset
                }
            }
            impl #impl_generics pi_render::rhi::shader::Uniform for #name #ty_generics #where_clause {
                type Binding = #bind;
            }
        };
        gen.into()
    } else {
        panic!("lost 'offset' or 'len'")
    }
}

pub fn derive_buffer_size(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    for attr in ast.attrs.iter() {
        let t = attr.tokens.to_string();
        let t = TokenStream::from_str(t.as_str()).unwrap();
        if attr.path.segments[0].ident == "min_size" {
            let size = parse_macro_input!(t as BufferSize).0;
            let gen = quote! {
                impl #impl_generics pi_render::rhi::shader::BufferSize for #name #ty_generics #where_clause {
                    #[inline]
                    fn min_size() -> usize {
                        #size
                    }
                }
            };
            return gen.into();
        }
    }

    panic!("lost min_size");
}

#[derive(Default)]
struct Layout {
    set: Option<proc_macro2::TokenStream>,
    binding: Option<proc_macro2::TokenStream>,
    count: Option<proc_macro2::TokenStream>,
    // sampler: Option<proc_macro2::TokenStream>,
    // texture: Option<proc_macro2::TokenStream>,
}

enum BindType {
    UniformBuffer,
    StorageBuffer,
    Tetxure(TextureDesc),
    // StorageTetxure(TextureDesc),
    Sampler(Sampler),
}

impl ToTokens for BindType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            BindType::UniformBuffer => tokens.extend(quote! {wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true, // 默认为true， 可修改
                min_binding_size: wgpu::BufferSize::new(<Self as pi_render::rhi::shader::BufferSize>::min_size() as u64),
            }}),
            BindType::StorageBuffer => tokens.extend(quote! {wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage{true},
                has_dynamic_offset: true, // 默认为true， 可修改
                min_binding_size: wgpu::BufferSize::new(<Self as pi_render::rhi::shader::BufferSize>::min_size() as u64),
            }}),
            BindType::Tetxure(r) => r.to_tokens(tokens),
            BindType::Sampler(r) => r.to_tokens(tokens),
        };
    }
}

pub struct Sampler(pub Ident); // Filtering, Comparison, NonFiltering

impl ToTokens for Sampler {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let r = &self.0;
        let ty = match r.to_string().as_str() {
            "Filtering" => quote! {wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)},
            "Comparison" => {
                quote! {wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison)}
            }
            "NonFiltering" => {
                quote! {wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering)}
            }
            _ => quote! {wgpu::BindingType::Sampler(wgpu::SamplerBindingType::#r)},
        };
        tokens.extend(ty);
    }
}

impl Parse for Sampler {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let content;
        parenthesized!(content in input);

        let key = Ident::parse_any(&content)?;
        Ok(Self(key))
    }
}

#[derive(Default)]
struct TextureDesc {
    kind: Option<Ident>, // "Float, Uint, Sint, Depth"
    multi: bool,         // Storage {} // TODO
    dim: Option<Ident>,  // D1, D2, D2Array, Cube, CubeArray, D3,
}

impl ToTokens for TextureDesc {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let kind = match &self.kind {
            Some(r) => match r.to_string().as_str() {
                "Float" => quote! {wgpu::TextureSampleType::Float{filterable: true}},
                "Sint" => quote! {wgpu::TextureSampleType::Sint},
                "Uint" => quote! {wgpu::TextureSampleType::Uint},
                "Depth" => quote! {wgpu::TextureSampleType::Depth},
                _ => quote! {wgpu::TextureSampleType::Float{filterable: true}},
            },
            None => quote! {wgpu::TextureSampleType::Float{filterable: true}},
        };
        let (multi, dim) = (self.multi, &self.dim);
        tokens.extend(quote! {
                wgpu::BindingType::Texture {
                multisampled: #multi,
                sample_type: #kind,
                view_dimension: wgpu::TextureViewDimension::#dim,
            }
        });
    }
}

impl TextureDesc {
    fn set_value(&mut self, key: Ident, content: &ParseBuffer) -> Result<(), syn::Error> {
        if key.to_string() == "kind" {
            let v = Ident::parse_any(content)?;
            self.kind = Some(v);
        } else if key.to_string() == "multi" {
            let v: syn::LitBool = content.parse()?;
            self.multi = v.value;
        } else if key.to_string() == "dim" {
            let v = Ident::parse_any(content)?;
            self.dim = Some(v);
        }
        Ok(())
    }
}

impl Parse for TextureDesc {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut desc = TextureDesc::default();
        let content;
        parenthesized!(content in input);

        let mut content1;
        let key = Ident::parse_any(&content)?;
        parenthesized!(content1 in content);
        desc.set_value(key, &content1)?;

        content.parse::<Comma>()?;

        let key = Ident::parse_any(&content)?;
        parenthesized!(content1 in content);
        desc.set_value(key, &content1)?;

        content.parse::<Comma>()?;
        let key = Ident::parse_any(&content)?;
        parenthesized!(content1 in content);
        desc.set_value(key, &content1)?;

        Ok(desc)
    }
}

impl Parse for Layout {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut layout = Layout::default();
        let content;
        parenthesized!(content in input);

        let mut content1;
        let key = Ident::parse_any(&content)?;
        parenthesized!(content1 in content);
        let tokens: proc_macro2::TokenStream = content1.parse()?;
        set_layout(&mut layout, key, tokens);

        content.parse::<Comma>()?;

        let key = Ident::parse_any(&content)?;
        parenthesized!(content1 in content);
        let tokens: proc_macro2::TokenStream = content1.parse()?;
        set_layout(&mut layout, key, tokens);

        if content.parse::<Comma>().is_ok() {
            let key = Ident::parse_any(&content)?;
            parenthesized!(content1 in content);
            let tokens: proc_macro2::TokenStream = content1.parse()?;
            set_layout(&mut layout, key, tokens);
        }

        Ok(layout)
    }
}

#[derive(Default)]
struct UniformSpan {
    offset: Option<proc_macro2::TokenStream>,
    len: Option<proc_macro2::TokenStream>,
    bind: Option<proc_macro2::TokenStream>,
}

impl Parse for UniformSpan {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut uniform = UniformSpan::default();
        let content;
        parenthesized!(content in input);

        let mut content1;
        let key = Ident::parse_any(&content)?;
        parenthesized!(content1 in content);
        let tokens: proc_macro2::TokenStream = content1.parse()?;
        set_uniform(&mut uniform, key, tokens);

        content.parse::<Comma>()?;

        let key = Ident::parse_any(&content)?;
        parenthesized!(content1 in content);
        let tokens: proc_macro2::TokenStream = content1.parse()?;
        set_uniform(&mut uniform, key, tokens);

        content.parse::<Comma>()?;

        let key = Ident::parse_any(&content)?;
        parenthesized!(content1 in content);
        let tokens: proc_macro2::TokenStream = content1.parse()?;
        set_uniform(&mut uniform, key, tokens);
        Ok(uniform)
    }
}

struct BufferSize(proc_macro2::TokenStream);

impl Parse for BufferSize {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let content;
        parenthesized!(content in input);

        let tokens: proc_macro2::TokenStream = content.parse()?;
        Ok(Self(tokens))
    }
}

fn set_layout(layout: &mut Layout, key: Ident, tokens: proc_macro2::TokenStream) {
    if key.to_string() == "set" {
        layout.set = Some(tokens);
    } else if key.to_string() == "binding" {
        layout.binding = Some(tokens);
    } else if key.to_string() == "count" {
        layout.count = Some(tokens);
    }
}

fn set_uniform(layout: &mut UniformSpan, key: Ident, tokens: proc_macro2::TokenStream) {
    if key.to_string() == "offset" {
        layout.offset = Some(tokens);
    } else if key.to_string() == "len" {
        layout.len = Some(tokens);
    } else if key.to_string() == "bind" {
        layout.bind = Some(tokens);
    }
}

#[derive(Default)]
struct Input {
    location: Option<proc_macro2::TokenStream>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let content;
        parenthesized!(content in input);

        let tokens: proc_macro2::TokenStream = content.parse()?;

        Ok(Input {location: Some(tokens)})
    }
}