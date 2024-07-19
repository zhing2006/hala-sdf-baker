use hala_renderer::scene;

/// The SDF baker settings.
#[derive(Debug, Clone, Copy)]
pub struct SDFBakerSettings{
  pub show_desired_box: bool,
  pub show_actual_box: bool,
  pub show_wireframe: bool,
  pub show_render_targets: bool,
  pub show_ray_map: bool,
  pub show_sdf: bool,

  pub selected_mesh_index: i32,
  pub sign_passes_count: i32,
  pub in_out_threshold: f32,
  pub surface_offset: f32,
  pub max_resolution: i32,
  pub center: [f32; 3],
  pub desired_size: [f32; 3],
  pub actual_size: [f32; 3],
  pub padding: [i32; 3],
}

impl Default for SDFBakerSettings {
  fn default() -> Self {
    Self {
      show_desired_box: true,
      show_actual_box: true,
      show_wireframe: true,
      show_render_targets: false,
      show_ray_map: false,
      show_sdf: true,

      selected_mesh_index: 0,
      sign_passes_count: 1,
      in_out_threshold: 0.5,
      surface_offset: 0.0,
      max_resolution: 64,
      center: [0.0, 0.0, 0.0],
      desired_size: [1.0, 1.0, 1.0],
      actual_size: [1.0, 1.0, 1.0],
      padding: [1, 1, 1],
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
