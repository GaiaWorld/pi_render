mod glsl;
mod layout;

use proc_macro::TokenStream as CompilerTokenStream;
use render_macro_utils::RenderManifest;

use syn::{parse_macro_input, DeriveInput, Path};

#[proc_macro_derive(AsStd140)]
pub fn derive_as_std140(input: CompilerTokenStream) -> CompilerTokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = layout::emit(input, "Std140", "std140", 16);

    CompilerTokenStream::from(expanded)
}

#[proc_macro_derive(AsStd430)]
pub fn derive_as_std430(input: CompilerTokenStream) -> CompilerTokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = layout::emit(input, "Std430", "std430", 0);

    CompilerTokenStream::from(expanded)
}

#[proc_macro_derive(GlslStruct)]
pub fn derive_glsl_struct(input: CompilerTokenStream) -> CompilerTokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = glsl::emit(input);

    CompilerTokenStream::from(expanded)
}

const PI_RENDER: &str = "pi_render";
const RENDER_CREVICE: &str = "render_crevice";
const RENDER_CORE: &str = "render_core";

fn render_crevice_path() -> Path {
    let render_manifest = RenderManifest::default();
    render_manifest
        .maybe_get_path(crate::PI_RENDER)
        .map(|render_path| {
            let mut segments = render_path.segments;
            segments.push(RenderManifest::parse_str("render"));
            Path {
                leading_colon: None,
                segments,
            }
        })
        .or_else(|| render_manifest.maybe_get_path(crate::RENDER_CORE))
        .map(|render_path| {
            let mut segments = render_path.segments;
            // TODO
            segments.push(RenderManifest::parse_str("pi_resource"));
            Path {
                leading_colon: None,
                segments,
            }
        })
        .unwrap_or_else(|| render_manifest.get_path(crate::RENDER_CREVICE))
}
