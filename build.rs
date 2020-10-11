use anyhow::*;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
const SHADER_EXTENSIONS: &[&str] = &["glsl", "vert", "frag", "comp"];

fn out_dir() -> String {
    std::env::var("OUT_DIR").unwrap()
}

fn find_shaders<P: AsRef<Path>>(path: P) -> io::Result<Vec<PathBuf>> {
    let mut shaders = vec![];
    let mut entries = std::fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        if entry.metadata()?.is_dir() {
            shaders.extend(find_shaders(&entry.path())?);
        } else {
            if let Some(ext) = entry.path().extension() {
                for extension in SHADER_EXTENSIONS {
                    if ext == *extension {
                        shaders.push(entry.path());
                    }
                }
            }
        }
    }
    Ok(shaders)
}

fn process_shader<P: AsRef<Path>>(path: P, compiler: &mut shaderc::Compiler) -> Result<()> {
    let path = path.as_ref();

    let kind = match path
        .extension()
        .unwrap()
        .to_str()
        .ok_or(anyhow!("path is not utf8"))?
    {
        "vert" => shaderc::ShaderKind::Vertex,
        "frag" => shaderc::ShaderKind::Fragment,
        "comp" => shaderc::ShaderKind::Compute,
        _ => shaderc::ShaderKind::InferFromSource,
    };

    let spirv = compiler
        .compile_into_spirv(
            &fs::read_to_string(path)?,
            kind,
            path.to_str().ok_or(anyhow!("path is not utf8"))?,
            "main",
            None,
        )
        .context("Compiling Spirv")?;

    let out_path = format!(
        "{}/{}.spv",
        &out_dir(),
        path.file_name()
            .ok_or(anyhow!("path is not a file: {:?}", path))?
            .to_str()
            .ok_or(anyhow!("path is not utf8"))?
    );
    let out_path = Path::new(&out_path);
    fs::create_dir_all(out_path.parent().unwrap())
        .context(format!("creating parent directories of {:?}", out_path))?;
    let mut file = fs::File::create(&out_path).context(format!("Opening {:?}", out_path))?;
    file.write_all(spirv.as_binary_u8())
        .context("writing spirv to file")?;
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=shaders/",);
    let mut compiler = shaderc::Compiler::new().unwrap();
    for shader in find_shaders("shaders/").unwrap() {
        process_shader(shader, &mut compiler).unwrap();
    }
}
