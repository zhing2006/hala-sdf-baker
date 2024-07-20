use std::rc::Rc;

use hala_gfx::HalaGPURequirements;

use hala_renderer::error::HalaRendererError;
use hala_renderer::scene::{
  cpu, gpu, loader
};
use hala_renderer::shader_cache::HalaShaderCache;
use hala_renderer::graphics_program::HalaGraphicsProgram;
use hala_renderer::renderer::{
  HalaRendererInfo,
  HalaRendererResources,
  HalaRendererData,
  HalaRendererStatistics,
  HalaRendererTrait,
};

pub mod settings;
pub mod sdf_resources;
pub mod udf_resources;
pub mod draw;
pub mod debug;
pub mod bake;

use crate::config;
use crate::baker::settings::SDFBakerSettings;
use crate::baker::sdf_resources::{
  SDFBakerResources,
  SDFBakerSDFVisualizationUniform,
};
use crate::baker::udf_resources::UDFBakerResources;

/// The axis enum.
pub(crate) enum Axis {
  X,
  Y,
  Z,
}

#[repr(C, align(4))]
#[derive(Debug, Clone, Copy)]
pub struct GlobalUniform {
  // The view matrix.
  pub v_mtx: glam::Mat4,
  // The projection matrix.
  pub p_mtx: glam::Mat4,
  // The view-projection matrix.
  pub vp_mtx: glam::Mat4,
}

#[repr(C, align(4))]
#[derive(Debug, Clone, Copy)]
pub struct ObjectUniform {
  // The model matrix.
  pub m_mtx: glam::Mat4,
  // The inverse model matrix.
  pub i_m_mtx: glam::Mat4,
  // The model-view matrix.
  pub mv_mtx: glam::Mat4,
  // The transposed model-view matrix.
  pub t_mv_mtx: glam::Mat4,
  // The inverse transposed model-view matrix.
  pub it_mv_mtx: glam::Mat4,
  // The model-view-projection matrix.
  pub mvp_mtx: glam::Mat4,
}

/// The SDF baker.
pub struct SDFBaker {
  pub(crate) info: HalaRendererInfo,

  pub(crate) resources: std::mem::ManuallyDrop<HalaRendererResources>,

  pub(crate) bake_command_buffers: std::mem::ManuallyDrop<hala_gfx::HalaCommandBufferSet>,

  pub(crate) static_descriptor_set: std::mem::ManuallyDrop<hala_gfx::HalaDescriptorSet>,
  pub(crate) global_uniform_buffer: std::mem::ManuallyDrop<hala_gfx::HalaBuffer>,

  pub(crate) baker_config: config::BakerConfig,
  pub(crate) sdf_baker_resources: std::mem::ManuallyDrop<SDFBakerResources>,
  pub(crate) udf_baker_resources: std::mem::ManuallyDrop<UDFBakerResources>,

  pub(crate) wireframe_program: Option<HalaGraphicsProgram>,
  pub(crate) wireframe_debug_program: Option<HalaGraphicsProgram>,
  pub(crate) bounds_program: std::mem::ManuallyDrop<HalaGraphicsProgram>,

  pub(crate) dynamic_descriptor_set: Option<hala_gfx::HalaDescriptorSet>,
  pub(crate) object_uniform_buffers: Vec<Vec<hala_gfx::HalaBuffer>>,

  pub(crate) scene_in_gpu: Option<gpu::HalaScene>,
  pub(crate) num_of_meshes: usize,
  pub(crate) matrices_in_scene: Vec<glam::Mat4>,

  pub(crate) textures_descriptor_set: Option<hala_gfx::HalaDescriptorSet>,

  pub(crate) data: HalaRendererData,
  pub(crate) statistics: HalaRendererStatistics,

  pub(crate) image3d_sampler: std::mem::ManuallyDrop<hala_gfx::HalaSampler>,

  pub(crate) cross_xyz_descriptor_set: std::mem::ManuallyDrop<hala_gfx::HalaDescriptorSet>,
  pub(crate) cross_xyz_program: std::mem::ManuallyDrop<HalaGraphicsProgram>,

  pub(crate) sdf_visualization_uniform_buffer: std::mem::ManuallyDrop<hala_gfx::HalaBuffer>,
  pub(crate) sdf_visualization_descriptor_set: std::mem::ManuallyDrop<hala_gfx::HalaDescriptorSet>,
  pub(crate) sdf_visualization_program: std::mem::ManuallyDrop<HalaGraphicsProgram>,

