use std::rc::Rc;

use hala_renderer::error::HalaRendererError;

use crate::baker::SDFBaker;

impl SDFBaker {

  pub(super) fn find_sign_create_buffers_images(
    &mut self,
    dimensions: &[u32; 3],
  ) -> Result<(), HalaRendererError> {
    if let Some(sign_map) = &self.baker_resources.sign_map {
      if sign_map.extent.width != dimensions[0] || sign_map.extent.height != dimensions[1] || sign_map.extent.depth != dimensions[2] {
        self.baker_resources.sign_map = None;
      }
    }
    if self.baker_resources.sign_map.is_none() {
      self.baker_resources.sign_map = Some(
        hala_gfx::HalaImage::new_3d(
          Rc::clone(&self.resources.context.borrow().logical_device),
          hala_gfx::HalaImageUsageFlags::SAMPLED | hala_gfx::HalaImageUsageFlags::STORAGE,
          hala_gfx::HalaFormat::R32_SFLOAT,
          dimensions[0],
          dimensions[1],
          dimensions[2],
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "sign_map.image3d",
        )?
      );
    }

    if let Some(sign_map_bis) = &self.baker_resources.sign_map_bis {
      if sign_map_bis.extent.width != dimensions[0] || sign_map_bis.extent.height != dimensions[1] || sign_map_bis.extent.depth != dimensions[2] {
        self.baker_resources.sign_map_bis = None;
      }
    }
    if self.baker_resources.sign_map_bis.is_none() {
      self.baker_resources.sign_map_bis = Some(
        hala_gfx::HalaImage::new_3d(
          Rc::clone(&self.resources.context.borrow().logical_device),
          hala_gfx::HalaImageUsageFlags::SAMPLED | hala_gfx::HalaImageUsageFlags::STORAGE,
          hala_gfx::HalaFormat::R32_SFLOAT,
          dimensions[0],
          dimensions[1],
          dimensions[2],
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "sign_map_bis.image3d",
        )?
      );
    }

    Ok(())
  }

