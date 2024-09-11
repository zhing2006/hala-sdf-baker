use anyhow::{
  Result,
  Context,
};

use hala_imgui::{
  HalaApplicationContextTrait,
  HalaApplication,
  HalaImGui,
};

use hala_renderer::{
  renderer::HalaRendererTrait,
  rz_renderer::HalaRenderer,
  scene,
};

use hala_sdf_baker::config;

/// The rasterization renderer application context.
struct RasterizationRendererApplicationContext {
  log_file: String,
  config: config::AppConfig,
  renderer: Option<HalaRenderer>,
  imgui: Option<HalaImGui>,
}

impl Drop for RasterizationRendererApplicationContext {
  fn drop(&mut self) {
    self.imgui = None;
    self.renderer = None;
  }
}

/// The implementation of the rasterization renderer application context.
impl RasterizationRendererApplicationContext {

  pub fn new() -> Result<Self> {
    let log_file = "./logs/test.log";
    let config_file = "./conf/config.yaml";

    // Load the configure.
    let config = config::load_app_config(config_file)?;
    log::debug!("Config: {:?}", config);
    config::validate_app_config(&config)?;

    // Create out directory.
    std::fs::create_dir_all("./out")
      .with_context(|| "Failed to create the output directory: ./out")?;

    Ok(Self {
      log_file: log_file.to_string(),
      config,
      renderer: None,
      imgui: None,
    })
  }

}

/// The implementation of the application context trait for the rasterization renderer application context.
impl HalaApplicationContextTrait for RasterizationRendererApplicationContext {

  fn get_log_console_fmt(&self) -> &str {
    "{d(%H:%M:%S)} {h({l:<5})} {t:<20.20} - {m}{n}"
  }
  fn get_log_file_fmt(&self) -> &str {
    "{d(%Y-%m-%d %H:%M:%S)} {h({l:<5})} {f}:{L} - {m}{n}"
  }
  fn get_log_file(&self) -> &std::path::Path {
    std::path::Path::new(self.log_file.as_str())
  }
  fn get_log_file_size(&self) -> u64 {
    1024 * 1024 /* 1MB */
  }
  fn get_log_file_roller_count(&self) -> u32 {
    5
  }

  fn get_window_title(&self) -> &str {
    "Rasterization Renderer"
  }
  fn get_window_size(&self) -> winit::dpi::PhysicalSize<u32> {
    winit::dpi::PhysicalSize::new(self.config.window.width as u32, self.config.window.height as u32)
  }

  fn get_imgui(&self) -> Option<&HalaImGui> {
    self.imgui.as_ref()
  }
  fn get_imgui_mut(&mut self) -> Option<&mut HalaImGui> {
    self.imgui.as_mut()
  }

  /// The before run function.
  /// param width: The width of the window.
  /// param height: The height of the window.
  /// param window: The window.
  /// return: The result.
  fn before_run(&mut self, _width: u32, _height: u32, window: &winit::window::Window) -> Result<()> {
    let now = std::time::Instant::now();
    let mut scene = scene::cpu::HalaScene::new(&self.config.scene_file)?;
    log::info!("Load scene used {}ms.", now.elapsed().as_millis());

    // Setup the renderer.
    let gpu_req = hala_gfx::HalaGPURequirements {
      width: self.config.window.width as u32,
      height: self.config.window.height as u32,
      version: (1, 3, 0),
      require_srgb_surface: true,
      require_mesh_shader: true,
      require_ray_tracing: false,
      require_10bits_output: false,
      is_low_latency: true,
      require_depth: true,
      require_printf_in_shader: cfg!(debug_assertions),
      ..Default::default()
    };

    let mut renderer = HalaRenderer::new(
      "Rasterization Renderer",
      &gpu_req,
      window,
    )?;

    let shaders_dir = if cfg!(debug_assertions) {
      "shaders/output/debug/test"
    } else {
      "shaders/output/release/test"
    };
    if gpu_req.require_mesh_shader {
      renderer.push_shaders_with_file(
        Some(&format!("{}/test.as_6_8.spv", shaders_dir)),
        &format!("{}/test.ms_6_8.spv", shaders_dir),
        &format!("{}/test.ps_6_8.spv", shaders_dir),
        "test",
      )?;
    } else {
      renderer.push_traditional_shaders_with_file(
        &format!("{}/test.vs_6_8.spv", shaders_dir),
        &format!("{}/test.ps_6_8.spv", shaders_dir),
        "test",
      )?;
    }

    renderer.set_scene(&mut scene)?;

    renderer.commit()?;

    self.imgui = Some(HalaImGui::new(
      std::rc::Rc::clone(&renderer.resources().context),
      false,
    )?);

    self.renderer = Some(renderer);

    Ok(())
  }

  /// The after run function.
  fn after_run(&mut self) {
    if let Some(renderer) = &mut self.renderer.take() {
      renderer.wait_idle().expect("Failed to wait the renderer idle.");
      self.imgui = None;
    }
  }

  /// The update function.
  /// param delta_time: The delta time.
  /// return: The result.
  fn update(&mut self, delta_time: f64, width: u32, height: u32) -> Result<()> {
    if let Some(imgui) = self.imgui.as_mut() {
      imgui.begin_frame(
        delta_time,
        width,
        height,
        |ui| {
          ui.window("Rasterization Renderer Test")
            .position([10.0, 10.0], imgui::Condition::FirstUseEver)
            .build(|| {
              ui.text("Hello, world!");
              ui.text("This is a test for the rasterization renderer.");
            }
          );

          Ok(())
        }
      )?;
      imgui.end_frame()?;
    }

    if let Some(renderer) = &mut self.renderer {
      renderer.update(
        delta_time,
        width,
        height,
        |index, command_buffers| {
          if let Some(imgui) = self.imgui.as_mut() {
            imgui.draw(index, command_buffers)?;
          }

          Ok(())
        }
      )?;
    }

    Ok(())
  }

  /// The render function.
  /// return: The result.
  fn render(&mut self) -> Result<()> {
    if let Some(renderer) = &mut self.renderer {
      renderer.render()?;
    }

    Ok(())
  }

}

fn main() -> Result<()> {
  // Initialize the application.
  let context = RasterizationRendererApplicationContext::new()?;
  context.init()?;

  // Run the application.
  let mut app = HalaApplication::new(Box::new(context));
  app.run()?;

  Ok(())
}
