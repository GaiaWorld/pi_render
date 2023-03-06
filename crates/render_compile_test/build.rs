use std::path::{Path, PathBuf};


use render_compile::{CompileShaderError, Parser};

fn main() -> Result<(), CompileShaderError> {

    // 除非修改build.rs， 否则不重新运行脚本
    println!("cargo:rerun-if-changed=build.rs");
    // visit_dirs("src/shader/", &mut |file| {
    //     if let Some(r) = file.extension() {
    //         let r = r.to_string_lossy();
    //         if r.ends_with("glsl") || r.ends_with("vert") || r.ends_with("frag") {
    //             println!("cargo:rerun-if-changed={:?}", file);
    //         }
    //     }
    // });

    let mut parser = Parser::default();

    let r = parser.push_gen_path(&["src/shader/"]).parse()?;

    for shader in r.shader_result.iter() {
        std::fs::write(&shader.0, &shader.1).unwrap();
		std::process::Command::new("rustfmt")
            .args([shader.0.clone()])
            .output()
            .expect("failed to execute process");
    }

    let mods = r.to_mod();
    for (dir, mods) in mods.iter() {
        std::fs::write(
            Path::new(dir).join("mod.rs"),
            mods.iter()
                .map(|r| "pub mod ".to_string() + r.as_str() + ";")
                .collect::<Vec<String>>()
                .join("\n"),
        )
        .unwrap();
    }
    Ok(())
}

fn visit_dirs<F: FnMut(&PathBuf), P: AsRef<Path>>(path: P, cb: &mut F) {
    let path = path.as_ref();
    if path.is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb);
                } else {
                    cb(&path);
                }
            }
        }
    }
    
}
