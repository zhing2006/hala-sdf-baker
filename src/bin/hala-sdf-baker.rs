use std::rc::Rc;

use anyhow::{
  Result,
  Context,
};

use clap::{arg, Command};

use hala_imgui::{
  HalaApplication,
  HalaImGui,
};

use hala_renderer::{
  renderer::HalaRendererTrait,
  rz_renderer::HalaRenderer,
  scene,
};

use hala_sdf_baker::config;

/// The SDF baker application.
struct SDFBakerApplication {
  log_file: String,
  config: config::AppConfig,
  renderer: Option<HalaRenderer>,
  imgui: Option<HalaImGui>,

  color_render_target: Option<hala_gfx::HalaImage>,
  depth_render_target: Option<hala_gfx::HalaImage>,
}

impl Drop for SDFBakerApplication {
  fn drop(&mut self) {
    self.color_render_target = None;
    self.depth_render_target = None;
    self.imgui = None;
    self.renderer = None;
  }
}

/// The implementation of the SDF baker application.
impl SDFBakerApplication {

  pub fn new() -> Result<Self> {
    // Parse the command line arguments.
    let matches = cli().get_matches();
    let log_file = match matches.get_one::<String>("log") {
      Some(log_file) => log_file,
      None => "./logs/sdf_baker.log"
    };
    let config_file = matches.get_one::<String>("config").with_context(|| "Failed to get the config file path.")?;

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

      color_render_target: None,
      depth_render_target: None,
    })
  }

}

/// The implementation of the application trait for the SDF baker application.
impl HalaApplication for SDFBakerApplication {

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
    "SDF Baker"
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

     // Setup features.
    let mut features = vec!["SDF_BAKER"];
    features.push("CONSERVATIVE_RASTERIZATION");

    // Setup the renderer.
    let gpu_req = hala_gfx::HalaGPURequirements {
      width: self.config.window.width as u32,
      height: self.config.window.height as u32,
      version: (1, 3, 0),
      require_srgb_surface: true,
      require_mesh_shader: false,
      require_ray_tracing: false,
      require_10bits_output: false,
      is_low_latency: true,
      require_depth: true,
      require_printf_in_shader: false,
      ..Default::default()
    };

    let mut renderer = HalaRenderer::new(
      "SDF Baker",
      &gpu_req,
      window,
    )?;

    {
      let context = renderer.resources().context.borrow();

      let render_target = hala_gfx::HalaImage::new_2d(
        Rc::clone(&context.logical_device),
        hala_gfx::HalaImageUsageFlags::COLOR_ATTACHMENT | hala_gfx::HalaImageUsageFlags::SAMPLED,
        hala_gfx::HalaFormat::R16G16B16A16_SFLOAT,
        self.config.window.width as u32,
        self.config.window.height as u32,
        1,
        1,
        hala_gfx::HalaMemoryLocation::GpuOnly,
        "custom_color.render_target",
      )?;
      self.color_render_target = Some(render_target);

      let render_target = hala_gfx::HalaImage::new_2d(
        Rc::clone(&context.logical_device),
        hala_gfx::HalaImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | hala_gfx::HalaImageUsageFlags::SAMPLED,
        hala_gfx::HalaFormat::D32_SFLOAT,
        self.config.window.width as u32,
        self.config.window.height as u32,
        1,
        1,
        hala_gfx::HalaMemoryLocation::GpuOnly,
        "custom_depth.render_target",
      )?;
      self.depth_render_target = Some(render_target);
    }

    let shaders_dir = if cfg!(debug_assertions) {
      format!("shaders/output/debug/hala-sdf-baker/{}", features.join("#"))
    } else {
      format!("shaders/output/release/hala-sdf-baker/{}", features.join("#"))
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
      std::rc::Rc::clone(&(*renderer.resources().context)),
      false,
    )?);

    self.renderer = Some(renderer);

    Ok(())
  }

  /// The after run function.
  fn after_run(&mut self) {
    if let Some(renderer) = &mut self.renderer.take() {
      renderer.wait_idle().expect("Failed to wait the renderer idle.");
      self.depth_render_target = None;
      self.color_render_target = None;
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
          ui.window("SDF Baker")
            .position([10.0, 10.0], imgui::Condition::FirstUseEver)
            .build(|| {
              if ui.button_with_size("Save", [100.0, 30.0]) {
                log::info!("Save button clicked.");
              }
            }
          );
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

/// The command line interface.
fn cli() -> Command {
  Command::new("sdf-baker")
    .about("The SDF Baker.")
    .arg_required_else_help(true)
    .arg(arg!(-l --log <LOG_FILE> "The file path of the log file. Default is ./logs/sdf_baker.log."))
    .arg(arg!(-c --config [CONFIG_FILE] "The file path of the config file."))
}

/// The normal main function.
fn main() -> Result<()> {
  // Initialize the application.
  let mut app = SDFBakerApplication::new()?;
  app.init()?;

  // Run the application.
  app.run()?;

  Ok(())
}