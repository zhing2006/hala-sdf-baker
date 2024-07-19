use std::rc::Rc;

use hala_renderer::error::HalaRendererError;

use crate::baker::SDFBaker;

impl SDFBaker {

  pub(super) fn dtw_create_buffers_images(
    &mut self,
    dimensions: &[u32; 3],
  ) -> Result<(), HalaRendererError> {
    if let Some(distance_texture) = &self.baker_resources.distance_texture {
      if distance_texture.extent.width != dimensions[0] || distance_texture.extent.height != dimensions[1] || distance_texture.extent.depth != dimensions[2] {
        self.baker_resources.distance_texture = None;
      }
    }
    if self.baker_resources.distance_texture.is_none() {
      self.baker_resources.distance_texture = Some(
        hala_gfx::HalaImage::new_3d(
          Rc::clone(&self.resources.context.borrow().logical_device),
          hala_gfx::HalaImageUsageFlags::SAMPLED | hala_gfx::HalaImageUsageFlags::STORAGE,
          hala_gfx::HalaFormat::R32_SFLOAT,
          dimensions[0],
          dimensions[1],
          dimensions[2],
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "distance_texture.image3d",
        )?
      );
    }

    Ok(())
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn dtw_update(
    &self,
    triangle_uvw_buffer: &hala_gfx::HalaBuffer,
    triangles_in_voxels_buffer: &hala_gfx::HalaBuffer,
    accum_counters_buffer: &hala_gfx::HalaBuffer,
    sign_map: &hala_gfx::HalaImage,
    voxels_texture: &hala_gfx::HalaImage,
    voxels_buffer: &hala_gfx::HalaBuffer,
    distance_texture: &hala_gfx::HalaImage,
  ) -> Result<&hala_gfx::HalaDescriptorSet, HalaRendererError> {
    let dtw_descriptor_set = self.baker_resources.descriptor_sets.get("distance_transform_winding")
      .ok_or(HalaRendererError::new("Failed to get the distance_transform_winding descriptor set.", None))?;
    dtw_descriptor_set.update_storage_buffers(
      0,
      0,
      &[triangle_uvw_buffer],
    );
    dtw_descriptor_set.update_storage_buffers(
      0,
      1,
      &[triangles_in_voxels_buffer],
    );
    dtw_descriptor_set.update_storage_buffers(
      0,
      2,
      &[accum_counters_buffer],
    );
    dtw_descriptor_set.update_sampled_images(
      0,
      3,
      &[sign_map],
    );
    dtw_descriptor_set.update_sampled_images(
      0,
      4,
      &[voxels_texture],
    );
    dtw_descriptor_set.update_storage_buffers(
      0,
      5,
      &[voxels_buffer],
    );
    dtw_descriptor_set.update_storage_images(
      0,
      6,
      &[distance_texture],
    );
    Ok(dtw_descriptor_set)
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn dtw_compute(
    &self,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    voxels_texture: &hala_gfx::HalaImage,
    voxels_buffer: &hala_gfx::HalaBuffer,
    distance_texture: &hala_gfx::HalaImage,
    descriptor_set: &hala_gfx::HalaDescriptorSet,
    dimensions: &[u32; 3],
  ) -> Result<(), HalaRendererError> {
    // voxels_texture be going to be read by compute shaders.
    // voxels_buffer be going to be written by compute shaders.
    // distance_texture be going to be written by compute shaders.
    {
      command_buffers.set_image_barriers(
        0,
        &[
          hala_gfx::HalaImageBarrierInfo {
            old_layout: hala_gfx::HalaImageLayout::GENERAL,
            new_layout: hala_gfx::HalaImageLayout::GENERAL,
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
            image: voxels_texture.raw,
            ..Default::default()
          },
          hala_gfx::HalaImageBarrierInfo {
            old_layout: hala_gfx::HalaImageLayout::GENERAL,
            new_layout: hala_gfx::HalaImageLayout::GENERAL,
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
            image: distance_texture.raw,
            ..Default::default()
          },
        ],
      );

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
        ],
      );
    }

    {
      let program = self.baker_resources.compute_programs.get("distance_transform_winding")
        .ok_or(HalaRendererError::new("Failed to get the distance_transform_winding compute program.", None))?;

      program.bind(
        0,
        command_buffers,
        &[
          &self.baker_resources.static_descriptor_set,
          descriptor_set,
        ],
      );

      let mut push_constants = Vec::new();
      push_constants.extend_from_slice(&self.settings.in_out_threshold.to_le_bytes());
      push_constants.extend_from_slice(&self.settings.surface_offset.to_le_bytes());
      program.push_constants(
        0,
        command_buffers,
        0,
        &push_constants,
      );

      program.dispatch(
        0,
        command_buffers,
        (dimensions[0] + 8 - 1) / 8,
        (dimensions[1] + 8 - 1) / 8,
        (dimensions[2] + 8 - 1) / 8,
      );
    }

    Ok(())
  }

}