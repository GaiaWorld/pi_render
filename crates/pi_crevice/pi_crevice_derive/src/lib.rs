mod glsl;
mod layout;

use pi_macro_utils::PiManifest;
use proc_macro::TokenStream as CompilerTokenStream;

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

const PI: &str = "pi_render";
const PI_CREVICE: &str = "pi_crevice";
const PI_RENDER: &str = "pi_render";

fn pi_crevice_path() -> Path {
    let pi_manifest = PiManifest::default();
    pi_manifest
        .maybe_get_path(crate::PI)
        .map(|pi_path| {
            let mut segments = pi_path.segments;
            segments.push(PiManifest::parse_str("render"));
            Path {
                leading_colon: None,
                segments,
            }
        })
        .or_else(|| pi_manifest.maybe_get_path(crate::PI_RENDER))
        .map(|pi_render_path| {
            let mut segments = pi_render_path.segments;
            segments.push(PiManifest::parse_str("pi_resource"));
            Path {
                leading_colon: None,
                segments,
            }
        })
        .unwrap_or_else(|| pi_manifest.get_path(crate::PI_CREVICE))
}