  pub settings: SDFBakerSettings,

  is_rotating_camera: bool,
  begin_rotating_camera_x: f32,
  begin_rotating_camera_y: f32,
}

/// The Drop implementation of the SDF baker.
impl Drop for SDFBaker {

  fn drop(&mut self) {
    self.textures_descriptor_set = None;

    self.scene_in_gpu = None;

    self.object_uniform_buffers.clear();
    self.dynamic_descriptor_set = None;

    self.wireframe_program = None;
    self.wireframe_debug_program = None;
    HalaShaderCache::get_instance().borrow_mut().clear();
    unsafe {
      std::mem::ManuallyDrop::drop(&mut self.sdf_visualization_program);
      std::mem::ManuallyDrop::drop(&mut self.sdf_visualization_descriptor_set);
      std::mem::ManuallyDrop::drop(&mut self.sdf_visualization_uniform_buffer);
      std::mem::ManuallyDrop::drop(&mut self.cross_xyz_program);
      std::mem::ManuallyDrop::drop(&mut self.cross_xyz_descriptor_set);
      std::mem::ManuallyDrop::drop(&mut self.image3d_sampler);
      std::mem::ManuallyDrop::drop(&mut self.bounds_program);
      std::mem::ManuallyDrop::drop(&mut self.udf_baker_resources);
      std::mem::ManuallyDrop::drop(&mut self.sdf_baker_resources);
      std::mem::ManuallyDrop::drop(&mut self.global_uniform_buffer);
      std::mem::ManuallyDrop::drop(&mut self.static_descriptor_set);
      std::mem::ManuallyDrop::drop(&mut self.bake_command_buffers);
      std::mem::ManuallyDrop::drop(&mut self.resources);
    }
    log::debug!("A HalaRenderer \"{}\" is dropped.", self.info().name);
  }

}

/// The implementation of the SDF baker.
impl SDFBaker {

