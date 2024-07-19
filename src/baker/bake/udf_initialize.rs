use std::rc::Rc;

use hala_renderer::error::HalaRendererError;

use crate::baker::SDFBaker;

impl SDFBaker {

  pub(super) fn udf_initialize_create_buffers_images(
    &mut self,
    num_of_voxels: u32,
    dimensions: &[u32; 3],
  ) -> Result<(), HalaRendererError> {
    if let Some(distance_texture) = &self.udf_baker_resources.distance_texture {
      if distance_texture.extent.width != dimensions[0] || distance_texture.extent.height != dimensions[1] || distance_texture.extent.depth != dimensions[2] {
        self.udf_baker_resources.distance_texture = None;
      }
    }
    if self.udf_baker_resources.distance_texture.is_none() {
      self.udf_baker_resources.distance_texture = Some(
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

    let jump_buffer_size = num_of_voxels as u64 * std::mem::size_of::<f32>() as u64;
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
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "jump_buffer.buffer",
        )?
      );
    };

    let jump_buffer_bis_size = num_of_voxels as u64 * std::mem::size_of::<f32>() as u64;
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
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "jump_buffer_bis.buffer",
        )?
      );
    };

    Ok(())
  }

  pub(super) fn udf_initialize_update(
    &self,
    distance_texture: &hala_gfx::HalaImage,
  ) -> Result<
    &hala_gfx::HalaDescriptorSet,
    HalaRendererError
  > {
    let descriptor_set = self.udf_baker_resources.descriptor_sets.get("udf_init")
      .ok_or(HalaRendererError::new("Failed to get the initialize descriptor set.", None))?;
    descriptor_set.update_storage_images(
      0,
      0,
      &[distance_texture],
    );

    Ok(descriptor_set)
  }

  pub(super) fn udf_initialize_compute(
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
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          image: distance_texture.raw,
          ..Default::default()
        },
      ],
    );

    let program = self.udf_baker_resources.compute_programs.get("udf_init")
      .ok_or(HalaRendererError::new("Failed to get the initialize program.", None))?;
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
