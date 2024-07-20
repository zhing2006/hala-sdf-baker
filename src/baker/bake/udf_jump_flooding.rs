use hala_renderer::error::HalaRendererError;

use crate::baker::SDFBaker;

impl SDFBaker {

  pub(super) fn jump_flooding_update(
    &self,
    distance_texture: &hala_gfx::HalaImage,
    jump_buffer: &hala_gfx::HalaBuffer,
    jump_buffer_bis: &hala_gfx::HalaBuffer,
  ) -> Result<
    (
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
    ),
    HalaRendererError,
  > {
    let num_of_passes = self.settings.max_resolution.ilog2() - 1;
    let get_write_jump_buffer = |i| {
      if i % 2 == 0 { // even
        jump_buffer
      } else {  // odd
        jump_buffer_bis
      }
    };
    let get_read_jump_buffer = |i| {
      if i % 2 == 0 { // even
        jump_buffer_bis
      } else {  // odd
        jump_buffer
      }
    };

    let jump_flooding_initialize_descriptor_set = self.udf_baker_resources.descriptor_sets.get("jump_flooding_init")
      .ok_or(HalaRendererError::new("Failed to get the jump floodinginitialize descriptor set.", None))?;
    jump_flooding_initialize_descriptor_set.update_sampled_images(
      0,
      0,
      &[distance_texture],
    );
    jump_flooding_initialize_descriptor_set.update_storage_buffers(
      0,
      1,
      &[jump_buffer],
    );
    let jump_flooding_odd_descriptor_set = self.udf_baker_resources.descriptor_sets.get("jump_flooding")
      .ok_or(HalaRendererError::new("Failed to get the jump_flooding descriptor set.", None))?;
    jump_flooding_odd_descriptor_set.update_storage_buffers(
      0,
      0,
      &[get_read_jump_buffer(1)],
    );
    jump_flooding_odd_descriptor_set.update_storage_buffers(
      0,
      1,
      &[get_write_jump_buffer(1)],
    );
    let jump_flooding_even_descriptor_set = self.udf_baker_resources.descriptor_sets.get("jump_flooding_2")
      .ok_or(HalaRendererError::new("Failed to get the jump_flooding_2 descriptor set.", None))?;
    jump_flooding_even_descriptor_set.update_storage_buffers(
      0,
      0,
      &[get_read_jump_buffer(0)],
    );
    jump_flooding_even_descriptor_set.update_storage_buffers(
      0,
      1,
      &[get_write_jump_buffer(0)],
    );
    let jump_flooding_finalize_descriptor_set = self.udf_baker_resources.descriptor_sets.get("jump_flooding_final")
      .ok_or(HalaRendererError::new("Failed to get the jump flooding finalize descriptor set.", None))?;
    jump_flooding_finalize_descriptor_set.update_storage_buffers(
      0,
      0,
      &[get_write_jump_buffer(num_of_passes)],
    );
    jump_flooding_finalize_descriptor_set.update_storage_images(
      0,
      1,
      &[distance_texture],
    );
    jump_flooding_finalize_descriptor_set.update_storage_buffers(
      0,
      2,
      &[get_read_jump_buffer(num_of_passes)],
    );

    Ok((
      jump_flooding_initialize_descriptor_set,
      jump_flooding_odd_descriptor_set,
      jump_flooding_even_descriptor_set,
      jump_flooding_finalize_descriptor_set,
    ))
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn jump_flooding_compute<'a: 'b, 'b>(
    &self,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    distance_texture: &hala_gfx::HalaImage,
    jump_buffer: &'a hala_gfx::HalaBuffer,
    jump_buffer_bis: &'a hala_gfx::HalaBuffer,
    jump_flooding_initialize_descriptor_set: &hala_gfx::HalaDescriptorSet,
    jump_flooding_odd_descriptor_set: &hala_gfx::HalaDescriptorSet,
    jump_flooding_even_descriptor_set: &hala_gfx::HalaDescriptorSet,
    jump_flooding_finalize_descriptor_set: &hala_gfx::HalaDescriptorSet,
    dimensions: &[u32; 3]
  ) -> Result<&'b hala_gfx::HalaBuffer, HalaRendererError> {
    // distance_texture be going to be read by compute shaders.
    // jump_buffer be going to be written by compute shaders.
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
            size: jump_buffer.size,
            buffer: jump_buffer.raw,
            ..Default::default()
          },
        ],
      );
    }

    // Initialize.
    {
      let program = self.udf_baker_resources.compute_programs.get("jump_flooding_init")
        .ok_or(HalaRendererError::new("Failed to get the jump flooding initialize compute program.", None))?;

      program.bind(
        0,
        command_buffers,
        &[
          &self.udf_baker_resources.static_descriptor_set,
          jump_flooding_initialize_descriptor_set,
        ],
      );

      program.dispatch(
        0,
        command_buffers,
        (dimensions[0] + 8 - 1) / 8,
        (dimensions[1] + 8 - 1) / 8,
        (dimensions[2] + 8 - 1) / 8,
      );
    }

    let num_of_passes = self.settings.max_resolution.ilog2() - 1;
    let get_write_jump_buffer = |i| {
      if i % 2 == 0 {
        jump_buffer
      } else {
        jump_buffer_bis
      }
    };
    let get_read_jump_buffer = |i| {
      if i % 2 == 0 {
        jump_buffer_bis
      } else {
        jump_buffer
      }
    };
    let get_jump_flooding_descriptor_set = |i| {
      if i % 2 == 0 { // even
        jump_flooding_even_descriptor_set
      } else {  // odd
        jump_flooding_odd_descriptor_set
      }
    };

    // Jump flooding.
    {
      for i in 1..=num_of_passes {
        let offset = ((1 << (num_of_passes - i)) as f32 + 0.5).floor() as i32;
        command_buffers.set_buffer_barriers(
          0,
          &[
            hala_gfx::HalaBufferBarrierInfo {
              src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
              dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
              size: get_read_jump_buffer(i).size,
              buffer: get_read_jump_buffer(i).raw,
              ..Default::default()
            },
            hala_gfx::HalaBufferBarrierInfo {
              src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              size: get_write_jump_buffer(i).size,
              buffer: get_write_jump_buffer(i).raw,
              ..Default::default()
            },
          ],
        );

        let program = self.udf_baker_resources.compute_programs.get("jump_flooding")
          .ok_or(HalaRendererError::new("Failed to get the jump flooding compute program.", None))?;
        let descriptor_set = get_jump_flooding_descriptor_set(i);

        program.bind(
          0,
          command_buffers,
          &[
            &self.udf_baker_resources.static_descriptor_set,
            descriptor_set,
          ],
        );

        program.push_constants(
          0,
          command_buffers,
          0,
          &offset.to_le_bytes(),
        );

        program.dispatch(
          0,
          command_buffers,
          (dimensions[0] + 8 - 1) / 8,
          (dimensions[1] + 8 - 1) / 8,
          (dimensions[2] + 8 - 1) / 8,
        );
      }
    }

    // Finalize.
    {
      command_buffers.set_buffer_barriers(
        0,
        &[
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            size: get_write_jump_buffer(num_of_passes).size,
            buffer: get_write_jump_buffer(num_of_passes).raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo{
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            size: get_read_jump_buffer(num_of_passes).size,
            buffer: get_read_jump_buffer(num_of_passes).raw,
            ..Default::default()
          }
        ],
      );

      command_buffers.set_image_barriers(
        0,
        &[
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

      let program = self.udf_baker_resources.compute_programs.get("jump_flooding_final")
        .ok_or(HalaRendererError::new("Failed to get the jump flooding finalize compute program.", None))?;

      program.bind(
        0,
        command_buffers,
        &[
          &self.udf_baker_resources.static_descriptor_set,
          jump_flooding_finalize_descriptor_set,
        ],
      );

      program.dispatch(
        0,
        command_buffers,
        (dimensions[0] + 8 - 1) / 8,
        (dimensions[1] + 8 - 1) / 8,
        (dimensions[2] + 8 - 1) / 8,
      );
    }

    Ok(get_read_jump_buffer(num_of_passes))
  }

}