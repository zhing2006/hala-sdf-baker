use std::rc::Rc;

use hala_renderer::error::HalaRendererError;

use crate::baker::SDFBaker;
use crate::baker::sdf_resources::SDFBakerResources;

impl SDFBaker {

  pub(super) fn prefix_sum_create_buffers_images(
    &mut self,
    num_of_voxels: u32,
  ) -> Result<(), HalaRendererError> {
    let voxels_buffer_size = num_of_voxels as u64 * std::mem::size_of::<[f32; 4]>() as u64;
    if let Some(voxels_buffer) = &self.sdf_baker_resources.voxels_buffer {
      if voxels_buffer.size != voxels_buffer_size {
        self.sdf_baker_resources.voxels_buffer = None;
      }
    }
    if self.sdf_baker_resources.voxels_buffer.is_none() {
      self.sdf_baker_resources.voxels_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          voxels_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "voxels.buffer",
        )?
      );
    };

    let counters_buffer_size = num_of_voxels as u64 * std::mem::size_of::<u32>() as u64;
    if let Some(counters_buffer) = &self.sdf_baker_resources.counters_buffer {
      if counters_buffer.size != counters_buffer_size {
        self.sdf_baker_resources.counters_buffer = None;
      }
    }
    if self.sdf_baker_resources.counters_buffer.is_none() {
      self.sdf_baker_resources.counters_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          counters_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "counters.buffer",
        )?
      );
    };

    let in_sum_blocks_buffer_size =
      ((num_of_voxels + SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE - 1) / SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE) as u64 *
      std::mem::size_of::<u32>() as u64;
    if let Some(in_sum_blocks_buffer) = &self.sdf_baker_resources.in_sum_blocks_buffer {
      if in_sum_blocks_buffer.size != in_sum_blocks_buffer_size {
        self.sdf_baker_resources.in_sum_blocks_buffer = None;
      }
    }
    if self.sdf_baker_resources.in_sum_blocks_buffer.is_none() {
      self.sdf_baker_resources.in_sum_blocks_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          in_sum_blocks_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "in_sum_blocks.buffer",
        )?
      );
    };

    let sum_blocks_buffer_size =
      ((num_of_voxels + SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE - 1) / SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE) as u64 *
      std::mem::size_of::<u32>() as u64;
    if let Some(sum_blocks_buffer) = &self.sdf_baker_resources.sum_blocks_buffer {
      if sum_blocks_buffer.size != sum_blocks_buffer_size {
        self.sdf_baker_resources.sum_blocks_buffer = None;
      }
    }
    if self.sdf_baker_resources.sum_blocks_buffer.is_none() {
      self.sdf_baker_resources.sum_blocks_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          sum_blocks_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "sum_blocks.buffer",
        )?
      );
    };

    let accum_sum_blocks_buffer_size =
      ((num_of_voxels + SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE - 1) / SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE) as u64 *
      std::mem::size_of::<u32>() as u64;
    if let Some(accum_sum_blocks_buffer) = &self.sdf_baker_resources.accum_sum_blocks_buffer {
      if accum_sum_blocks_buffer.size != accum_sum_blocks_buffer_size {
        self.sdf_baker_resources.accum_sum_blocks_buffer = None;
      }
    }
    if self.sdf_baker_resources.accum_sum_blocks_buffer.is_none() {
      self.sdf_baker_resources.accum_sum_blocks_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          accum_sum_blocks_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "accum_sum_blocks.buffer",
        )?
      );
    };

    let accum_counters_buffer_size = num_of_voxels as u64 * std::mem::size_of::<u32>() as u64;
    if let Some(accum_counters_buffer) = &self.sdf_baker_resources.accum_counters_buffer {
      if accum_counters_buffer.size != accum_counters_buffer_size {
        self.sdf_baker_resources.accum_counters_buffer = None;
      }
    }
    if self.sdf_baker_resources.accum_counters_buffer.is_none() {
      self.sdf_baker_resources.accum_counters_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          accum_counters_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "accum_counters.buffer",
        )?
      );
    };

    let tmp_buffer_size = num_of_voxels as u64 * std::mem::size_of::<u32>() as u64;
    if let Some(tmp_buffer) = &self.sdf_baker_resources.tmp_buffer {
      if tmp_buffer.size != tmp_buffer_size {
        self.sdf_baker_resources.tmp_buffer = None;
      }
    }
    if self.sdf_baker_resources.tmp_buffer.is_none() {
      self.sdf_baker_resources.tmp_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          tmp_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "tmp.buffer",
        )?
      );
    };

    let sq_thread_group_size = SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE * SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE;
    let additional_sum_blocks_buffer_size =
      ((num_of_voxels + sq_thread_group_size - 1) / sq_thread_group_size) as u64 *
      std::mem::size_of::<u32>() as u64;
    if let Some(additional_sum_blocks_buffer) = &self.sdf_baker_resources.additional_sum_blocks_buffer {
      if additional_sum_blocks_buffer.size != additional_sum_blocks_buffer_size {
        self.sdf_baker_resources.additional_sum_blocks_buffer = None;
      }
    }
    if self.sdf_baker_resources.additional_sum_blocks_buffer.is_none() {
      self.sdf_baker_resources.additional_sum_blocks_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          additional_sum_blocks_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "additional_sum_blocks.buffer",
        )?
      );
    };

    Ok(())
  }

  #[allow(clippy::too_many_arguments, clippy::type_complexity)]
  pub(super) fn prefix_sum_update(
    &self,
    counters_buffer: &hala_gfx::HalaBuffer,
    tmp_buffer: &hala_gfx::HalaBuffer,
    sum_blocks_buffer: &hala_gfx::HalaBuffer,
    in_sum_blocks_buffer: &hala_gfx::HalaBuffer,
    additional_sum_blocks_buffer: &hala_gfx::HalaBuffer,
    accum_sum_blocks_buffer: &hala_gfx::HalaBuffer,
    accum_counters_buffer: &hala_gfx::HalaBuffer,
  ) -> Result<
    (
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
    ),
    HalaRendererError
  > {
    let in_bucket_sum_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("in_bucket_sum")
      .ok_or(HalaRendererError::new("Failed to get the in_bucket_sum descriptor set.", None))?;
    in_bucket_sum_descriptor_set.update_storage_buffers(
      0,
      0,
      &[counters_buffer],
    );
    in_bucket_sum_descriptor_set.update_storage_buffers(
      0,
      1,
      &[tmp_buffer],
    );
    let block_sum_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("block_sum")
      .ok_or(HalaRendererError::new("Failed to get the block_sum descriptor set.", None))?;
    block_sum_descriptor_set.update_storage_buffers(
      0,
      0,
      &[tmp_buffer],
    );
    block_sum_descriptor_set.update_storage_buffers(
      0,
      1,
      &[accum_sum_blocks_buffer],
    );
    block_sum_descriptor_set.update_storage_buffers(
      0,
      2,
      &[counters_buffer],
    );
    let final_sum_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("final_sum")
      .ok_or(HalaRendererError::new("Failed to get the final_sum descriptor set.", None))?;
    final_sum_descriptor_set.update_storage_buffers(
      0,
      0,
      &[tmp_buffer],
    );
    final_sum_descriptor_set.update_storage_buffers(
      0,
      1,
      &[accum_counters_buffer],
    );
    final_sum_descriptor_set.update_storage_buffers(
      0,
      2,
      &[counters_buffer],
    );
    final_sum_descriptor_set.update_storage_buffers(
      0,
      3,
      &[accum_sum_blocks_buffer],
    );
    let to_block_sum_buffer_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("to_block_sum_buffer")
      .ok_or(HalaRendererError::new("Failed to get the to_block_sum_buffer descriptor set.", None))?;
    to_block_sum_buffer_descriptor_set.update_storage_buffers(
      0,
      0,
      &[tmp_buffer],
    );
    to_block_sum_buffer_descriptor_set.update_storage_buffers(
      0,
      1,
      &[sum_blocks_buffer],
    );
    to_block_sum_buffer_descriptor_set.update_storage_buffers(
      0,
      2,
      &[counters_buffer],
    );

    let in_bucket_sum_2_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("in_bucket_sum_2")
      .ok_or(HalaRendererError::new("Failed to get the in_bucket_sum_2 descriptor set.", None))?;
    in_bucket_sum_2_descriptor_set.update_storage_buffers(
      0,
      0,
      &[sum_blocks_buffer],
    );
    in_bucket_sum_2_descriptor_set.update_storage_buffers(
      0,
      1,
      &[in_sum_blocks_buffer],
    );
    let block_sum_2_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("block_sum_2")
      .ok_or(HalaRendererError::new("Failed to get the block_sum_2 descriptor set.", None))?;
    block_sum_2_descriptor_set.update_storage_buffers(
      0,
      0,
      &[in_sum_blocks_buffer],
    );
    block_sum_2_descriptor_set.update_storage_buffers(
      0,
      1,
      &[additional_sum_blocks_buffer],
    );
    block_sum_2_descriptor_set.update_storage_buffers(
      0,
      2,
      &[sum_blocks_buffer],
    );
    let final_sum_2_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("final_sum_2")
      .ok_or(HalaRendererError::new("Failed to get the final_sum_2 descriptor set.", None))?;
    final_sum_2_descriptor_set.update_storage_buffers(
      0,
      0,
      &[in_sum_blocks_buffer],
    );
    final_sum_2_descriptor_set.update_storage_buffers(
      0,
      1,
      &[accum_sum_blocks_buffer],
    );
    final_sum_2_descriptor_set.update_storage_buffers(
      0,
      2,
      &[sum_blocks_buffer],
    );
    final_sum_2_descriptor_set.update_storage_buffers(
      0,
      3,
      &[additional_sum_blocks_buffer],
    );

    Ok((
      in_bucket_sum_descriptor_set,
      block_sum_descriptor_set,
      final_sum_descriptor_set,
      to_block_sum_buffer_descriptor_set,
      in_bucket_sum_2_descriptor_set,
      block_sum_2_descriptor_set,
      final_sum_2_descriptor_set,
    ))
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn prefix_sum_compute(
    &self,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    voxels_buffer: &hala_gfx::HalaBuffer,
    counters_buffer: &hala_gfx::HalaBuffer,
    tmp_buffer: &hala_gfx::HalaBuffer,
    sum_blocks_buffer: &hala_gfx::HalaBuffer,
    in_sum_blocks_buffer: &hala_gfx::HalaBuffer,
    additional_sum_blocks_buffer: &hala_gfx::HalaBuffer,
    accum_sum_blocks_buffer: &hala_gfx::HalaBuffer,
    accum_counters_buffer: &hala_gfx::HalaBuffer,
    in_bucket_sum_descriptor_set: &hala_gfx::HalaDescriptorSet,
    block_sum_descriptor_set: &hala_gfx::HalaDescriptorSet,
    final_sum_descriptor_set: &hala_gfx::HalaDescriptorSet,
    to_block_sum_buffer_descriptor_set: &hala_gfx::HalaDescriptorSet,
    in_bucket_sum_2_descriptor_set: &hala_gfx::HalaDescriptorSet,
    block_sum_2_descriptor_set: &hala_gfx::HalaDescriptorSet,
    final_sum_2_descriptor_set: &hala_gfx::HalaDescriptorSet,
    num_of_voxels: u32,
  ) -> Result<(), HalaRendererError> {
    // voxels_buffer and counters_buffer be going to be read by compute shaders.
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
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            size: counters_buffer.size,
            buffer: counters_buffer.raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            size: tmp_buffer.size,
            buffer: tmp_buffer.raw,
            ..Default::default()
          },
        ],
      );
    }

    // Prefix sum.
    {
      let dispatch_size = SDFBaker::get_prefix_sum_dispatch_size(num_of_voxels);
      let in_bucket_sum_program = self.sdf_baker_resources.compute_programs.get("in_bucket_sum")
        .ok_or(HalaRendererError::new("Failed to get the in_bucket_sum program.", None))?;

      in_bucket_sum_program.bind(
        0,
        command_buffers,
        &[
          &self.sdf_baker_resources.static_descriptor_set,
          in_bucket_sum_descriptor_set
        ],
      );

      let mut push_constants = Vec::new();
      push_constants.extend_from_slice(&dispatch_size.0.to_le_bytes());
      push_constants.extend_from_slice(&num_of_voxels.to_le_bytes());
      push_constants.extend_from_slice(&0u32.to_le_bytes());
      in_bucket_sum_program.push_constants(
        0,
        command_buffers,
        0,
        &push_constants,
      );

      in_bucket_sum_program.dispatch(
        0,
        command_buffers,
        dispatch_size.0,
        dispatch_size.1,
        1,
      );

      command_buffers.set_buffer_barriers(
        0,
        &[
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            size: tmp_buffer.size,
            buffer: tmp_buffer.raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            size: accum_sum_blocks_buffer.size,
            buffer: accum_sum_blocks_buffer.raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            size: sum_blocks_buffer.size,
            buffer: sum_blocks_buffer.raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            size: in_sum_blocks_buffer.size,
            buffer: in_sum_blocks_buffer.raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            size: additional_sum_blocks_buffer.size,
            buffer: additional_sum_blocks_buffer.raw,
            ..Default::default()
          },
        ],
      );

      let sq_thread_group_size = SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE * SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE;
      let num_of_blocks = (num_of_voxels + SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE - 1) / SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE;
      if num_of_blocks > SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE {
        {
          let to_block_sum_buffer_program = self.sdf_baker_resources.compute_programs.get("to_block_sum_buffer")
            .ok_or(HalaRendererError::new("Failed to get the to_block_sum_buffer program.", None))?;

          to_block_sum_buffer_program.bind(
            0,
            command_buffers,
            &[
              &self.sdf_baker_resources.static_descriptor_set,
              to_block_sum_buffer_descriptor_set,
            ],
          );

          let mut push_constants = Vec::new();
          push_constants.extend_from_slice(&0u32.to_le_bytes());
          push_constants.extend_from_slice(&num_of_voxels.to_le_bytes());
          push_constants.extend_from_slice(&0u32.to_le_bytes());
          to_block_sum_buffer_program.push_constants(
            0,
            command_buffers,
            0,
            &push_constants,
          );

          to_block_sum_buffer_program.dispatch(
            0,
            command_buffers,
            (num_of_voxels + sq_thread_group_size - 1) / sq_thread_group_size,
            1,
            1,
          );
        }

        {
          command_buffers.set_buffer_barriers(
            0,
            &[
              hala_gfx::HalaBufferBarrierInfo {
                src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
                src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
                dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
                dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
                size: sum_blocks_buffer.size,
                buffer: sum_blocks_buffer.raw,
                ..Default::default()
              },
            ],
          );

          in_bucket_sum_program.bind(
            0,
            command_buffers,
            &[
              &self.sdf_baker_resources.static_descriptor_set,
              in_bucket_sum_2_descriptor_set,
            ],
          );

          let mut push_constants = Vec::new();
          push_constants.extend_from_slice(&dispatch_size.0.to_le_bytes());
          push_constants.extend_from_slice(&num_of_blocks.to_le_bytes());
          push_constants.extend_from_slice(&0u32.to_le_bytes());
          in_bucket_sum_program.push_constants(
            0,
            command_buffers,
            0,
            &push_constants,
          );

          in_bucket_sum_program.dispatch(
            0,
            command_buffers,
            (num_of_voxels + sq_thread_group_size - 1) / sq_thread_group_size,
            1,
            1,
          );
        }

        {
          command_buffers.set_buffer_barriers(
            0,
            &[
              hala_gfx::HalaBufferBarrierInfo {
                src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
                src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
                dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
                dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
                size: in_sum_blocks_buffer.size,
                buffer: in_sum_blocks_buffer.raw,
                ..Default::default()
              },
            ],
          );

          let cb_thread_group_size = sq_thread_group_size * SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE;
          let block_sum_program = self.sdf_baker_resources.compute_programs.get("block_sum")
            .ok_or(HalaRendererError::new("Failed to get the block_sum program.", None))?;

          block_sum_program.bind(
            0,
            command_buffers,
            &[
              &self.sdf_baker_resources.static_descriptor_set,
              block_sum_2_descriptor_set,
            ],
          );

          let mut push_constants = Vec::new();
          push_constants.extend_from_slice(&0u32.to_le_bytes());
          push_constants.extend_from_slice(&num_of_blocks.to_le_bytes());
          push_constants.extend_from_slice(&0u32.to_le_bytes());
          block_sum_program.push_constants(
            0,
            command_buffers,
            0,
            &push_constants,
          );

          block_sum_program.dispatch(
            0,
            command_buffers,
            (num_of_voxels + cb_thread_group_size - 1) / cb_thread_group_size,
            1,
            1,
          );
        }

        {
          command_buffers.set_buffer_barriers(
            0,
            &[
              hala_gfx::HalaBufferBarrierInfo {
                src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
                src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
                dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
                dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
                size: additional_sum_blocks_buffer.size,
                buffer: additional_sum_blocks_buffer.raw,
                ..Default::default()
              },
            ],
          );

          let final_sum_program = self.sdf_baker_resources.compute_programs.get("final_sum")
            .ok_or(HalaRendererError::new("Failed to get the final_sum program.", None))?;

          final_sum_program.bind(
            0,
            command_buffers,
            &[
              &self.sdf_baker_resources.static_descriptor_set,
              final_sum_2_descriptor_set,
            ],
          );

          let mut push_constants = Vec::new();
          push_constants.extend_from_slice(&dispatch_size.0.to_le_bytes());
          push_constants.extend_from_slice(&num_of_blocks.to_le_bytes());
          push_constants.extend_from_slice(&0u32.to_le_bytes());
          final_sum_program.push_constants(
            0,
            command_buffers,
            0,
            &push_constants,
          );

          final_sum_program.dispatch(
            0,
            command_buffers,
            (num_of_voxels + sq_thread_group_size - 1) / sq_thread_group_size,
            1,
            1,
          );
        }
      } else {
        let block_sum_program = self.sdf_baker_resources.compute_programs.get("block_sum")
          .ok_or(HalaRendererError::new("Failed to get the block_sum program.", None))?;

        block_sum_program.bind(
          0,
          command_buffers,
          &[
            &self.sdf_baker_resources.static_descriptor_set,
            block_sum_descriptor_set,
          ],
        );

        let mut push_constants = Vec::new();
        push_constants.extend_from_slice(&0u32.to_le_bytes());
        push_constants.extend_from_slice(&num_of_voxels.to_le_bytes());
        push_constants.extend_from_slice(&0u32.to_le_bytes());
        block_sum_program.push_constants(
          0,
          command_buffers,
          0,
          &push_constants,
        );

        block_sum_program.dispatch(
          0,
          command_buffers,
          (num_of_voxels + sq_thread_group_size - 1) / sq_thread_group_size,
          1,
          1,
        );
      }

      command_buffers.set_buffer_barriers(
        0,
        &[
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            size: accum_sum_blocks_buffer.size,
            buffer: accum_sum_blocks_buffer.raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            size: accum_counters_buffer.size,
            buffer: accum_counters_buffer.raw,
            ..Default::default()
          },
        ],
      );

      let final_sum_program = self.sdf_baker_resources.compute_programs.get("final_sum")
        .ok_or(HalaRendererError::new("Failed to get the final_sum program.", None))?;

      final_sum_program.bind(
        0,
        command_buffers,
        &[
          &self.sdf_baker_resources.static_descriptor_set,
          final_sum_descriptor_set,
        ],
      );

      let mut push_constants = Vec::new();
      push_constants.extend_from_slice(&dispatch_size.0.to_le_bytes());
      push_constants.extend_from_slice(&num_of_voxels.to_le_bytes());
      push_constants.extend_from_slice(&0u32.to_le_bytes());
      final_sum_program.push_constants(
        0,
        command_buffers,
        0,
        &push_constants,
      );

      final_sum_program.dispatch(
        0,
        command_buffers,
        dispatch_size.0,
        dispatch_size.1,
        1,
      );
    }

    Ok(())
  }

}