  pub(super) fn find_sign_update(
    &self,
    ray_map: &hala_gfx::HalaImage,
    sign_map: &hala_gfx::HalaImage,
    sign_map_bis: &hala_gfx::HalaImage,
  ) -> Result<
  (
    &hala_gfx::HalaDescriptorSet,
    &hala_gfx::HalaDescriptorSet,
    &hala_gfx::HalaDescriptorSet,
  ),
    HalaRendererError
  > {
    let sign_pass_6rays_descriptor_set = self.baker_resources.descriptor_sets.get("sign_pass_6rays")
      .ok_or(HalaRendererError::new("Failed to get the sign_pass_6rays descriptor set.", None))?;
    sign_pass_6rays_descriptor_set.update_sampled_images(
      0,
      0,
      &[ray_map],
    );
    sign_pass_6rays_descriptor_set.update_storage_images(
      0,
      1,
      &[sign_map],
    );

    let sign_pass_neighbors_descriptor_set = self.baker_resources.descriptor_sets.get("sign_pass_neighbors")
      .ok_or(HalaRendererError::new("Failed to get the sign_pass_neighbors descriptor set.", None))?;
    sign_pass_neighbors_descriptor_set.update_sampled_images(
      0,
      0,
      &[ray_map],
    );
    sign_pass_neighbors_descriptor_set.update_sampled_images(
      0,
    1,
      &[sign_map_bis],
    );
    sign_pass_neighbors_descriptor_set.update_storage_images(
      0,
      2,
      &[sign_map],
    );

    let sign_pass_neighbors_2_descriptor_set = self.baker_resources.descriptor_sets.get("sign_pass_neighbors_2")
      .ok_or(HalaRendererError::new("Failed to get the sign_pass_neighbors_2 descriptor set.", None))?;
    sign_pass_neighbors_2_descriptor_set.update_sampled_images(
      0,
      0,
      &[ray_map],
    );
    sign_pass_neighbors_2_descriptor_set.update_sampled_images(
      0,
      1,
      &[sign_map],
    );
    sign_pass_neighbors_2_descriptor_set.update_storage_images(
      0,
      2,
      &[sign_map_bis],
    );

    Ok((
      sign_pass_6rays_descriptor_set,
      sign_pass_neighbors_descriptor_set,
      sign_pass_neighbors_2_descriptor_set,
    ))
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn find_sign_compute<'a: 'b, 'b>(
    &self,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    ray_map: &hala_gfx::HalaImage,
    sign_map: &'a hala_gfx::HalaImage,
    sign_map_bis: &'a hala_gfx::HalaImage,
    sign_pass_6rays_descriptor_set: &hala_gfx::HalaDescriptorSet,
    sign_pass_neighbors_descriptor_set: &hala_gfx::HalaDescriptorSet,
    sign_pass_neighbors_2_descriptor_set: &hala_gfx::HalaDescriptorSet,
    dimensions: &[u32; 3],
  ) -> Result<&'b hala_gfx::HalaImage, HalaRendererError> {
    // ray_map be going to be read by compute shaders.
    // sign_map and sign_map_bis be going to be written by compute shaders.
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
            image: ray_map.raw,
            ..Default::default()
          },
          hala_gfx::HalaImageBarrierInfo {
            old_layout: hala_gfx::HalaImageLayout::GENERAL,
            new_layout: hala_gfx::HalaImageLayout::GENERAL,
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
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
            image: sign_map_bis.raw,
            ..Default::default()
          },
        ],
      );
    }

    // First pass.
    {
      let sign_pass_6rays_program = self.baker_resources.compute_programs.get("sign_pass_6rays")
        .ok_or(HalaRendererError::new("Failed to get the sign_pass_6rays program.", None))?;
      sign_pass_6rays_program.bind(
        0,
        command_buffers,
        &[
          &self.baker_resources.static_descriptor_set,
          sign_pass_6rays_descriptor_set,
        ]
      );
      sign_pass_6rays_program.dispatch(
        0,
        command_buffers,
        (dimensions[0] + 4 - 1) / 4,
        (dimensions[1] + 4 - 1) / 4,
        (dimensions[2] + 4 - 1) / 4,
      );
    }

    let get_read_sign_map = |i: i32| -> &hala_gfx::HalaImage {
      if i % 2 == 0 {
        sign_map_bis
      } else {
        sign_map
      }
    };
    let get_write_sign_map = |i: i32| -> &hala_gfx::HalaImage {
      if i % 2 == 0 {
        sign_map
      } else {
        sign_map_bis
      }
    };

    // 2-n passes.
    {
      let sign_pass_neighbors_program = self.baker_resources.compute_programs.get("sign_pass_neighbors")
        .ok_or(HalaRendererError::new("Failed to get the sign_pass_neighbors program.", None))?;

      let num_of_neighnors = 8u32;
      let mut normalize_factor = 6.0f32;

      let get_descriptor_set = |i: i32| -> &hala_gfx::HalaDescriptorSet {
        if i % 2 == 0 {
          sign_pass_neighbors_descriptor_set
        } else {
          sign_pass_neighbors_2_descriptor_set
        }
      };

      for i in 1..=self.settings.sign_passes_count {
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
              image: get_read_sign_map(i).raw,
              ..Default::default()
            },
            hala_gfx::HalaImageBarrierInfo {
              old_layout: hala_gfx::HalaImageLayout::GENERAL,
              new_layout: hala_gfx::HalaImageLayout::GENERAL,
              src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
              image: get_write_sign_map(i).raw,
              ..Default::default()
            },
          ],
        );

        sign_pass_neighbors_program.bind(
          0,
          command_buffers,
          &[
            &self.baker_resources.static_descriptor_set,
            get_descriptor_set(i),
          ]
        );

        let mut push_constants = Vec::new();
        push_constants.extend_from_slice(&normalize_factor.to_le_bytes());
        push_constants.extend_from_slice(&num_of_neighnors.to_le_bytes());
        push_constants.extend_from_slice(&i.to_le_bytes());
        // Only normalize the last pass.
        push_constants.extend_from_slice(&(if i == self.settings.sign_passes_count { 1u32 } else { 0u32 }).to_le_bytes());

        sign_pass_neighbors_program.push_constants(
          0,
          command_buffers,
          0,
          &push_constants,
        );

        sign_pass_neighbors_program.dispatch(
          0,
          command_buffers,
          (dimensions[0] + 4 - 1) / 4,
          (dimensions[1] + 4 - 1) / 4,
          (dimensions[2] + 4 - 1) / 4,
        );

        normalize_factor = normalize_factor + num_of_neighnors as f32 * 6.0 * normalize_factor;
      }
    }

    Ok(get_write_sign_map(self.settings.sign_passes_count))
  }

}
