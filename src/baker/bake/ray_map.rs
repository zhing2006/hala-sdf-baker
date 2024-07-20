use std::rc::Rc;

use hala_renderer::error::HalaRendererError;

use crate::baker::SDFBaker;

impl SDFBaker {

  pub(super) fn ray_map_create_buffers_images(
    &mut self,
    dimensions: &[u32; 3],
  ) -> Result<(), HalaRendererError> {
    if let Some(ray_map) = &self.sdf_baker_resources.ray_map {
      if ray_map.extent.width != dimensions[0] || ray_map.extent.height != dimensions[1] || ray_map.extent.depth != dimensions[2] {
        self.sdf_baker_resources.ray_map = None;
      }
    }
    if self.sdf_baker_resources.ray_map.is_none() {
      self.sdf_baker_resources.ray_map = Some(
        hala_gfx::HalaImage::new_3d(
          Rc::clone(&self.resources.context.borrow().logical_device),
          hala_gfx::HalaImageUsageFlags::SAMPLED | hala_gfx::HalaImageUsageFlags::STORAGE,
          hala_gfx::HalaFormat::R32G32B32A32_SFLOAT,
          dimensions[0],
          dimensions[1],
          dimensions[2],
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "ray_map.image3d",
        )?
      );
    }

    Ok(())
  }

