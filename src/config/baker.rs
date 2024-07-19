use std::path::Path;
use std::collections::HashMap;

use serde::Deserialize;

use anyhow::{Result, Context};

use hala_renderer::prelude::*;

/// The baker configure.
#[derive(Deserialize)]
pub struct BakerConfig {
  pub compute_programs: HashMap<String, HalaComputeProgramDesc>,
  pub graphics_programs: HashMap<String, HalaGraphicsProgramDesc>,
}

/// The baker configure implementation.
impl BakerConfig {

  /// Load the baker configure.
  /// param: config_file: the configure file path.
  /// return: the application configure.
  pub fn load<P: AsRef<Path>>(config_path: P) -> Result<Self> {
    let path = config_path.as_ref();
    let config_str = std::fs::read_to_string(path)
      .with_context(|| format!("Failed to read the config file: {:?}", path))?;
    let config: Self = serde_yaml::from_str(&config_str)
      .with_context(|| format!("Failed to parse the config file: {:?}", path))?;
    Ok(config)
  }

}