  /// Create a new renderer.
  /// param name: The name of the SDF baker.
  /// param gpu_req: The GPU requirements of the SDF baker.
  /// param window: The window of the SDF baker.
  /// return: the SDF baker.
  pub fn new(
    name: &str,
    gpu_req: &HalaGPURequirements,
    window: &winit::window::Window,
  ) -> Result<Self, HalaRendererError> {
    let width = gpu_req.width;
    let height = gpu_req.height;

    let resources = HalaRendererResources::new(
      name,
      gpu_req,
      window,
      &Self::get_descriptor_sizes(),
    )?;

    let bake_command_buffers = hala_gfx::HalaCommandBufferSet::new(
      Rc::clone(&resources.context.borrow().logical_device),
      Rc::clone(&resources.context.borrow().pools),
      hala_gfx::HalaCommandBufferType::GRAPHICS,
      hala_gfx::HalaCommandBufferLevel::PRIMARY,
      1,
      "bake.cmd_buffer",
    )?;

    let static_descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
      Rc::clone(&resources.context.borrow().logical_device),
      Rc::clone(&resources.descriptor_pool),
      hala_gfx::HalaDescriptorSetLayout::new(
        Rc::clone(&resources.context.borrow().logical_device),
        &[
          hala_gfx::HalaDescriptorSetLayoutBinding { // Global uniform buffer.
            binding_index: 0,
            descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | (if resources.context.borrow().gpu_req.require_mesh_shader { hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH } else { hala_gfx::HalaShaderStageFlags::VERTEX }),
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Cameras uniform buffer.
            binding_index: 1,
            descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | (if resources.context.borrow().gpu_req.require_mesh_shader { hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH } else { hala_gfx::HalaShaderStageFlags::VERTEX }),
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Lights uniform buffer.
            binding_index: 2,
            descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | (if resources.context.borrow().gpu_req.require_mesh_shader { hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH } else { hala_gfx::HalaShaderStageFlags::VERTEX }),
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
        ],
        "main_static.descriptor_set_layout",
      )?,
      0,
      "main_static.descriptor_set",
    )?;

    // Create global uniform buffer.
    let global_uniform_buffer = hala_gfx::HalaBuffer::new(
      Rc::clone(&resources.context.borrow().logical_device),
      std::mem::size_of::<GlobalUniform>() as u64,
      hala_gfx::HalaBufferUsageFlags::UNIFORM_BUFFER,
      hala_gfx::HalaMemoryLocation::CpuToGpu,
      "global.uniform_buffer",
    )?;

    // Load shaders.
    let mut features = vec!["SDF_BAKER"];
    features.push("CONSERVATIVE_RASTERIZATION");
    let shaders_dir = if cfg!(debug_assertions) {
      format!("shaders/output/debug/hala-sdf-baker/{}", features.join("#"))
    } else {
      format!("shaders/output/release/hala-sdf-baker/{}", features.join("#"))
    };
    HalaShaderCache::get_instance().borrow_mut().set_shader_dir(&shaders_dir);

    // Create pipelines.
    // If we have cache file at ./out/pipeline_cache.bin, we can load it.
    let pipeline_cache = if std::path::Path::new("./out/pipeline_cache.bin").exists() {
      log::debug!("Load pipeline cache from file: ./out/pipeline_cache.bin");
      hala_gfx::HalaPipelineCache::with_cache_file(
        Rc::clone(&resources.context.borrow().logical_device),
        "./out/pipeline_cache.bin",
      )?
    } else {
      log::debug!("Create a new pipeline cache.");
      hala_gfx::HalaPipelineCache::new(
        Rc::clone(&resources.context.borrow().logical_device),
      )?
    };

    let sdf_baker_config = match config::BakerConfig::load("./conf/sdf_baker.yaml") {
      Ok(config) => config,
      Err(err) => {
        log::error!("Failed to load sdf baker config: {:?}", err);
        return Err(HalaRendererError::new("Failed to load sdf baker config.", None));
      }
    };
    let sdf_baker_resources = SDFBakerResources::new(
      Rc::clone(&resources.context.borrow().logical_device),
      Rc::clone(&resources.descriptor_pool),
      &resources.context.borrow().swapchain,
      &sdf_baker_config,
      &pipeline_cache,
    )?;

    let udf_baker_config = match config::BakerConfig::load("./conf/udf_baker.yaml") {
      Ok(config) => config,
      Err(err) => {
        log::error!("Failed to load udf baker config: {:?}", err);
        return Err(HalaRendererError::new("Failed to load udf baker config.", None));
      }
    };
    let udf_baker_resources = UDFBakerResources::new(
      Rc::clone(&resources.context.borrow().logical_device),
      Rc::clone(&resources.descriptor_pool),
      &udf_baker_config,
      &pipeline_cache,
    )?;

    let bounds_desc = sdf_baker_config.graphics_programs.get("bounds").ok_or(HalaRendererError::new("Failed to get graphics program \"bounds\".", None))?;
    let bounds_program = HalaGraphicsProgram::new(
      Rc::clone(&resources.context.borrow().logical_device),
      &resources.context.borrow().swapchain,
      &[] as &[&hala_gfx::HalaDescriptorSetLayout],
      hala_gfx::HalaPipelineCreateFlags::default(),
      &[] as &[hala_gfx::HalaVertexInputAttributeDescription],
      &[] as &[hala_gfx::HalaVertexInputBindingDescription],
      &[],
      bounds_desc,
      Some(&pipeline_cache),
      "bounds",
    )?;

    let image3d_sampler = hala_gfx::HalaSampler::new(
      Rc::clone(&resources.context.borrow().logical_device),
      (hala_gfx::HalaFilter::LINEAR, hala_gfx::HalaFilter::LINEAR),
      hala_gfx::HalaSamplerMipmapMode::LINEAR,
      (hala_gfx::HalaSamplerAddressMode::CLAMP_TO_EDGE, hala_gfx::HalaSamplerAddressMode::CLAMP_TO_EDGE, hala_gfx::HalaSamplerAddressMode::CLAMP_TO_EDGE),
      0.0,
      false,
      1.0,
      (0.0, 0.0),
      "cross_xyz.sampler",
    )?;

    let cross_xyz_desc = sdf_baker_config.graphics_programs.get("cross_xyz").ok_or(HalaRendererError::new("Failed to get graphics program \"cross_xyz\".", None))?;
    let cross_xyz_bindings = cross_xyz_desc.bindings.iter().enumerate().map(|(binding_index, binding_type)| {
      hala_gfx::HalaDescriptorSetLayoutBinding {
        binding_index: binding_index as u32,
        descriptor_type: *binding_type,
        descriptor_count: 1,
        stage_flags: hala_gfx::HalaShaderStageFlags::VERTEX | hala_gfx::HalaShaderStageFlags::FRAGMENT,
        binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
      }
    }).collect::<Vec<_>>();
    let cross_xyz_descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
      Rc::clone(&resources.context.borrow().logical_device),
      Rc::clone(&resources.descriptor_pool),
      hala_gfx::HalaDescriptorSetLayout::new(
        Rc::clone(&resources.context.borrow().logical_device),
        cross_xyz_bindings.as_slice(),
        "cross_xyz.descriptor_set_layout",
      )?,
      0,
      "cross_xyz.descriptor_set",
    )?;
    let cross_xyz_program = HalaGraphicsProgram::new(
      Rc::clone(&resources.context.borrow().logical_device),
      &resources.context.borrow().swapchain,
      &[&cross_xyz_descriptor_set.layout],
      hala_gfx::HalaPipelineCreateFlags::default(),
      &[] as &[hala_gfx::HalaVertexInputAttributeDescription],
      &[] as &[hala_gfx::HalaVertexInputBindingDescription],
      &[],
      cross_xyz_desc,
      Some(&pipeline_cache),
      "cross_xyz",
    )?;