  pub(super) fn ray_map_update(
    &self,
    accum_counters_buffer: &hala_gfx::HalaBuffer,
    triangles_in_voxels_buffer: &hala_gfx::HalaBuffer,
    triangle_uvw_buffer: &hala_gfx::HalaBuffer,
    ray_map: &hala_gfx::HalaImage,
  ) -> Result<
    (
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
    ),
    HalaRendererError
  > {
    let generate_ray_map_local2x2_descriptor_set =  self.sdf_baker_resources.descriptor_sets.get("generate_ray_map_local2x2")
      .ok_or(HalaRendererError::new("Failed to get the generate_ray_map_local2x2 descriptor set.", None))?;
    generate_ray_map_local2x2_descriptor_set.update_storage_buffers(
      0,
      0,
      &[accum_counters_buffer],
    );
    generate_ray_map_local2x2_descriptor_set.update_storage_buffers(
      0,
      1,
      &[triangles_in_voxels_buffer],
    );
    generate_ray_map_local2x2_descriptor_set.update_storage_buffers(
      0,
      2,
      &[triangle_uvw_buffer],
    );
    generate_ray_map_local2x2_descriptor_set.update_storage_images(
      0,
      3,
      &[ray_map],
    );

    let ray_map_sum_x_descriptor_set =  self.sdf_baker_resources.descriptor_sets.get("ray_map_sum_x")
      .ok_or(HalaRendererError::new("Failed to get the ray_map_sum_x descriptor set.", None))?;
    ray_map_sum_x_descriptor_set.update_storage_images(
      0,
      0,
      &[ray_map],
    );

    let ray_map_sum_y_descriptor_set =  self.sdf_baker_resources.descriptor_sets.get("ray_map_sum_y")
      .ok_or(HalaRendererError::new("Failed to get the ray_map_sum_y descriptor set.", None))?;
    ray_map_sum_y_descriptor_set.update_storage_images(
      0,
      0,
      &[ray_map],
    );

    let ray_map_sum_z_descriptor_set =  self.sdf_baker_resources.descriptor_sets.get("ray_map_sum_z")
      .ok_or(HalaRendererError::new("Failed to get the ray_map_sum_z descriptor set.", None))?;
    ray_map_sum_z_descriptor_set.update_storage_images(
      0,
      0,
      &[ray_map],
    );

    Ok((
      generate_ray_map_local2x2_descriptor_set,
      ray_map_sum_x_descriptor_set,
      ray_map_sum_y_descriptor_set,
      ray_map_sum_z_descriptor_set,
    ))
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn ray_map_compute(
    &self,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    accum_counters_buffer: &hala_gfx::HalaBuffer,
    triangles_in_voxels_buffer: &hala_gfx::HalaBuffer,
    triangle_uvw_buffer: &hala_gfx::HalaBuffer,
    ray_map: &hala_gfx::HalaImage,
    generate_ray_map_local2x2_descriptor_set: &hala_gfx::HalaDescriptorSet,
    _ray_map_sum_x_descriptor_set: &hala_gfx::HalaDescriptorSet,
    _ray_map_sum_y_descriptor_set: &hala_gfx::HalaDescriptorSet,
    _ray_map_sum_z_descriptor_set: &hala_gfx::HalaDescriptorSet,
    dimensions: &[u32; 3],
  ) -> Result<(), HalaRendererError> {
    // accum_counters_buffer and triangles_in_voxels_buffer be going to be read by compute shaders.
    {
      command_buffers.set_buffer_barriers(
        0,
        &[
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            size: triangle_uvw_buffer.size,
            buffer: triangle_uvw_buffer.raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            size: accum_counters_buffer.size,
            buffer: accum_counters_buffer.raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            size: triangles_in_voxels_buffer.size,
            buffer: triangles_in_voxels_buffer.raw,
            ..Default::default()
          },
        ],
      );
    }

    // Generate ray map for each local 2x2 block by 8 different offsets.
    // offsets = (0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 0), (1, 0, 1), (0, 1, 1), (1, 1, 1)
    {
      let generate_ray_map_local2x2_program = self.sdf_baker_resources.compute_programs.get("generate_ray_map_local2x2")
        .ok_or(HalaRendererError::new("Failed to get the generate_ray_map_local2x2 compute program.", None))?;

      generate_ray_map_local2x2_program.bind(
        0,
        command_buffers,
        &[
          &self.sdf_baker_resources.static_descriptor_set,
          generate_ray_map_local2x2_descriptor_set,
        ],
      );

      let mut ray_map_offsets = [0u32, 0u32, 0u32];
      for i in 0..8u32 {
        command_buffers.set_image_barriers(
          0,
          &[
            hala_gfx::HalaImageBarrierInfo {
              old_layout: hala_gfx::HalaImageLayout::GENERAL,
              new_layout: hala_gfx::HalaImageLayout::GENERAL,
              src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
              dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ | hala_gfx::HalaAccessFlags2::SHADER_WRITE,
              aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
              image: ray_map.raw,
              ..Default::default()
            },
          ],
        );

        ray_map_offsets[0] = i & 1;
        ray_map_offsets[1] = (i & 2) >> 1;
        ray_map_offsets[2] = (i & 4) >> 2;

        let mut push_constants = Vec::new();
        push_constants.extend_from_slice(&ray_map_offsets[0].to_le_bytes());
        push_constants.extend_from_slice(&ray_map_offsets[1].to_le_bytes());
        push_constants.extend_from_slice(&ray_map_offsets[2].to_le_bytes());
        generate_ray_map_local2x2_program.push_constants(
          0,
          command_buffers,
          0,
          &push_constants,
        );

        generate_ray_map_local2x2_program.dispatch(
          0,
          command_buffers,
          (dimensions[0] + 16 - 1) / 16,
          (dimensions[1] + 16 - 1) / 16,
          (dimensions[2] + 16 - 1) / 16,
        );
      }
    }

    // Sum ray map from x, y and z directions.
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
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ | hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
            image: ray_map.raw,
            ..Default::default()
          },
        ],
      );

      let ray_map_sum_x_program = self.sdf_baker_resources.compute_programs.get("ray_map_sum_x")
        .ok_or(HalaRendererError::new("Failed to get the ray_map_sum_x compute program.", None))?;
      ray_map_sum_x_program.bind(
        0,
        command_buffers,
        &[
          &self.sdf_baker_resources.static_descriptor_set,
          _ray_map_sum_x_descriptor_set,
        ],
      );
      ray_map_sum_x_program.dispatch(
        0,
        command_buffers,
        1,
        (dimensions[1] + 8 - 1) / 8,
        (dimensions[2] + 8 - 1) / 8,
      );

      command_buffers.set_image_barriers(
        0,
        &[
          hala_gfx::HalaImageBarrierInfo {
            old_layout: hala_gfx::HalaImageLayout::GENERAL,
            new_layout: hala_gfx::HalaImageLayout::GENERAL,
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ | hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
            image: ray_map.raw,
            ..Default::default()
          },
        ],
      );

      let ray_map_sum_y_program = self.sdf_baker_resources.compute_programs.get("ray_map_sum_y")
        .ok_or(HalaRendererError::new("Failed to get the ray_map_sum_y compute program.", None))?;
      ray_map_sum_y_program.bind(
        0,
        command_buffers,
        &[
          &self.sdf_baker_resources.static_descriptor_set,
          _ray_map_sum_y_descriptor_set,
        ],
      );
      ray_map_sum_y_program.dispatch(
        0,
        command_buffers,
        (dimensions[0] + 8 - 1) / 8,
        1,
        (dimensions[2] + 8 - 1) / 8,
      );

      command_buffers.set_image_barriers(
        0,
        &[
          hala_gfx::HalaImageBarrierInfo {
            old_layout: hala_gfx::HalaImageLayout::GENERAL,
            new_layout: hala_gfx::HalaImageLayout::GENERAL,
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ | hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
            image: ray_map.raw,
            ..Default::default()
          },
        ],
      );

      let ray_map_sum_z_program = self.sdf_baker_resources.compute_programs.get("ray_map_sum_z")
        .ok_or(HalaRendererError::new("Failed to get the ray_map_sum_z compute program.", None))?;
      ray_map_sum_z_program.bind(
        0,
        command_buffers,
        &[
          &self.sdf_baker_resources.static_descriptor_set,
          _ray_map_sum_z_descriptor_set,
        ],
      );
      ray_map_sum_z_program.dispatch(
        0,
        command_buffers,
        (dimensions[0] + 8 - 1) / 8,
        (dimensions[1] + 8 - 1) / 8,
        1,
      );
    }

    Ok(())
  }

}