use hala_renderer::scene;

/// The SDF baker settings.
#[derive(Debug, Clone, Copy)]
pub struct SDFBakerSettings{
  pub is_sdf: bool, // Whether the baker is SDF or UDF.

  // Common settings.
  pub show_desired_box: bool,
  pub show_actual_box: bool,
  pub show_wireframe: bool,
  pub show_sdf: bool,

  // SDF settings.
  pub show_render_targets: bool,
  pub show_ray_map: bool,

  // UDF settings.
  // pub show_udf: bool, // Use the show_sdf instead.

  // Common settings.
  pub selected_mesh_index: i32,
  pub max_resolution: i32,
  pub surface_offset: f32,
  pub center: [f32; 3],
  pub desired_size: [f32; 3],
  pub actual_size: [f32; 3],
  pub padding: [f32; 3],

  // SDF settings.
  pub sign_passes_count: i32,
  pub in_out_threshold: f32,

  // UDF settings.
}

impl Default for SDFBakerSettings {
  fn default() -> Self {
    Self {
      is_sdf: true,

      show_desired_box: true,
      show_actual_box: true,
      show_wireframe: true,
      show_sdf: true,

      show_render_targets: false,
      show_ray_map: false,

      selected_mesh_index: 0,
      max_resolution: 64,
      surface_offset: 0.0,
      center: [0.0, 0.0, 0.0],
      desired_size: [1.0, 1.0, 1.0],
      actual_size: [1.0, 1.0, 1.0],
      padding: [1.0, 1.0, 1.0],

      sign_passes_count: 1,
      in_out_threshold: 0.5,
    }
  }
}

impl SDFBakerSettings {
  pub fn get_bounds(&self) -> scene::HalaBounds {
    scene::HalaBounds {
      center: self.center,
      extents: [
        self.actual_size[0] / 2.0,
        self.actual_size[1] / 2.0,
        self.actual_size[2] / 2.0,
      ],
    }
  }
}
