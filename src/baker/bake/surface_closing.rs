use std::rc::Rc;

use hala_renderer::error::HalaRendererError;

use crate::baker::SDFBaker;

impl SDFBaker {

  pub(super) fn surface_closing_create_buffers_images(
    &mut self,
    dimensions: &[u32; 3],
  ) -> Result<(), HalaRendererError> {
    if let Some(voxels_texture) = &self.sdf_baker_resources.voxels_texture {
      if voxels_texture.extent.width != dimensions[0] || voxels_texture.extent.height != dimensions[1] || voxels_texture.extent.depth != dimensions[2] {
        self.sdf_baker_resources.voxels_texture = None;
      }
    }
    if self.sdf_baker_resources.voxels_texture.is_none() {
      self.sdf_baker_resources.voxels_texture = Some(
        hala_gfx::HalaImage::new_3d(
          Rc::clone(&self.resources.context.borrow().logical_device),
          hala_gfx::HalaImageUsageFlags::SAMPLED | hala_gfx::HalaImageUsageFlags::STORAGE,
          hala_gfx::HalaFormat::R32G32B32A32_SFLOAT,
          dimensions[0],
          dimensions[1],
          dimensions[2],
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "voxels_texture.image3d",
        )?
      );
    }

    if let Some(voxels_texture_bis) = &self.sdf_baker_resources.voxels_texture_bis {
      if voxels_texture_bis.extent.width != dimensions[0] || voxels_texture_bis.extent.height != dimensions[1] || voxels_texture_bis.extent.depth != dimensions[2] {
        self.sdf_baker_resources.voxels_texture_bis = None;
      }
    }
    if self.sdf_baker_resources.voxels_texture_bis.is_none() {
      self.sdf_baker_resources.voxels_texture_bis = Some(
        hala_gfx::HalaImage::new_3d(
          Rc::clone(&self.resources.context.borrow().logical_device),
          hala_gfx::HalaImageUsageFlags::SAMPLED | hala_gfx::HalaImageUsageFlags::STORAGE,
          hala_gfx::HalaFormat::R32G32B32A32_SFLOAT,
          dimensions[0],
          dimensions[1],
          dimensions[2],
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "voxels_texture_bis.image3d",
        )?
      );
    }

    Ok(())
  }

