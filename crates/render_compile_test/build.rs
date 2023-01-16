use std::path::Path;

use render_compile::Parser;

fn main() {
    // // 除非修改build.rs， 否则不重新运行脚本
    // println!("cargo:rerun-if-changed=build.rs");

    // let mut parser = Parser::default();
    // std::fs::write("build_error.text", "no error").unwrap();

    // let r = parser.push_gen_path(&["src/shaders/"]).parse();

    // match r {
    //     Ok(r) => {
    //         for shader in r.shader_result.iter() {
    //             std::fs::write(&shader.0, &shader.1).unwrap();
    //         }

    //         let mods = r.to_mod();
    //         for (dir, mods) in mods.iter() {
    //             std::fs::write(
    //                 Path::new(dir).join("mod.rs"),
    //                 mods.iter()
    //                     .map(|r| "pub mod ".to_string() + r.as_str() + ";")
    //                     .collect::<Vec<String>>()
    //                     .join("\n"),
    //             )
    //             .unwrap()
    //         }
    //     }

    //     Err(e) => {
    //         std::fs::write("build_error.text", e.to_string()).unwrap();
    //     }
    // }
}
