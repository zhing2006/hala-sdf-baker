use std::rc::Rc;
use std::cell::RefCell;

use std::collections::HashMap;

use hala_renderer::error::HalaRendererError;
use hala_renderer::compute_program::HalaComputeProgram;

use crate::config;

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub(super) struct UDFBakerCSGlobalUniform {
  pub dimensions: [u32; 3],
  pub num_of_voxels: u32,
  pub num_of_triangles: u32,
  pub initial_distance: f32,
  pub max_size: f32,
  pub max_dimension: u32,
  pub center: [f32; 3],
  pub padding0: f32,
  pub extents: [f32; 3],
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub(super) struct UDFBakerCSMeshUniform {
  pub vertex_position_offset: u32,
  pub vertex_stride: u32,
  pub index_stride: u32,
}

/// The UDF baker resources.
pub(crate) struct UDFBakerResources {
  pub(crate) static_descriptor_set: hala_gfx::HalaDescriptorSet,
  pub(crate) global_uniform_buffer: hala_gfx::HalaBuffer,

  pub(crate) mesh_uniform_buffer: hala_gfx::HalaBuffer,

  pub(crate) distance_texture: Option<hala_gfx::HalaImage>,

  pub(crate) jump_buffer: Option<hala_gfx::HalaBuffer>,
  pub(crate) jump_buffer_bis: Option<hala_gfx::HalaBuffer>,

  pub(crate) descriptor_sets: HashMap<String, hala_gfx::HalaDescriptorSet>,

  pub(crate) compute_programs: HashMap<String, HalaComputeProgram>,
}

/// The implementation of UDF baker resources.
impl UDFBakerResources {

  /// Create a new UDF baker resources.
  /// param logical_device: The logical device.
  /// param descriptor_pool: The descriptor pool.
  /// param baker_config: The baker config.
  /// param pipeline_cache: The pipeline cache.
  /// return: The result.
  pub(crate) fn new(
    logical_device: Rc<RefCell<hala_gfx::HalaLogicalDevice>>,
    descriptor_pool: Rc<RefCell<hala_gfx::HalaDescriptorPool>>,
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
      std::mem::size_of::<UDFBakerCSGlobalUniform>() as u64,
      hala_gfx::HalaBufferUsageFlags::UNIFORM_BUFFER,
      hala_gfx::HalaMemoryLocation::CpuToGpu,
      "baker_global.uniform_buffer",
    )?;
    static_descriptor_set.update_uniform_buffers(0, 0, &[&global_uniform_buffer]);

    let mesh_uniform_buffer = hala_gfx::HalaBuffer::new(
      logical_device.clone(),
      std::mem::size_of::<UDFBakerCSMeshUniform>() as u64,
      hala_gfx::HalaBufferUsageFlags::UNIFORM_BUFFER,
      hala_gfx::HalaMemoryLocation::CpuToGpu,
      "mesh.uniform_buffer",
    )?;

    let mut descriptor_sets = HashMap::new();
    let mut compute_programs = HashMap::new();
    let mut compute_descs = HashMap::new();
    for (name, desc) in baker_config.compute_programs.iter() {
      compute_descs.insert(name.clone(), desc);
    }
    let dup_descriptor_set_names = ["jump_flooding"];
    for &descriptor_set_name in dup_descriptor_set_names.iter() {
      let desc = &baker_config.compute_programs[descriptor_set_name];
      let name = format!("{}_2", descriptor_set_name);
      compute_descs.insert(name, desc);
    }
    for (name, &desc) in compute_descs.iter() {
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
        let descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
          logical_device.clone(),
          descriptor_pool.clone(),
          hala_gfx::HalaDescriptorSetLayout::new(
            logical_device.clone(),
            descriptor_bindings.as_slice(),
            &format!("{}.descriptor_set_layout", name),
          )?,
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

    Ok(Self {
      static_descriptor_set,
      global_uniform_buffer,

      mesh_uniform_buffer,

      distance_texture: None,

      jump_buffer: None,
      jump_buffer_bis: None,

      descriptor_sets,
      compute_programs,
    })
  }

}