  pub(super) fn surface_closing_update(
    &self,
    voxels_buffer: &hala_gfx::HalaBuffer,
    sign_map: &hala_gfx::HalaImage,
    voxels_texture: &hala_gfx::HalaImage,
    voxels_texture_bis: &hala_gfx::HalaImage,
  ) -> Result<
    (
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
    ),
    HalaRendererError
  > {
    let in_out_edge_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("in_out_edge")
      .ok_or(HalaRendererError::new("Failed to get the in_out_edge descriptor set.", None))?;
    in_out_edge_descriptor_set.update_sampled_images(
      0,
      0,
      &[sign_map],
    );
    in_out_edge_descriptor_set.update_storage_images(
      0,
      1,
      &[voxels_texture],
    );

    let buffer_2_image_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("buffer_2_image")
      .ok_or(HalaRendererError::new("Failed to get the buffer_2_image descriptor set.", None))?;
    buffer_2_image_descriptor_set.update_storage_buffers(
      0,
      0,
      &[voxels_buffer],
    );
    buffer_2_image_descriptor_set.update_storage_images(
      0,
      1,
      &[voxels_texture],
    );

    let jfa_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("jfa")
      .ok_or(HalaRendererError::new("Failed to get the jfa descriptor set.", None))?;
    jfa_descriptor_set.update_sampled_images(
      0,
      0,
      &[voxels_texture],
    );
    jfa_descriptor_set.update_storage_images(
      0,
      1,
      &[voxels_texture_bis],
    );

    let jfa_2_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("jfa_2")
      .ok_or(HalaRendererError::new("Failed to get the jfa_2 descriptor set.", None))?;
    jfa_2_descriptor_set.update_sampled_images(
      0,
      0,
      &[voxels_texture_bis],
    );
    jfa_2_descriptor_set.update_storage_images(
      0,
      1,
      &[voxels_texture],
    );

    Ok((
      in_out_edge_descriptor_set,
      buffer_2_image_descriptor_set,
      jfa_descriptor_set,
      jfa_2_descriptor_set,
    ))
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn surface_closing_compute<'a: 'b, 'b>(
    &self,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    voxels_buffer: &hala_gfx::HalaBuffer,
    sign_map: &hala_gfx::HalaImage,
    voxels_texture: &'a hala_gfx::HalaImage,
    voxels_texture_bis: &'a hala_gfx::HalaImage,
    in_out_edge_descriptor_set: &hala_gfx::HalaDescriptorSet,
    buffer_2_image_descriptor_set: &hala_gfx::HalaDescriptorSet,
    jfa_descriptor_set: &hala_gfx::HalaDescriptorSet,
    jfa_2_descriptor_set: &hala_gfx::HalaDescriptorSet,
    dimensions: &[u32; 3],
  ) -> Result<&'b hala_gfx::HalaImage, HalaRendererError> {
    // sign_map be going to be read by compute shaders.
    // voxels_texture be going to be written by compute shaders.
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
            image: sign_map.raw,
            ..Default::default()
          },
          hala_gfx::HalaImageBarrierInfo {
            old_layout: hala_gfx::HalaImageLayout::GENERAL,
            new_layout: hala_gfx::HalaImageLayout::GENERAL,
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
            image: voxels_texture.raw,
            ..Default::default()
          },
        ],
      );
    }

    // Find in out edge by threshold.
    {
      let program = self.sdf_baker_resources.compute_programs.get("in_out_edge")
        .ok_or(HalaRendererError::new("Failed to get the in_out_edge compute program.", None))?;

      let threshold = if self.settings.sign_passes_count == 0 {
        self.settings.in_out_threshold * 6.0
      } else {
        self.settings.in_out_threshold
      };

      program.bind(
        0,
        command_buffers,
        &[
          &self.sdf_baker_resources.static_descriptor_set,
          in_out_edge_descriptor_set,
        ],
      );

      program.push_constants(
        0,
        command_buffers,
        0,
        &threshold.to_le_bytes(),
      );

      program.dispatch(
        0,
        command_buffers,
        (dimensions[0] + 4 - 1) / 4,
        (dimensions[1] + 4 - 1) / 4,
        (dimensions[2] + 4 - 1) / 4,
      );
    }

    // voxels_buffer be going to be read by compute shaders.
    // voxels_texture be going to be written by compute shaders.
    {
      command_buffers.set_buffer_barriers(
        0,
        &[
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            size: voxels_buffer.size,
            buffer: voxels_buffer.raw,
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
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
            image: voxels_texture.raw,
            ..Default::default()
          },
        ],
      );
    }

    // Buffer to texture.
    {
      let program = self.sdf_baker_resources.compute_programs.get("buffer_2_image")
        .ok_or(HalaRendererError::new("Failed to get the buffer_2_image compute program.", None))?;

      program.bind(
        0,
        command_buffers,
        &[
          &self.sdf_baker_resources.static_descriptor_set,
          buffer_2_image_descriptor_set,
        ],
      );

      program.dispatch(
        0,
        command_buffers,
        (dimensions[0] + 4 - 1) / 4,
        (dimensions[1] + 4 - 1) / 4,
        (dimensions[2] + 4 - 1) / 4,
      );
    }

    // voxels_texture be going to be read by compute shaders.
    // voxels_texture_bis be going to be written by compute shaders.
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
            image: voxels_texture_bis.raw,
            ..Default::default()
          },
        ],
      );
    }

    // JFA pass 0.
    {
      let program = self.sdf_baker_resources.compute_programs.get("jfa")
        .ok_or(HalaRendererError::new("Failed to get the jfa compute program.", None))?;

      program.bind(
        0,
        command_buffers,
        &[
          &self.sdf_baker_resources.static_descriptor_set,
          jfa_descriptor_set,
        ],
      );

      program.push_constants(
        0,
        command_buffers,
        0,
        &1i32.to_le_bytes(),
      );

      program.dispatch(
        0,
        command_buffers,
        (dimensions[0] + 4 - 1) / 4,
        (dimensions[1] + 4 - 1) / 4,
        (dimensions[2] + 4 - 1) / 4,
      );
    }

    let get_read_voxels_texture = |i: u32| -> &hala_gfx::HalaImage {
      if i % 2 == 0 {
        voxels_texture
      } else {
        voxels_texture_bis
      }
    };
    let get_write_voxels_texture = |i: u32| -> &hala_gfx::HalaImage {
      if i % 2 == 0 {
        voxels_texture_bis
      } else {
        voxels_texture
      }
    };

    // JFA 1 to N passes.
    let num_of_steps = self.settings.max_resolution.ilog2();
    {
      let program = self.sdf_baker_resources.compute_programs.get("jfa")
        .ok_or(HalaRendererError::new("Failed to get the jfa compute program.", None))?;

      let get_jfa_descriptor_set = |i: u32| -> &hala_gfx::HalaDescriptorSet {
        if i % 2 == 0 {
          jfa_descriptor_set
        } else {
          jfa_2_descriptor_set
        }
      };

      for level in 1..=num_of_steps {
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
              image: get_read_voxels_texture(level).raw,
              ..Default::default()
            },
            hala_gfx::HalaImageBarrierInfo {
              old_layout: hala_gfx::HalaImageLayout::GENERAL,
              new_layout: hala_gfx::HalaImageLayout::GENERAL,
              src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
              image: get_write_voxels_texture(level).raw,
              ..Default::default()
            },
          ],
        );

        program.bind(
          0,
          command_buffers,
          &[
            &self.sdf_baker_resources.static_descriptor_set,
            get_jfa_descriptor_set(level),
          ],
        );

        let offset = (1 << (num_of_steps - level)) as u32;
        program.push_constants(
          0,
          command_buffers,
          0,
          &offset.to_le_bytes(),
        );

        program.dispatch(
          0,
          command_buffers,
          (dimensions[0] + 4 - 1) / 4,
          (dimensions[1] + 4 - 1) / 4,
          (dimensions[2] + 4 - 1) / 4,
        );
      }
    }

    Ok(get_write_voxels_texture(num_of_steps))
  }

}