    let sdf_visualization_uniform_buffer = hala_gfx::HalaBuffer::new(
      Rc::clone(&resources.context.borrow().logical_device),
      std::mem::size_of::<SDFBakerSDFVisualizationUniform>() as u64,
      hala_gfx::HalaBufferUsageFlags::UNIFORM_BUFFER,
      hala_gfx::HalaMemoryLocation::CpuToGpu,
      "sdf_visualization.uniform_buffer",
    )?;
    let sdf_visualization_desc = sdf_baker_config.graphics_programs.get("sdf_visualization")
      .ok_or(HalaRendererError::new("Failed to get graphics program \"sdf_visualization\".", None))?;
    let sdf_visualization_bindings = sdf_visualization_desc.bindings.iter().enumerate().map(|(binding_index, binding_type)| {
      hala_gfx::HalaDescriptorSetLayoutBinding {
        binding_index: binding_index as u32,
        descriptor_type: *binding_type,
        descriptor_count: 1,
        stage_flags: hala_gfx::HalaShaderStageFlags::VERTEX | hala_gfx::HalaShaderStageFlags::FRAGMENT,
        binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
      }
    }).collect::<Vec<_>>();
    let sdf_visualization_descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
      Rc::clone(&resources.context.borrow().logical_device),
      Rc::clone(&resources.descriptor_pool),
      hala_gfx::HalaDescriptorSetLayout::new(
        Rc::clone(&resources.context.borrow().logical_device),
        sdf_visualization_bindings.as_slice(),
        "sdf_visualization.descriptor_set_layout",
      )?,
      0,
      "sdf_visualization.descriptor_set",
    )?;
    let sdf_visualization_program = HalaGraphicsProgram::new(
      Rc::clone(&resources.context.borrow().logical_device),
      &resources.context.borrow().swapchain,
      &[
        &sdf_visualization_descriptor_set.layout
      ],
      hala_gfx::HalaPipelineCreateFlags::default(),
      &[] as &[hala_gfx::HalaVertexInputAttributeDescription],
      &[] as &[hala_gfx::HalaVertexInputBindingDescription],
      &[],
      sdf_visualization_desc,
      Some(&pipeline_cache),
      "sdf_visualization",
    )?;

    pipeline_cache.save("./out/pipeline_cache.bin")?;

    // Return the SDF baker.
    log::debug!("A HalaRenderer \"{}\"[{} x {}] is created.", name, width, height);
    Ok(Self {
      info: HalaRendererInfo::new(name, width, height),

      resources: std::mem::ManuallyDrop::new(resources),

      bake_command_buffers: std::mem::ManuallyDrop::new(bake_command_buffers),

      static_descriptor_set: std::mem::ManuallyDrop::new(static_descriptor_set),
      global_uniform_buffer: std::mem::ManuallyDrop::new(global_uniform_buffer),

      baker_config: sdf_baker_config,
      sdf_baker_resources: std::mem::ManuallyDrop::new(sdf_baker_resources),
      udf_baker_resources: std::mem::ManuallyDrop::new(udf_baker_resources),

      wireframe_program: None,
      wireframe_debug_program: None,
      bounds_program: std::mem::ManuallyDrop::new(bounds_program),

      dynamic_descriptor_set: None,
      object_uniform_buffers: Vec::new(),

      scene_in_gpu: None,
      num_of_meshes: 0,
      matrices_in_scene: Vec::new(),

      textures_descriptor_set: None,

      data: HalaRendererData::new(),
      statistics: HalaRendererStatistics::new(),

      settings: SDFBakerSettings::default(),

      image3d_sampler: std::mem::ManuallyDrop::new(image3d_sampler),

      cross_xyz_descriptor_set: std::mem::ManuallyDrop::new(cross_xyz_descriptor_set),
      cross_xyz_program: std::mem::ManuallyDrop::new(cross_xyz_program),

      sdf_visualization_uniform_buffer: std::mem::ManuallyDrop::new(sdf_visualization_uniform_buffer),
      sdf_visualization_descriptor_set: std::mem::ManuallyDrop::new(sdf_visualization_descriptor_set),
      sdf_visualization_program: std::mem::ManuallyDrop::new(sdf_visualization_program),

      is_rotating_camera: false,
      begin_rotating_camera_x: f32::NAN,
      begin_rotating_camera_y: f32::NAN,
    })
  }

  /// Set the scene to be rendered.
  /// param scene_in_cpu: The scene in the CPU.
  /// return: The result.
  pub fn set_scene(&mut self, scene_in_cpu: &mut cpu::HalaScene) -> Result<(), HalaRendererError> {
    let scene_in_gpu = {
      let context = self.resources.context.borrow();
      // Release the old scene in the GPU.
      self.scene_in_gpu = None;

      // Upload the new scene to the GPU.
      loader::HalaSceneGPUUploader::upload(
        &context,
        &self.resources.graphics_command_buffers,
        &self.resources.transfer_command_buffers,
        scene_in_cpu,
        false,
      false)
    }?;

    for mesh in scene_in_gpu.meshes.iter() {
      for _ in mesh.primitives.iter() {
        self.num_of_meshes += 1;
        self.matrices_in_scene.push(mesh.transform);
      }
    }

    self.scene_in_gpu = Some(scene_in_gpu);

    self.fit_box_to_bounds();
    self.snap_box_to_bounds();

    Ok(())
  }

  pub fn begin_rotate_camera(&mut self) -> Result<(), HalaRendererError> {
    self.is_rotating_camera = true;
    self.begin_rotating_camera_x = f32::NAN;
    self.begin_rotating_camera_y = f32::NAN;
    Ok(())
  }

  pub fn end_rotate_camera(&mut self) -> Result<(), HalaRendererError> {
    self.is_rotating_camera = false;
    Ok(())
  }

  pub fn is_rotating_camera(&self) -> bool {
    self.is_rotating_camera
  }

  pub fn rotate_camera(&mut self, x: f32, y: f32) -> Result<(), HalaRendererError> {
    if self.is_rotating_camera {
      if self.begin_rotating_camera_x.is_nan() {
        self.begin_rotating_camera_x = x;
      }
      if self.begin_rotating_camera_y.is_nan() {
        self.begin_rotating_camera_y = y;
      }

      if let Some(scene_in_gpu) = self.scene_in_gpu.as_mut() {
        let delta_x = x - self.begin_rotating_camera_x;
        let delta_y = y - self.begin_rotating_camera_y;
        let sensitivity = 0.005f32;
        let camera_position = scene_in_gpu.camera_view_matrices[0].w_axis.truncate();
        let translation = glam::Mat4::from_translation(camera_position - glam::Vec3::from_array(self.settings.center));
        let inverse_translation = glam::Mat4::from_translation(glam::Vec3::from_array(self.settings.center) - camera_position);
        let rot_x = glam::Mat3::from_rotation_x(delta_y * sensitivity);
        let rot_y = glam::Mat3::from_rotation_y(delta_x * sensitivity);
        let rotation = rot_y * rot_x;
        let mtx = translation * glam::Mat4::from_mat3(rotation) * inverse_translation;
        scene_in_gpu.camera_view_matrices[0] = mtx * scene_in_gpu.camera_view_matrices[0];
      }

      self.begin_rotating_camera_x = x;
      self.begin_rotating_camera_y = y;
    }
    Ok(())
  }

}

