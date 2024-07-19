use hala_renderer::error::HalaRendererError;

use crate::baker::SDFBaker;

impl SDFBaker {

  #[allow(clippy::too_many_arguments)]
  pub(super) fn initialize_update(
    &self,
    voxels_buffer: &hala_gfx::HalaBuffer,
    counters_buffer: &hala_gfx::HalaBuffer,
    accum_counters_buffer: &hala_gfx::HalaBuffer,
    ray_map: &hala_gfx::HalaImage,
    sign_map: &hala_gfx::HalaImage,
    sign_map_bis: &hala_gfx::HalaImage,
    voxels_texture: &hala_gfx::HalaImage,
    voxels_texture_bis: &hala_gfx::HalaImage,
  ) -> Result<&hala_gfx::HalaDescriptorSet, HalaRendererError> {
    let descriptor_set = self.baker_resources.descriptor_sets.get("init")
      .ok_or(HalaRendererError::new("Failed to get the initialize descriptor set.", None))?;
    descriptor_set.update_storage_buffers(
      0,
      0,
      &[voxels_buffer],
    );
    descriptor_set.update_storage_buffers(
      0,
      1,
      &[counters_buffer],
    );
    descriptor_set.update_storage_buffers(
      0,
      2,
      &[accum_counters_buffer],
    );
    descriptor_set.update_storage_images(
      0,
      3,
      &[ray_map],
    );
    descriptor_set.update_storage_images(
      0,
      4,
      &[sign_map],
    );
    descriptor_set.update_storage_images(
      0,
      5,
      &[sign_map_bis],
    );
    descriptor_set.update_storage_images(
      0,
      6,
      &[voxels_texture],
    );
    descriptor_set.update_storage_images(
      0,
      7,
      &[voxels_texture_bis],
    );

    Ok(descriptor_set)
  }

  pub(super) fn initialize_compute(
    &self,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    voxels_buffer: &hala_gfx::HalaBuffer,
    counters_buffer: &hala_gfx::HalaBuffer,
    accum_counters_buffer: &hala_gfx::HalaBuffer,
    descriptor_set: &hala_gfx::HalaDescriptorSet,
    dimensions: &[u32; 3],
  ) -> Result<(), HalaRendererError> {
    command_buffers.set_buffer_barriers(
      0,
      &[
        hala_gfx::HalaBufferBarrierInfo {
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          size: voxels_buffer.size,
          buffer: voxels_buffer.raw,
          ..Default::default()
        },
        hala_gfx::HalaBufferBarrierInfo {
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          size: counters_buffer.size,
          buffer: counters_buffer.raw,
          ..Default::default()
        },
        hala_gfx::HalaBufferBarrierInfo {
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          size: accum_counters_buffer.size,
          buffer: accum_counters_buffer.raw,
          ..Default::default()
        },
      ]
    );

    let program = self.baker_resources.compute_programs.get("init")
      .ok_or(HalaRendererError::new("Failed to get the initialize program.", None))?;
    program.bind(
      0,
      command_buffers,
      &[
        &self.baker_resources.static_descriptor_set,
        descriptor_set,
      ]
    );
    program.dispatch(
      0,
      command_buffers,
      (dimensions[0] as f32 / 8.0).ceil() as u32,
      (dimensions[1] as f32 / 8.0).ceil() as u32,
      (dimensions[2] as f32 / 8.0).ceil() as u32,
    );

    Ok(())
  }

}