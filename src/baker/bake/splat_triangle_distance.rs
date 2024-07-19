use hala_renderer::error::HalaRendererError;
use crate::baker::SDFBaker;

use crate::baker::udf_resources::UDFBakerCSMeshUniform;

impl SDFBaker {

  pub(super) fn splat_triangle_distance_update(
    &self,
    index_buffer: &hala_gfx::HalaBuffer,
    vertex_buffer: &hala_gfx::HalaBuffer,
    distance_texture: &hala_gfx::HalaImage,
  ) -> Result<
    &hala_gfx::HalaDescriptorSet,
    HalaRendererError
  > {
    let mesh_uniform = UDFBakerCSMeshUniform {
      vertex_position_offset: 0,
      vertex_stride: std::mem::size_of::<hala_renderer::scene::HalaVertex>() as u32,
      index_stride: std::mem::size_of::<u32>() as u32,
    };
    log::debug!("Mesh uniform: {:?}", mesh_uniform);
    self.udf_baker_resources.mesh_uniform_buffer.update_memory(0, std::slice::from_ref(&mesh_uniform))?;

    let descriptor_set = self.udf_baker_resources.descriptor_sets.get("splat_triangle_distance")
      .ok_or(HalaRendererError::new("Failed to get the splat_triangle_distance descriptor set.", None))?;
    descriptor_set.update_uniform_buffers(
      0,
      0,
      &[&self.udf_baker_resources.mesh_uniform_buffer],
    );
    descriptor_set.update_storage_buffers(
      0,
      1,
      &[index_buffer],
    );
    descriptor_set.update_storage_buffers(
      0,
      2,
      &[vertex_buffer],
    );
    descriptor_set.update_storage_images(
      0,
      3,
      &[distance_texture],
    );

    Ok(descriptor_set)
  }

  pub(super) fn splat_triangle_distance_compute(
    &self,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    distance_texture: &hala_gfx::HalaImage,
    descriptor_set: &hala_gfx::HalaDescriptorSet,
    dispatch_size_x: u32,
    dispatch_size_y: u32,
  ) -> Result<(), HalaRendererError> {
    command_buffers.set_image_barriers(
      0,
      &[
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::GENERAL,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ | hala_gfx::HalaAccessFlags2::SHADER_WRITE,
          image: distance_texture.raw,
          ..Default::default()
        },
      ],
    );

    let program = self.udf_baker_resources.compute_programs.get("splat_triangle_distance")
      .ok_or(HalaRendererError::new("Failed to get the splat_triangle_distance program.", None))?;
    program.bind(
      0,
      command_buffers,
      &[
        &self.udf_baker_resources.static_descriptor_set,
        descriptor_set,
      ]
    );
    program.dispatch(
      0,
      command_buffers,
      dispatch_size_x,
      dispatch_size_y,
      1,
    );

    Ok(())
  }

}