/// The implementation of the SDF baker trait.
impl HalaRendererTrait for SDFBaker {

  fn info(&self) -> &HalaRendererInfo {
    &self.info
  }

  fn info_mut(&mut self) -> &mut HalaRendererInfo {
    &mut self.info
  }

  fn resources(&self) -> &HalaRendererResources {
    &self.resources
  }

  fn resources_mut(&mut self) -> &mut HalaRendererResources {
    &mut self.resources
  }

  fn data(&self) -> &HalaRendererData {
    &self.data
  }

  fn data_mut(&mut self) -> &mut HalaRendererData {
    &mut self.data
  }

  fn statistics(&self) -> &HalaRendererStatistics {
    &self.statistics
  }

  fn statistics_mut(&mut self) -> &mut HalaRendererStatistics {
    &mut self.statistics
  }

  fn get_descriptor_sizes() -> Vec<(hala_gfx::HalaDescriptorType, usize)> {
    vec![
      (
        hala_gfx::HalaDescriptorType::STORAGE_IMAGE,
        8,
      ),
      (
        hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
        32,
      ),
      (
        hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
        256,
      ),
      (
        hala_gfx::HalaDescriptorType::SAMPLED_IMAGE,
        256,
      ),
      (
        hala_gfx::HalaDescriptorType::SAMPLER,
        256,
      ),
      (
        hala_gfx::HalaDescriptorType::COMBINED_IMAGE_SAMPLER,
        256,
      ),
    ]
  }

