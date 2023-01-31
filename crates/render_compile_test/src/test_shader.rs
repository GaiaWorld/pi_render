use std::path::Path;

use render_compile::{CompileShaderError, Parser, ProgramDesc};

#[test]
fn test() -> Result<(), CompileShaderError> {
    std::env::set_current_dir("E:\\pi_render_master\\crates\\render_compile_test");
    // println!("cargo:rerun-if-changed=src/shaders/");
    let mut parser = Parser::default();
    std::fs::write("build_error.text", "zzzzzzz").unwrap();

    let r = parser
        .push_gen_path(&["src/shaders/"])
        .push_program(vec![ProgramDesc::new("src/shaders/test.vert", "src/shaders/test.frag", "src/shaders/aa")])
        .parse()?;
    for shader in r.shader_result.iter() {
        std::fs::write(&shader.0, &shader.1).unwrap();
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
        .unwrap()
    }
    Ok(())
    // for item in sharder_infos.iter() {
    // 	render_compile::compile_and_out(
    // 		item[0],
    // 		render_compile::ProcessedShader::Glsl(Cow::Borrowed(read_to_string(item[1]).unwrap().as_str()), naga::ShaderStage::Vertex),
    // 		render_compile::ProcessedShader::Glsl(Cow::Borrowed(read_to_string(item[2]).unwrap().as_str()), naga::ShaderStage::Fragment),
    // 		&target
    // 	);
    // }

    // let mods: Vec<String> = sharder_infos.iter().map(|r| {format!("pub mod {};", r[0])}).collect();
    // std::fs::write(target.join("mod.rs"), mods.join("\n")).unwrap();
}

// use render_derive::{BindLayout, BufferSize, Uniform};

// #[derive(BindLayout, BufferSize)]
// #[layout(set(2), binding(3))]
// #[min_size(10)]
// pub struct CameraMatrixBind;

// #[derive(Uniform)]
// #[span(offset(0), len(4))]
// pub struct CameraMatrixUniform<'a>(&'a [f32]);

// #[test]
// fn test() {
//     use pi_render::rhi::shader::{BindLayout, BufferSize, Uniform};
//     println!(
//         "==============={}, {}, {}",
//         CameraMatrixBind::set(),
//         CameraMatrixBind::binding(),
//         CameraMatrixBind::min_size()
//     );

//     let mut bb = Vec::with_capacity(10);
//     println!("==============={:?}", bb);
//     unsafe { bb.set_len(4) };
//     CameraMatrixUniform(&[1.0]).write_into(0, &mut bb);

//     println!("==============={:?}", bb);
// }
