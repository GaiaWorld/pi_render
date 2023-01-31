#[macro_use]
extern crate lazy_static;

// pub mod shaders;
pub mod test_shader;


use cargo_manifest::{DepsSet, Manifest};
use pi_atom::Atom;
use pi_hash::XHashSet;
use std::{env, path::PathBuf};

// #[test]
// fn test11() {
//     let meta = shaders::test::ProgramMeta::create_meta();

//     let mut vs_defines = XHashSet::default();
//     vs_defines.extend(
//         vec![
//             Atom::from("AAA"),
//             Atom::from("BBB"),
//             Atom::from("CCC"),
//             Atom::from("DD"),
//             Atom::from("EEE"),
//             Atom::from("FFF"),
//             Atom::from("GGG"),
//             Atom::from("FFF"),
//         ]
//         .into_iter(),
//     );

//     std::fs::write("output.vert", meta.to_code(&vs_defines, wgpu::ShaderStages::VERTEX));
//     std::fs::write("output.frag", meta.to_code(&vs_defines, wgpu::ShaderStages::FRAGMENT));
// }

pub struct RenderManifest {
    manifest: Manifest,
}

impl Default for RenderManifest {
    fn default() -> Self {
        Self {
            manifest: env::var_os("CARGO_MANIFEST_DIR")
                .map(PathBuf::from)
                .map(|mut path| {
                    path.push("Cargo.toml");
                    Manifest::from_path(path).unwrap()
                })
                .unwrap(),
        }
    }
}

const PI: &'static str = "pi_render";

impl RenderManifest {
    pub fn maybe_get_path<'a>(&'a self, name: &'a str) -> Option<&'a str> {
        let find_in_deps = |deps: &'a DepsSet| -> Option<&'a str> {
            let package = if let Some(dep) = deps.get(name) {
                return Some(dep.package().unwrap_or(name));
            } else if let Some(dep) = deps.get(PI) {
                dep.package().unwrap_or(PI)
            } else {
                return None;
            };
            None

            // if let Some(module) = name.strip_prefix("render_") {
            //     path.segments.push(Self::parse_str(module));
            // }
            // Some(path)
        };

        let deps = self.manifest.dependencies.as_ref();
        let deps_dev = self.manifest.dev_dependencies.as_ref();
        println!("deps: {:?}", deps);

        deps.and_then(find_in_deps).or_else(|| deps_dev.and_then(find_in_deps))
    }
    pub fn get_path<'a>(&'a self, name: &'a str) -> Option<&'a str> { self.maybe_get_path(name) }

    // pub fn parse_str<T: syn::parse::Parse>(path: &str) -> T { syn::parse(path.parse::<TokenStream>().unwrap()).unwrap() }
}

fn main() {
    let d = RenderManifest::default();
    let r = d.get_path("cargo_metadat");
    println!("r=============={:?}", r);
    // let manifest = std::env::var_os("CARGO_MANIFEST_DIR")
    //     .map(PathBuf::from)
    //     .map(|mut path| {
    //         path.push("Cargo.toml");
    //         Manifest::from_path(path).unwrap()
    //     })
    //     .unwrap();
    // println!("main fast ======{:?}", manifest);
}