  /// Commit all GPU resources.
  /// return: The result.
  fn commit(&mut self) -> Result<(), HalaRendererError> {
    let context = self.resources.context.borrow();
    let scene = self.scene_in_gpu.as_ref().ok_or(HalaRendererError::new("The scene in GPU is none!", None))?;

    // Assert camera count.
    if scene.camera_view_matrices.is_empty() || scene.camera_proj_matrices.is_empty() {
      return Err(HalaRendererError::new("There is no camera in the scene!", None));
    }

    // Collect vertex and index buffers.
    let mut vertex_buffers = Vec::new();
    let mut index_buffers = Vec::new();
    for mesh in scene.meshes.iter() {
      for primitive in mesh.primitives.iter() {
        vertex_buffers.push(primitive.vertex_buffer.as_ref());
        index_buffers.push(primitive.index_buffer.as_ref());
      }
    }

    // Create dynamic descriptor set.
    let dynamic_descriptor_set = hala_gfx::HalaDescriptorSet::new(
      Rc::clone(&context.logical_device),
      Rc::clone(&self.resources.descriptor_pool),
      hala_gfx::HalaDescriptorSetLayout::new(
        Rc::clone(&context.logical_device),
        &[
          hala_gfx::HalaDescriptorSetLayoutBinding { // Materials uniform buffers.
            binding_index: 0,
            descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
            descriptor_count: scene.materials.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Object uniform buffers.
            binding_index: 1,
            descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
            descriptor_count: scene.meshes.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Vertex storage buffers.
            binding_index: 2,
            descriptor_type: hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
            descriptor_count: vertex_buffers.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Index storage buffers.
            binding_index: 3,
            descriptor_type: hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
            descriptor_count: index_buffers.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
        ],
        "main_dynamic.descriptor_set_layout",
      )?,
      context.swapchain.num_of_images,
      0,
      "main_dynamic.descriptor_set",
    )?;

    for (mesh_index, mesh) in scene.meshes.iter().enumerate() {
      // Prepare object data.
      let mv_mtx = scene.camera_view_matrices[0] * mesh.transform;
      let object_uniform = ObjectUniform {
        m_mtx: mesh.transform,
        i_m_mtx: mesh.transform.inverse(),
        mv_mtx,
        t_mv_mtx: mv_mtx.transpose(),
        it_mv_mtx: mv_mtx.inverse().transpose(),
        mvp_mtx: scene.camera_proj_matrices[0] * mv_mtx,
      };

      // Create object uniform buffer.
      let mut buffers = Vec::with_capacity(context.swapchain.num_of_images);
      for index in 0..context.swapchain.num_of_images {
        let buffer = hala_gfx::HalaBuffer::new(
          Rc::clone(&context.logical_device),
          std::mem::size_of::<ObjectUniform>() as u64,
          hala_gfx::HalaBufferUsageFlags::UNIFORM_BUFFER,
          hala_gfx::HalaMemoryLocation::CpuToGpu,
          &format!("object_{}_{}.uniform_buffer", mesh_index, index),
        )?;

        buffer.update_memory(0, &[object_uniform])?;

        buffers.push(buffer);
      }

      self.object_uniform_buffers.push(buffers);
    }

    for index in 0..context.swapchain.num_of_images {
      dynamic_descriptor_set.update_uniform_buffers(
        index,
        0,
        scene.materials.as_slice(),
      );
      dynamic_descriptor_set.update_uniform_buffers(
        index,
        1,
        self.object_uniform_buffers.iter().map(|buffers| &buffers[index]).collect::<Vec<_>>().as_slice(),
      );
      dynamic_descriptor_set.update_storage_buffers(
        index,
        2,
        vertex_buffers.as_slice(),
      );
      dynamic_descriptor_set.update_storage_buffers(
        index,
        3,
        index_buffers.as_slice(),
      );
    }

    // Update static descriptor set.
    self.static_descriptor_set.update_uniform_buffers(0, 0, &[self.global_uniform_buffer.as_ref()]);
    self.static_descriptor_set.update_uniform_buffers(0, 1, &[scene.cameras.as_ref()]);
    self.static_descriptor_set.update_uniform_buffers(0, 2, &[scene.lights.as_ref()]);

    // Create texture descriptor set.
    let textures_descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
      Rc::clone(&context.logical_device),
      Rc::clone(&self.resources.descriptor_pool),
      hala_gfx::HalaDescriptorSetLayout::new(
        Rc::clone(&context.logical_device),
        &[
          hala_gfx::HalaDescriptorSetLayoutBinding { // All textures in the scene.
            binding_index: 0,
            descriptor_type: hala_gfx::HalaDescriptorType::SAMPLED_IMAGE,
            descriptor_count: scene.textures.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding {
            binding_index: 1,
            descriptor_type: hala_gfx::HalaDescriptorType::SAMPLER,
            descriptor_count: scene.textures.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
        ],
        "textures.descriptor_set_layout",
      )?,
      0,
      "textures.descriptor_set",
    )?;

    let textures: &Vec<_> = scene.textures.as_ref();
    let samplers: &Vec<_> = scene.samplers.as_ref();
    let images: &Vec<_> = scene.images.as_ref();
    let mut final_images = Vec::new();
    let mut final_samplers = Vec::new();
    for (sampler_index, image_index) in textures.iter().enumerate() {
      let image = images.get(*image_index as usize).ok_or(HalaRendererError::new("The image is none!", None))?;
      let sampler = samplers.get(sampler_index).ok_or(HalaRendererError::new("The sampler is none!", None))?;
      final_images.push(image);
      final_samplers.push(sampler);
    }
    if !final_images.is_empty() && !final_samplers.is_empty() {
      textures_descriptor_set.update_sampled_images(0, 0, final_images.as_slice());
      textures_descriptor_set.update_samplers(0, 1, final_samplers.as_slice());
    }

    // If we have cache file at ./out/pipeline_cache.bin, we can load it.
    let pipeline_cache = if std::path::Path::new("./out/pipeline_cache.bin").exists() {
      log::debug!("Load pipeline cache from file: ./out/pipeline_cache.bin");
      hala_gfx::HalaPipelineCache::with_cache_file(
        Rc::clone(&context.logical_device),
        "./out/pipeline_cache.bin",
      )?
    } else {
      log::debug!("Create a new pipeline cache.");
      hala_gfx::HalaPipelineCache::new(
        Rc::clone(&context.logical_device),
      )?
    };

    // Create graphics pipelines.
    let wireframe_desc = self.baker_config.graphics_programs.get("wireframe").ok_or(HalaRendererError::new("Failed to get the wireframe program.", None))?;
    let wireframe_program = HalaGraphicsProgram::new(
      Rc::clone(&context.logical_device),
      &context.swapchain,
      &[
        &self.static_descriptor_set.layout,
        &dynamic_descriptor_set.layout,
        &textures_descriptor_set.layout
      ],
      hala_gfx::HalaPipelineCreateFlags::default(),
      &[
        hala_gfx::HalaVertexInputAttributeDescription {
          binding: 0,
          location: 0,
          offset: 0,
          format: hala_gfx::HalaFormat::R32G32B32_SFLOAT, // Position.
        },
      ],
      &[
        hala_gfx::HalaVertexInputBindingDescription {
          binding: 0,
          stride: std::mem::size_of::<hala_renderer::scene::HalaVertex>() as u32,
          input_rate: hala_gfx::HalaVertexInputRate::VERTEX,
        }
      ],
      &[hala_gfx::HalaDynamicState::VIEWPORT],
      wireframe_desc,
      Some(&pipeline_cache),
      "wireframe"
    )?;
    let wireframe_debug_desc = self.baker_config.graphics_programs.get("wireframe_debug").ok_or(HalaRendererError::new("Failed to get the wireframe debug program.", None))?;
    let wireframe_debug_program = HalaGraphicsProgram::new(
      Rc::clone(&context.logical_device),
      &context.swapchain,
      &[
        &self.static_descriptor_set.layout,
        &dynamic_descriptor_set.layout,
        &textures_descriptor_set.layout
      ],
      hala_gfx::HalaPipelineCreateFlags::default(),
      &[
        hala_gfx::HalaVertexInputAttributeDescription {
          binding: 0,
          location: 0,
          offset: 0,
          format: hala_gfx::HalaFormat::R32G32B32_SFLOAT, // Position.
        },
      ],
      &[
        hala_gfx::HalaVertexInputBindingDescription {
          binding: 0,
          stride: 16,
          input_rate: hala_gfx::HalaVertexInputRate::VERTEX,
        }
      ],
      &[hala_gfx::HalaDynamicState::VIEWPORT],
      wireframe_debug_desc,
      Some(&pipeline_cache),
      "wireframe_debug"
    )?;

    // Save pipeline cache.
    pipeline_cache.save("./out/pipeline_cache.bin")?;

    self.wireframe_program = Some(wireframe_program);
    self.wireframe_debug_program = Some(wireframe_debug_program);

    self.dynamic_descriptor_set = Some(dynamic_descriptor_set);
    self.textures_descriptor_set = Some(textures_descriptor_set);

    Ok(())
  }

  /// Update the SDF baker.
  /// param delta_time: The delta time.
  /// param width: The width of the window.
  /// param height: The height of the window.
  /// param ui_fn: The draw UI function.
  /// return: The result.
  fn update<F>(&mut self, _delta_time: f64, width: u32, height: u32, ui_fn: F) -> Result<(), HalaRendererError>
    where F: FnOnce(usize, &hala_gfx::HalaCommandBufferSet) -> Result<(), hala_gfx::HalaGfxError>
  {
    self.pre_update(width, height)?;

    // TEMP: for test ONLY! Don't forget to remove it.
    // self.bake_udf()?;

    let scene = self.scene_in_gpu.as_ref().ok_or(HalaRendererError::new("The scene in GPU is none!", None))?;
    let context = self.resources.context.borrow();

    // Update global uniform buffer(Only use No.1 camera).
    self.global_uniform_buffer.update_memory(0, &[GlobalUniform {
      v_mtx: scene.camera_view_matrices[0],
      p_mtx: scene.camera_proj_matrices[0],
      vp_mtx: scene.camera_proj_matrices[0] * scene.camera_view_matrices[0],
    }])?;

    // Update object uniform buffers.
    for (mesh_index, mesh) in scene.meshes.iter().enumerate() {
      // Prepare object data.
      let mv_mtx = scene.camera_view_matrices[0] * mesh.transform;
      let object_uniform = ObjectUniform {
        m_mtx: mesh.transform,
        i_m_mtx: mesh.transform.inverse(),
        mv_mtx,
        t_mv_mtx: mv_mtx.transpose(),
        it_mv_mtx: mv_mtx.inverse().transpose(),
        mvp_mtx: scene.camera_proj_matrices[0] * mv_mtx,
      };

      for index in 0..context.swapchain.num_of_images {
        let buffer = self.object_uniform_buffers[mesh_index][index].as_ref();
        buffer.update_memory(0, &[object_uniform])?;
      }
    }

    // Update SDF visualization uniform buffer.
    let sdf_visualization_uniform = SDFBakerSDFVisualizationUniform {
      m_mtx: *self.get_model_matrix_in_scene(self.settings.selected_mesh_index),
      i_m_mtx: self.get_model_matrix_in_scene(self.settings.selected_mesh_index).inverse(),
      vp_mtx: self.get_vp_matrix_in_scene(),
      mvp_mtx: self.get_mvp_matrix_in_scene(self.settings.selected_mesh_index),
      camera_position: self.get_camera_position(0),
      offset: 0.0,
      dimensions: self.estimate_grid_size(),
      inv_resolution: 1.0 / self.settings.max_resolution as f32,
    };
    self.sdf_visualization_uniform_buffer.update_memory(0, std::slice::from_ref(&sdf_visualization_uniform))?;

    // Update the SDF baker.
    self.record_command_buffer(
      self.data.image_index,
      &self.resources.graphics_command_buffers,
      ui_fn,
    )?;

    Ok(())
  }

}
