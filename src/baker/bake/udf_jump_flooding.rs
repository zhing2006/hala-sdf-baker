use std::rc::Rc;

use hala_renderer::error::HalaRendererError;

use crate::baker::SDFBaker;

impl SDFBaker {

  pub(super) fn jump_flooding_create_buffers_images(
    &mut self,
    num_of_voxels: u32,
  ) -> Result<(), HalaRendererError> {
    let jump_buffer_size = num_of_voxels as u64 * std::mem::size_of::<u32>() as u64;
    if let Some(jump_buffer) = &self.udf_baker_resources.jump_buffer {
      if jump_buffer.size != jump_buffer_size {
        self.udf_baker_resources.jump_buffer = None;
      }
    }
    if self.udf_baker_resources.jump_buffer.is_none() {
      self.udf_baker_resources.jump_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          jump_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "jump_buffer.buffer",
        )?
      );
    };

    let jump_buffer_bis_size = num_of_voxels as u64 * std::mem::size_of::<u32>() as u64;
    if let Some(jump_buffer_bis) = &self.udf_baker_resources.jump_buffer_bis {
      if jump_buffer_bis.size != jump_buffer_bis_size {
        self.udf_baker_resources.jump_buffer_bis = None;
      }
    }
    if self.udf_baker_resources.jump_buffer_bis.is_none() {
      self.udf_baker_resources.jump_buffer_bis = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          jump_buffer_bis_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "jump_buffer_bis.buffer",
        )?
      );
    };

    Ok(())
  }

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
    let num_of_steps = self.settings.max_resolution.ilog2();
    let get_read_jump_buffer = |i: u32| {
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
      &[jump_buffer],
    );
    jump_flooding_odd_descriptor_set.update_storage_images(
      0,
      1,
      &[distance_texture],
    );
    jump_flooding_odd_descriptor_set.update_storage_buffers(
      0,
      2,
      &[jump_buffer_bis],
    );
    let jump_flooding_even_descriptor_set = self.udf_baker_resources.descriptor_sets.get("jump_flooding_2")
      .ok_or(HalaRendererError::new("Failed to get the jump_flooding_2 descriptor set.", None))?;
    jump_flooding_even_descriptor_set.update_storage_buffers(
      0,
      0,
      &[jump_buffer_bis],
    );
    jump_flooding_even_descriptor_set.update_storage_images(
      0,
      1,
      &[distance_texture],
    );
    jump_flooding_even_descriptor_set.update_storage_buffers(
      0,
      2,
      &[jump_buffer],
    );
    let jump_flooding_finalize_descriptor_set = self.udf_baker_resources.descriptor_sets.get("jump_flooding_final")
      .ok_or(HalaRendererError::new("Failed to get the jump flooding finalize descriptor set.", None))?;
    jump_flooding_finalize_descriptor_set.update_storage_images(
      0,
      0,
      &[distance_texture],
    );
    jump_flooding_finalize_descriptor_set.update_storage_buffers(
      0,
      1,
      &[get_read_jump_buffer(num_of_steps)],
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

    let num_of_steps = self.settings.max_resolution.ilog2();
    let get_read_jump_buffer = |i| {
      if i % 2 == 0 {
        jump_buffer_bis
      } else {
        jump_buffer
      }
    };
    let get_write_jump_buffer = |i| {
      if i % 2 == 0 {
        jump_buffer
      } else {
        jump_buffer_bis
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
      for i in 1..=num_of_steps {
        let offset = ((1 << (num_of_steps - i)) as f32 + 0.5).floor() as i32;
        let read_buffer = get_read_jump_buffer(i);
        let write_buffer = get_write_jump_buffer(i);
        command_buffers.set_buffer_barriers(
          0,
          &[
            hala_gfx::HalaBufferBarrierInfo {
              src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
              dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
              size: read_buffer.size,
              buffer: read_buffer.raw,
              ..Default::default()
            },
            hala_gfx::HalaBufferBarrierInfo {
              src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              size: write_buffer.size,
              buffer: write_buffer.raw,
              ..Default::default()
            },
          ],
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
              image: distance_texture.raw,
              ..Default::default()
            }
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
      let write_buffer = get_read_jump_buffer(num_of_steps);
      command_buffers.set_buffer_barriers(
        0,
        &[
          hala_gfx::HalaBufferBarrierInfo{
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            size: write_buffer.size,
            buffer: write_buffer.raw,
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
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ | hala_gfx::HalaAccessFlags2::SHADER_WRITE,
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

      program.push_constants_f32(
        0,
        command_buffers,
        0,
        &[self.settings.surface_offset],
      );

      program.dispatch(
        0,
        command_buffers,
        (dimensions[0] + 8 - 1) / 8,
        (dimensions[1] + 8 - 1) / 8,
        (dimensions[2] + 8 - 1) / 8,
      );
    }

    Ok(get_read_jump_buffer(num_of_steps))
  }

}