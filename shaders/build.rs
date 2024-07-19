use std::fs;
use serde::Deserialize;
use hassle_rs::{
  compile_hlsl,
  validate_dxil,
};

// Shader project.
#[derive(Debug, Deserialize, Default, Clone)]
struct ShaderProject {
  pub name: String,
  pub global_macros: Vec<String>,
  pub optional_macro_combinations: Vec<Vec<String>>,
}

// Shader make file.
#[derive(Debug, Deserialize, Default, Clone)]
struct ShaderMakeFile {
  pub projects: Vec<ShaderProject>,
}

fn main() {
  println!("cargo:rerun-if-changed=src");

  let profile = std::env::var("PROFILE").unwrap();
  let output_dir = if profile == "debug" { "output/debug" } else { "output/release" };

  if !std::path::Path::new("src/make_shaders.yaml").exists() {
    panic!("The make_shaders.yaml file is not found.");
  }

  let make_str = std::fs::read_to_string("src/make_shaders.yaml").expect("Failed to read src/make_shaders.yaml file.");
  let make_file: ShaderMakeFile = serde_yaml::from_str(&make_str).expect("Failed to parse src/make_shaders.yaml file.");

  for project in make_file.projects.iter() {
    if !project.optional_macro_combinations.is_empty() {
      for optional_macros in project.optional_macro_combinations.iter() {
        compile_shaders(&project.name, output_dir, &project.global_macros, optional_macros);
      }
    } else {
      compile_shaders(&project.name, output_dir, &project.global_macros, &Vec::new());
    }
  }
}

/// Compile shaders in the specified directory.
/// param shader_dir: The directory of the shaders.
/// param output_dir: The output directory of the compiled shaders.
/// param global_macros: The global macros.
/// param optional_macros: The optional macros.
fn compile_shaders(shader_dir: &str, output_dir: &str, global_macros: &Vec<String>, optional_macros: &Vec<String>) {
  let profile = std::env::var("PROFILE").unwrap();
  let output_dir = format!("{}/{}/{}", output_dir, shader_dir, optional_macros.join("#"));
  // println!("cargo:warning=Output directory: {}", output_dir);

  // Make output directory if it doesn't exist.
  fs::create_dir_all(&output_dir).unwrap();

  // Find all *.hlsl files in src directory.
  let mut hlsl_files = Vec::new();
  for entry in fs::read_dir(format!("src/{}", shader_dir)).unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();
    if path.is_file() && path.extension().unwrap() == "hlsl" {
      hlsl_files.push(path.clone());
    }
    if path.is_dir() {
      for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().unwrap() == "hlsl" {
          hlsl_files.push(path.clone());
        }
      }
    }
  }

  let mut options = Vec::<String>::new();
  options.push("-spirv".to_string());
  options.push("-fspv-target-env=vulkan1.3".to_string());
  options.push("-fspv-reduce-load-size".to_string());
  options.push("-fspv-extension=KHR".to_string());
  options.push("-fspv-extension=SPV_EXT_descriptor_indexing".to_string());
  options.push("-fspv-extension=SPV_KHR_float_controls".to_string());
  // options.push("-fspv-extension=SPV_EXT_shader_atomic_float_add".to_string()); // dxc doesn't support this extension.
  options.push("-fspv-extension=SPV_EXT_shader_image_int64".to_string());
  options.push("-WX".to_string());
  options.push("-Zpc".to_string());
  options.push("-I src/inc".to_string());
  if profile == "debug" {
    options.push("-Od".to_string());
    options.push("-Zi".to_string());
  } else {
    options.push("-O3".to_string());
  }

  let mut defines = Vec::<(String, Option<String>)>::new();
  for macro_name in global_macros.iter() {
    defines.push((macro_name.to_owned(), Some("1".to_string())));
  }
  for macro_name in optional_macros.iter() {
    defines.push((format!("USE_{}", macro_name), Some("1".to_string())));
  }

  // Compile all *.hlsl files into *.spv files.
  for hlsl_file in hlsl_files.iter() {
    // Get filename without extension.
    let hlsl_file_stem = hlsl_file.file_stem().unwrap().to_str().unwrap();
    // Get relative path of the hlsl file without filename.
    let hlsl_file_path = hlsl_file.parent().unwrap().strip_prefix(format!("src/{}", shader_dir)).unwrap();
    // Get string after the last dot in file_stem.
    let shader_kind = hlsl_file_stem.split('.').last().unwrap();

    // Skip unknown shader kinds.
    if !shader_kind.starts_with("vs") &&
       !shader_kind.starts_with("ps") &&
       !shader_kind.starts_with("cs") &&
       !shader_kind.starts_with("gs") &&
       !shader_kind.starts_with("hs") &&
       !shader_kind.starts_with("ds") &&
       !shader_kind.starts_with("lib") &&
       !shader_kind.starts_with("ms") &&
       !shader_kind.starts_with("as")
    {
      continue;
    }

    // Add options for specific shader kinds.
    let mut this_options = Vec::new();
    this_options.extend(options.iter().cloned());
    if shader_kind.starts_with("ms") || shader_kind.starts_with("as") {
      this_options.push("-fspv-extension=SPV_EXT_mesh_shader".to_string());
    }

    let ir = match compile_hlsl(
      hlsl_file.to_str().unwrap(),
      &fs::read_to_string(&hlsl_file).unwrap(),
      "main",
      shader_kind,
      // Convert this_options to &[&str].
      &this_options.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
      // Convert defines to &[(&str, Option<&str>)].
      &defines.iter().map(|(k, v)| (k.as_str(), v.as_ref().map(|v| v.as_str()))).collect::<Vec<_>>(),
    ) {
      Ok(ir) => ir,
      Err(err) => {
        println!("cargo:error=Failed to compile shader {}: {}", hlsl_file_stem, err);
        panic!();
      }
    };

    let result = validate_dxil(&ir);
    if let Some(err) = result.err() {
      println!("Validation shader {} failed: {}", hlsl_file_stem, err);
    }

    let output_dir = format!("{}/{}", &output_dir, hlsl_file_path.to_str().unwrap());
    // Make output directory if it doesn't exist.
    fs::create_dir_all(&output_dir).unwrap();

    // Save the binary result to a file.
    let mut file = fs::File::create(format!("{}/{}.spv", output_dir, hlsl_file_stem)).unwrap();
    std::io::Write::write_all(&mut file, &ir).unwrap();
  }
}
