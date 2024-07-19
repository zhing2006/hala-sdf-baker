use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::config;

use hala_renderer::{
  error::HalaRendererError,
  compute_program::HalaComputeProgram,
  graphics_program::HalaGraphicsProgram,
};

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub(super) struct SDFBakerCSGlobalUniform {
  pub dimensions: [u32; 3],
  pub max_dimension: u32,
  pub upper_bound_count: u32,
  pub num_of_triangles: u32,
  pub max_extent: f32,
  pub padding0: f32,
  pub min_bounds_extended: [f32; 3],
  pub padding1: f32,
  pub max_bounds_extended: [f32; 3],
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub(super) struct SDFBakerCSMeshUniform {
  pub vertex_position_offset: u32,
  pub vertex_stride: u32,
  pub index_stride: u32,
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub(super) struct SDFBakerCSConservativeRasterizationUniform {
  pub world_to_clip: [glam::Mat4; 3],
  pub clip_to_world: [glam::Mat4; 3],
  pub conservative_offset: f32,
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub(super) struct SDFBakerSDFVisualizationUniform {
  pub m_mtx: glam::Mat4,
  pub i_m_mtx: glam::Mat4,
  pub vp_mtx: glam::Mat4,
  pub mvp_mtx: glam::Mat4,
  pub camera_position: glam::Vec3,
  pub offset: f32,
  pub dimensions: [u32; 3],
  pub inv_resolution: f32,
}

/// The baker resources.
pub(crate) struct SDFBakerResources {
  pub(crate) static_descriptor_set: hala_gfx::HalaDescriptorSet,
  pub(crate) global_uniform_buffer: hala_gfx::HalaBuffer,

  pub(crate) mesh_uniform_buffer: hala_gfx::HalaBuffer,
  pub(crate) conservative_rasterization_uniform_buffer: hala_gfx::HalaBuffer,

  pub(crate) descriptor_sets: HashMap<String, hala_gfx::HalaDescriptorSet>,

  pub(crate) compute_programs: HashMap<String, HalaComputeProgram>,

  pub(crate) triangle_uvw_buffer: Option<hala_gfx::HalaBuffer>,
  pub(crate) coord_flip_buffer: Option<hala_gfx::HalaBuffer>,
  pub(crate) aabb_buffer: Option<hala_gfx::HalaBuffer>,
  pub(crate) vertices_buffer: Option<hala_gfx::HalaBuffer>,

  pub(crate) voxels_buffer: Option<hala_gfx::HalaBuffer>,
  pub(crate) counters_buffer: Option<hala_gfx::HalaBuffer>,
  pub(crate) in_sum_blocks_buffer: Option<hala_gfx::HalaBuffer>,
  pub(crate) sum_blocks_buffer: Option<hala_gfx::HalaBuffer>,
  pub(crate) additional_sum_blocks_buffer: Option<hala_gfx::HalaBuffer>,
  pub(crate) accum_counters_buffer: Option<hala_gfx::HalaBuffer>,
  pub(crate) accum_sum_blocks_buffer: Option<hala_gfx::HalaBuffer>,
  pub(crate) tmp_buffer: Option<hala_gfx::HalaBuffer>,

  pub(crate) triangles_in_voxels_buffer: Option<hala_gfx::HalaBuffer>,

  pub(crate) ray_map: Option<hala_gfx::HalaImage>,
  pub(crate) sign_map: Option<hala_gfx::HalaImage>,
  pub(crate) sign_map_bis: Option<hala_gfx::HalaImage>,

  pub(crate) voxels_texture: Option<hala_gfx::HalaImage>,
  pub(crate) voxels_texture_bis: Option<hala_gfx::HalaImage>,

  pub(crate) distance_texture: Option<hala_gfx::HalaImage>,

  pub(crate) render_targets: [Option<hala_gfx::HalaImage>; 3],

  pub(crate) image_2_screen_sampler: hala_gfx::HalaSampler,
  pub(crate) image_2_screen_descriptor_sets: [hala_gfx::HalaDescriptorSet; 3],
  pub(crate) image_2_screen_program: HalaGraphicsProgram,

  pub(crate) image3d_sampler: hala_gfx::HalaSampler,

  pub(crate) cross_xyz_descriptor_set: hala_gfx::HalaDescriptorSet,
  pub(crate) cross_xyz_program: HalaGraphicsProgram,

  pub(crate) sdf_visualization_uniform_buffer: hala_gfx::HalaBuffer,
  pub(crate) sdf_visualization_descriptor_set: hala_gfx::HalaDescriptorSet,
  pub(crate) sdf_visualization_program: HalaGraphicsProgram,

  pub(crate) write_uvw_and_coverage_programs: [Option<HalaGraphicsProgram>; 3],
  pub(crate) write_triangle_ids_to_voxels_programs: [Option<HalaGraphicsProgram>; 3],

  pub(crate) num_of_triangles: u32,
}

impl SDFBakerResources {

  pub const PREFIX_SUM_THREAD_GROUP_SIZE: u32 = 512;

  /// Create a new SDF baker resources.
  /// param logical_device: The logical device.
  /// param descriptor_pool: The descriptor pool.
  /// param swapchain: The swapchain.
  /// param baker_config: The baker config.
  /// param pipeline_cache: The pipeline cache.
  /// return: The result.
  pub(crate) fn new(
    logical_device: Rc<RefCell<hala_gfx::HalaLogicalDevice>>,
    descriptor_pool: Rc<RefCell<hala_gfx::HalaDescriptorPool>>,
    swapchain: &hala_gfx::HalaSwapchain,
    baker_config: &config::BakerConfig,
    pipeline_cache: &hala_gfx::HalaPipelineCache,
  ) -> Result<Self, HalaRendererError> {
    let static_descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
      logical_device.clone(),
      descriptor_pool.clone(),
      hala_gfx::HalaDescriptorSetLayout::new(
        logical_device.clone(),
        &[
          hala_gfx::HalaDescriptorSetLayoutBinding { // Global uniform buffer.
            binding_index: 0,
            descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: hala_gfx::HalaShaderStageFlags::VERTEX | hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
        ],
        "baker_global.descriptor_set_layout",
      )?,
      0,
      "baker_global.descriptor_set",
    )?;
    let global_uniform_buffer = hala_gfx::HalaBuffer::new(
      logical_device.clone(),
      std::mem::size_of::<SDFBakerCSGlobalUniform>() as u64,
      hala_gfx::HalaBufferUsageFlags::UNIFORM_BUFFER,
      hala_gfx::HalaMemoryLocation::CpuToGpu,
      "baker_global.uniform_buffer",
    )?;
    static_descriptor_set.update_uniform_buffers(0, 0, &[&global_uniform_buffer]);

    let mesh_uniform_buffer = hala_gfx::HalaBuffer::new(
      logical_device.clone(),
      std::mem::size_of::<SDFBakerCSMeshUniform>() as u64,
      hala_gfx::HalaBufferUsageFlags::UNIFORM_BUFFER,
      hala_gfx::HalaMemoryLocation::CpuToGpu,
      "mesh.uniform_buffer",
    )?;

    let conservative_rasterization_uniform_buffer = hala_gfx::HalaBuffer::new(
      logical_device.clone(),
      std::mem::size_of::<SDFBakerCSConservativeRasterizationUniform>() as u64,
      hala_gfx::HalaBufferUsageFlags::UNIFORM_BUFFER,
      hala_gfx::HalaMemoryLocation::CpuToGpu,
      "conservative_rasterization.uniform_buffer",
    )?;

    let mut descriptor_sets = HashMap::new();
    let mut compute_programs = HashMap::new();
    for (name, desc) in baker_config.compute_programs.iter() {
      let descriptor_bindings = desc.bindings.iter().enumerate().map(|(binding_index, binding_type)| {
        hala_gfx::HalaDescriptorSetLayoutBinding {
          binding_index: binding_index as u32,
          descriptor_type: *binding_type,
          descriptor_count: 1,
          stage_flags: hala_gfx::HalaShaderStageFlags::COMPUTE,
          binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
        }
      }).collect::<Vec<_>>();

      let mut descriptor_set_layouts = vec![&static_descriptor_set.layout];
      if !descriptor_bindings.is_empty() {
        let descriptor_set = hala_gfx::HalaDescriptorSet::new(
          logical_device.clone(),
          descriptor_pool.clone(),
          hala_gfx::HalaDescriptorSetLayout::new(
            logical_device.clone(),
            descriptor_bindings.as_slice(),
            &format!("{}.descriptor_set_layout", name),
          )?,
          swapchain.num_of_images,
          0,
          &format!("{}.descriptor_set", name),
        )?;
        descriptor_sets.insert(name.clone(), descriptor_set);
        let layout = &descriptor_sets[name].layout;
        descriptor_set_layouts.push(layout);
      }

      let program = HalaComputeProgram::new(
        logical_device.clone(),
        descriptor_set_layouts.as_slice(),
        desc,
        Some(pipeline_cache),
        name,
      )?;
      compute_programs.insert(name.clone(), program);
    }
    let dup_descriptor_set_names = ["in_bucket_sum", "block_sum", "final_sum", "sign_pass_neighbors", "jfa"];
    for &descriptor_set_name in dup_descriptor_set_names.iter() {
      let desc = &baker_config.compute_programs[descriptor_set_name];
      let name = format!("{}_2", descriptor_set_name);
      let descriptor_bindings = desc.bindings.iter().enumerate().map(|(binding_index, binding_type)| {
        hala_gfx::HalaDescriptorSetLayoutBinding {
          binding_index: binding_index as u32,
          descriptor_type: *binding_type,
          descriptor_count: 1,
          stage_flags: hala_gfx::HalaShaderStageFlags::COMPUTE,
          binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
        }
      }).collect::<Vec<_>>();

      if !descriptor_bindings.is_empty() {
        let descriptor_set = hala_gfx::HalaDescriptorSet::new(
          logical_device.clone(),
          descriptor_pool.clone(),
          hala_gfx::HalaDescriptorSetLayout::new(
            logical_device.clone(),
            descriptor_bindings.as_slice(),
            &format!("{}.descriptor_set_layout", name),
          )?,
          swapchain.num_of_images,
          0,
          &format!("{}.descriptor_set", name),
        )?;
        descriptor_sets.insert(name.clone(), descriptor_set);
      }
    }

    let image_2_screen_sampler = hala_gfx::HalaSampler::new(
      logical_device.clone(),
      (hala_gfx::HalaFilter::LINEAR, hala_gfx::HalaFilter::LINEAR),
      hala_gfx::HalaSamplerMipmapMode::LINEAR,
      (hala_gfx::HalaSamplerAddressMode::CLAMP_TO_EDGE, hala_gfx::HalaSamplerAddressMode::CLAMP_TO_EDGE, hala_gfx::HalaSamplerAddressMode::CLAMP_TO_EDGE),
      0.0,
      false,
      1.0,
      (0.0, 0.0),
      "image_2_screen.sampler",
    )?;
    let image_2_screen_desc = baker_config.graphics_programs.get("image_2_screen")
      .ok_or(HalaRendererError::new("Failed to get graphics program \"image_2_screen\".", None))?;
    let descriptor_bindings = image_2_screen_desc.bindings.iter().enumerate().map(|(binding_index, binding_type)| {
      hala_gfx::HalaDescriptorSetLayoutBinding {
        binding_index: binding_index as u32,
        descriptor_type: *binding_type,
        descriptor_count: 1,
        stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT,
        binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
      }
    }).collect::<Vec<_>>();
    let image_2_screen_descriptor_sets = [
      hala_gfx::HalaDescriptorSet::new_static(
        logical_device.clone(),
        descriptor_pool.clone(),
        hala_gfx::HalaDescriptorSetLayout::new(
          logical_device.clone(),
          descriptor_bindings.as_slice(),
          "image_2_screen_0.descriptor_set_layout",
        )?,
        0,
        "image_2_screen_0.descriptor_set",
      )?,
      hala_gfx::HalaDescriptorSet::new_static(
        logical_device.clone(),
        descriptor_pool.clone(),
        hala_gfx::HalaDescriptorSetLayout::new(
          logical_device.clone(),
          descriptor_bindings.as_slice(),
          "image_2_screen_1.descriptor_set_layout",
        )?,
        0,
        "image_2_screen_1.descriptor_set",
      )?,
      hala_gfx::HalaDescriptorSet::new_static(
        logical_device.clone(),
        descriptor_pool.clone(),
        hala_gfx::HalaDescriptorSetLayout::new(
          logical_device.clone(),
          descriptor_bindings.as_slice(),
          "image_2_screen_2.descriptor_set_layout",
        )?,
        0,
        "image_2_screen_2.descriptor_set",
      )?,
    ];
    let image_2_screen_program = HalaGraphicsProgram::new(
      logical_device.clone(),
      swapchain,
      &[&image_2_screen_descriptor_sets[0].layout],
      hala_gfx::HalaPipelineCreateFlags::default(),
      &[] as &[hala_gfx::HalaVertexInputAttributeDescription],
      &[] as &[hala_gfx::HalaVertexInputBindingDescription],
      &[
        hala_gfx::HalaDynamicState::VIEWPORT,
      ],
      image_2_screen_desc,
      Some(pipeline_cache),
      "image_2_screen",
    )?;

    let image3d_sampler = hala_gfx::HalaSampler::new(
      logical_device.clone(),
      (hala_gfx::HalaFilter::LINEAR, hala_gfx::HalaFilter::LINEAR),
      hala_gfx::HalaSamplerMipmapMode::LINEAR,
      (hala_gfx::HalaSamplerAddressMode::CLAMP_TO_EDGE, hala_gfx::HalaSamplerAddressMode::CLAMP_TO_EDGE, hala_gfx::HalaSamplerAddressMode::CLAMP_TO_EDGE),
      0.0,
      false,
      1.0,
      (0.0, 0.0),
      "cross_xyz.sampler",
    )?;

    let cross_xyz_desc = baker_config.graphics_programs.get("cross_xyz").ok_or(HalaRendererError::new("Failed to get graphics program \"cross_xyz\".", None))?;
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
      logical_device.clone(),
      descriptor_pool.clone(),
      hala_gfx::HalaDescriptorSetLayout::new(
        logical_device.clone(),
        cross_xyz_bindings.as_slice(),
        "cross_xyz.descriptor_set_layout",
      )?,
      0,
      "cross_xyz.descriptor_set",
    )?;
    let cross_xyz_program = HalaGraphicsProgram::new(
      logical_device.clone(),
      swapchain,
      &[&cross_xyz_descriptor_set.layout],
      hala_gfx::HalaPipelineCreateFlags::default(),
      &[] as &[hala_gfx::HalaVertexInputAttributeDescription],
      &[] as &[hala_gfx::HalaVertexInputBindingDescription],
      &[],
      cross_xyz_desc,
      Some(pipeline_cache),
      "cross_xyz",
    )?;

    let sdf_visualization_uniform_buffer = hala_gfx::HalaBuffer::new(
      logical_device.clone(),
      std::mem::size_of::<SDFBakerSDFVisualizationUniform>() as u64,
      hala_gfx::HalaBufferUsageFlags::UNIFORM_BUFFER,
      hala_gfx::HalaMemoryLocation::CpuToGpu,
      "sdf_visualization.uniform_buffer",
    )?;
    let sdf_visualization_desc = baker_config.graphics_programs.get("sdf_visualization")
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
      logical_device.clone(),
      descriptor_pool.clone(),
      hala_gfx::HalaDescriptorSetLayout::new(
        logical_device.clone(),
        sdf_visualization_bindings.as_slice(),
        "sdf_visualization.descriptor_set_layout",
      )?,
      0,
      "sdf_visualization.descriptor_set",
    )?;
    let sdf_visualization_program = HalaGraphicsProgram::new(
      logical_device.clone(),
      swapchain,
      &[
        &sdf_visualization_descriptor_set.layout
      ],
      hala_gfx::HalaPipelineCreateFlags::default(),
      &[] as &[hala_gfx::HalaVertexInputAttributeDescription],
      &[] as &[hala_gfx::HalaVertexInputBindingDescription],
      &[],
      sdf_visualization_desc,
      Some(pipeline_cache),
      "sdf_visualization",
    )?;

    let write_uvw_and_coverage_desc = baker_config.graphics_programs.get("write_uvw_and_coverage")
      .ok_or(HalaRendererError::new("Failed to get graphics program \"write_uvw_and_coverage\".", None))?;
    let write_uvw_and_coverage_bindings = write_uvw_and_coverage_desc.bindings.iter().enumerate().map(|(binding_index, binding_type)| {
      hala_gfx::HalaDescriptorSetLayoutBinding {
        binding_index: binding_index as u32,
        descriptor_type: *binding_type,
        descriptor_count: 1,
        stage_flags: hala_gfx::HalaShaderStageFlags::VERTEX | hala_gfx::HalaShaderStageFlags::FRAGMENT,
        binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
      }
    }).collect::<Vec<_>>();
    let write_uvw_and_coverage_descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
      logical_device.clone(),
      descriptor_pool.clone(),
      hala_gfx::HalaDescriptorSetLayout::new(
        logical_device.clone(),
        write_uvw_and_coverage_bindings.as_slice(),
        "write_uvw_and_coverage.descriptor_set_layout",
      )?,
      0,
      "write_uvw_and_coverage.descriptor_set",
    )?;
    descriptor_sets.insert("write_uvw_and_coverage".to_string(), write_uvw_and_coverage_descriptor_set);

    let write_triangle_ids_to_voxels_desc = baker_config.graphics_programs.get("write_triangle_ids_to_voxels")
      .ok_or(HalaRendererError::new("Failed to get graphics program \"write_triangle_ids_to_voxels\".", None))?;
    let write_triangle_ids_to_voxels_bindings = write_triangle_ids_to_voxels_desc.bindings.iter().enumerate().map(|(binding_index, binding_type)| {
      hala_gfx::HalaDescriptorSetLayoutBinding {
        binding_index: binding_index as u32,
        descriptor_type: *binding_type,
        descriptor_count: 1,
        stage_flags: hala_gfx::HalaShaderStageFlags::VERTEX | hala_gfx::HalaShaderStageFlags::FRAGMENT,
        binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
      }
    }).collect::<Vec<_>>();
    let write_triangle_ids_to_voxels_descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
      logical_device.clone(),
      descriptor_pool.clone(),
      hala_gfx::HalaDescriptorSetLayout::new(
        logical_device.clone(),
        write_triangle_ids_to_voxels_bindings.as_slice(),
        "write_triangle_ids_to_voxels.descriptor_set_layout",
      )?,
      0,
      "write_triangle_ids_to_voxels.descriptor_set",
    )?;
    descriptor_sets.insert("write_triangle_ids_to_voxels".to_string(), write_triangle_ids_to_voxels_descriptor_set);

    const IMAGE_REPEAT_NONE: Option<hala_gfx::HalaImage> = None;
    const PROGRAM_REPEAT_NONE: Option<HalaGraphicsProgram> = None;
    Ok(Self {
      static_descriptor_set,
      global_uniform_buffer,

      mesh_uniform_buffer,
      conservative_rasterization_uniform_buffer,

      descriptor_sets,

      compute_programs,

      triangle_uvw_buffer: None,
      coord_flip_buffer: None,
      aabb_buffer: None,
      vertices_buffer: None,

      voxels_buffer: None,
      counters_buffer: None,
      in_sum_blocks_buffer: None,
      sum_blocks_buffer: None,
      additional_sum_blocks_buffer: None,
      accum_counters_buffer: None,
      accum_sum_blocks_buffer: None,
      tmp_buffer: None,

      triangles_in_voxels_buffer: None,

      ray_map: None,
      sign_map: None,
      sign_map_bis: None,

      voxels_texture: None,
      voxels_texture_bis: None,

      distance_texture: None,

      render_targets: [IMAGE_REPEAT_NONE; 3],

      image_2_screen_sampler,
      image_2_screen_descriptor_sets,
      image_2_screen_program,

      image3d_sampler,

      cross_xyz_descriptor_set,
      cross_xyz_program,

      sdf_visualization_uniform_buffer,
      sdf_visualization_descriptor_set,
      sdf_visualization_program,

      write_uvw_and_coverage_programs: [PROGRAM_REPEAT_NONE; 3],
      write_triangle_ids_to_voxels_programs: [PROGRAM_REPEAT_NONE; 3],

      num_of_triangles: 0,
    })
  }

}