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
  scene,
};

use hala_sdf_baker::{
  config,
  baker::SDFBaker,
};

/// The SDF baker application.
struct SDFBakerApplication {
  log_file: String,
  output_file: String,
  config: config::AppConfig,
  baker: Option<SDFBaker>,
  imgui: Option<HalaImGui>,
}

impl Drop for SDFBakerApplication {
  fn drop(&mut self) {
    self.imgui = None;
    self.baker = None;
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
    let output_file = matches.get_one::<String>("output").with_context(|| "Failed to get the output file path.")?;

    // Load the configure.
    let config = config::load_app_config(config_file)?;
    log::debug!("Config: {:?}", config);
    config::validate_app_config(&config)?;

    // Create out directory.
    std::fs::create_dir_all("./out")
      .with_context(|| "Failed to create the output directory: ./out")?;

    Ok(Self {
      log_file: log_file.to_string(),
      output_file: output_file.to_string(),
      config,
      baker: None,
      imgui: None,
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
      require_printf_in_shader: cfg!(debug_assertions),
      ..Default::default()
    };

    let mut baker = SDFBaker::new(
      "SDF Baker",
      &gpu_req,
      window,
    )?;

    baker.set_scene(&mut scene)?;

    baker.commit()?;

    self.imgui = Some(HalaImGui::new(
      std::rc::Rc::clone(&(*baker.resources().context)),
      false,
    )?);

    self.baker = Some(baker);

    Ok(())
  }

  /// The after run function.
  fn after_run(&mut self) {
    if let Some(baker) = &mut self.baker.take() {
      baker.wait_idle().expect("Failed to wait the renderer idle.");
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
          let baker = self.baker.as_mut().unwrap();

          ui.window("SDF Baker")
            .collapsed(false, imgui::Condition::FirstUseEver)
            .position([10.0, 10.0], imgui::Condition::FirstUseEver)
            .always_auto_resize(true)
            .build(|| {
              let _ = ui.checkbox("To Bake SDF(checked) or UDF(unchecked)", &mut baker.settings.is_sdf);

              if let Some(_t) = ui.tree_node("Advanced Settings") {
                if baker.settings.is_sdf {
                  let _ = ui.input_int("Sign Passes Count", &mut baker.settings.sign_passes_count).build();
                  let _ = ui.input_float("In/Out Threshold", &mut baker.settings.in_out_threshold).build();
                }
                let _ = ui.input_float("Surface Offset", &mut baker.settings.surface_offset).build();

                ui.separator();
              }

              ui.tree_node_config("Common Settings").opened(true, imgui::Condition::FirstUseEver).build(|| {
                let mut need_to_fit = false;
                let mut need_to_snap = false;
                if ui.input_int("Selected Mesh", &mut baker.settings.selected_mesh_index).build() {
                  baker.settings.selected_mesh_index = baker.settings.selected_mesh_index.clamp(0, baker.get_num_of_meshes() as i32 - 1);
                  need_to_fit = true;
                  need_to_snap = true;
                }
                if imgui::Drag::new("Max Resolution")
                  .range(2, 1024)
                  .build(ui, &mut baker.settings.max_resolution)
                {
                  need_to_fit = true;
                  need_to_snap = true;
                }
                let _ = ui.input_float3("Center", &mut baker.settings.center).build();
                ui.disabled(true, || {
                  let _ = ui.input_float3("Desired Size", &mut baker.settings.desired_size).build();
                  let _ = ui.input_float3("Actual Size", &mut baker.settings.actual_size).build();
                });
                if ui.input_float3("Padding", &mut baker.settings.padding).build() {
                  baker.settings.padding[0] = baker.settings.padding[0].max(0.0);
                  baker.settings.padding[1] = baker.settings.padding[1].max(0.0);
                  baker.settings.padding[2] = baker.settings.padding[2].max(0.0);
                  need_to_fit = true;
                  need_to_snap = true;
                }

                if need_to_fit {
                  baker.fit_box_to_bounds();
                }
                if need_to_snap {
                  baker.snap_box_to_bounds();
                }
              });

              if let Some(_t) = ui.tree_node("Debug Settings") {
                let _ = ui.checkbox("Show Desired Box", &mut baker.settings.show_desired_box);
                let _ = ui.checkbox("Show Actual Box", &mut baker.settings.show_actual_box);
                let _ = ui.checkbox("Show Wireframe", &mut baker.settings.show_wireframe);
                if baker.settings.is_sdf {
                  let _ = ui.checkbox("Show Render Targets", &mut baker.settings.show_render_targets);
                  let _ = ui.checkbox("Show Ray Map", &mut baker.settings.show_ray_map);
                  let _ = ui.checkbox("Show SDF", &mut baker.settings.show_sdf);
                } else {
                  let _ = ui.checkbox("Show UDF", &mut baker.settings.show_sdf);
                }
              }

              ui.separator();

              if ui.button_with_size("Bake", [100.0, 30.0]) {
                match if baker.settings.is_sdf {
                  baker.bake_sdf()
                } else {
                  baker.bake_udf()
                } {
                  Ok(_) => {
                    log::info!("Bake success.");
                  },
                  Err(e) => {
                    log::error!("Bake failed: {:?}", e);
                  }
                }
              }
              ui.same_line();
              if ui.button_with_size("Save", [100.0, 30.0]) {
                match if baker.settings.is_sdf {
                  baker.save_sdf(std::path::Path::new(&self.output_file))
                } else {
                  baker.save_udf(std::path::Path::new(&self.output_file))
                } {
                  Ok(_) => {
                    log::info!("Save success.");
                  },
                  Err(e) => {
                    log::error!("Save failed: {:?}", e);
                  }
                }
              }
            }
          );

          // let mut b = true;
          // ui.show_demo_window(&mut b);
        }
      )?;
      imgui.end_frame()?;
    }

    if let Some(baker) = &mut self.baker {
      baker.update(
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
    if let Some(baker) = &mut self.baker {
      baker.render()?;
    }

    Ok(())
  }

  /// The mouse button event function.
  /// param button: The mouse button.
  /// param is_pressed: The flag to indicate the button is pressed or released.
  /// return: The result.
  fn on_mouse_button_event(&mut self, button: winit::event::MouseButton, is_pressed: bool) -> Result<()> {
    if let Some(baker) = &mut self.baker {
      if button == winit::event::MouseButton::Left && is_pressed {
        baker.begin_rotate_camera()?;
      } else if button == winit::event::MouseButton::Left && !is_pressed {
        baker.end_rotate_camera()?;
      }
    }
    Ok(())
  }

  fn on_mouse_cursor_event(&mut self, x: f32, y: f32) -> Result<()> {
    if let Some(baker) = &mut self.baker {
      if baker.is_rotating_camera() {
        baker.rotate_camera(x, y)?;
      }
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
    .arg(arg!(-o --output [OUTPUT_FILE] "The file path of the output file."))
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