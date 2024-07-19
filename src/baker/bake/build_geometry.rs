use std::rc::Rc;

use hala_renderer::error::HalaRendererError;
use hala_renderer::scene::bounds::HalaBounds;
use hala_renderer::graphics_program::HalaGraphicsProgram;

use crate::baker::SDFBaker;

use crate::baker::sdf_resources::{
  SDFBakerCSMeshUniform,
  SDFBakerCSConservativeRasterizationUniform,
};

impl SDFBaker {

  pub(super) fn build_geometry_create_buffers_images(
    &mut self,
    num_of_triangles: u32,
    dimensions: &[u32; 3],
    upper_bound_count: u32,
  ) -> Result<(), HalaRendererError> {
    let triangle_uvw_buffer_size = (num_of_triangles * 3 * std::mem::size_of::<[f32; 4]>() as u32) as u64;
    if let Some(triangle_uvw_buffer) = &self.sdf_baker_resources.triangle_uvw_buffer {
      if triangle_uvw_buffer.size != triangle_uvw_buffer_size {
        self.sdf_baker_resources.triangle_uvw_buffer = None;
      }
    }
    if self.sdf_baker_resources.triangle_uvw_buffer.is_none() {
      self.sdf_baker_resources.triangle_uvw_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          triangle_uvw_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "triangle_uvw.buffer",
        )?
      );
    };

    let coord_flip_buffer_size = (num_of_triangles * std::mem::size_of::<u32>() as u32) as u64;
    if let Some(coord_flip_buffer) = &self.sdf_baker_resources.coord_flip_buffer {
      if coord_flip_buffer.size != coord_flip_buffer_size {
        self.sdf_baker_resources.coord_flip_buffer = None;
      }
    }
    if self.sdf_baker_resources.coord_flip_buffer.is_none() {
      self.sdf_baker_resources.coord_flip_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          coord_flip_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "coord_flip.buffer",
        )?
      );
    };

    let aabb_buffer_size = (num_of_triangles * std::mem::size_of::<[f32; 4]>() as u32) as u64;
    if let Some(aabb_buffer) = &self.sdf_baker_resources.aabb_buffer {
      if aabb_buffer.size != aabb_buffer_size {
        self.sdf_baker_resources.aabb_buffer = None;
      }
    }
    if self.sdf_baker_resources.aabb_buffer.is_none() {
      self.sdf_baker_resources.aabb_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          aabb_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "aabb.buffer",
        )?
      );
    };

    let vertices_buffer_size = (num_of_triangles * 3 * std::mem::size_of::<[f32; 4]>() as u32) as u64;
    if let Some(vertices_buffer) = &self.sdf_baker_resources.vertices_buffer {
      if vertices_buffer.size != vertices_buffer_size {
        self.sdf_baker_resources.vertices_buffer = None;
      }
    }
    if self.sdf_baker_resources.vertices_buffer.is_none() {
      self.sdf_baker_resources.vertices_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          vertices_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::VERTEX_BUFFER,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "vertices.buffer",
        )?
      );
      self.sdf_baker_resources.num_of_triangles = num_of_triangles;
    };

    let triangles_in_voxels_buffer_size = (upper_bound_count * std::mem::size_of::<u32>() as u32) as u64;
    if let Some(triangles_in_voxels_buffer) = &self.sdf_baker_resources.triangles_in_voxels_buffer {
      if triangles_in_voxels_buffer.size != triangles_in_voxels_buffer_size {
        self.sdf_baker_resources.triangles_in_voxels_buffer = None;
      }
    }
    if self.sdf_baker_resources.triangles_in_voxels_buffer.is_none() {
      self.sdf_baker_resources.triangles_in_voxels_buffer = Some(
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          triangles_in_voxels_buffer_size,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "triangles_in_voxels.buffer",
        )?
      );
    };

    let (width, height) = (
      [dimensions[0], dimensions[2], dimensions[1]],
      [dimensions[1], dimensions[0], dimensions[2]],
    );
    for i in 0..3 {
      if let Some(render_target) = &self.sdf_baker_resources.render_targets[i] {
        if render_target.extent.width != width[i] || render_target.extent.height != height[i] {
          self.sdf_baker_resources.render_targets[i] = None;
          self.sdf_baker_resources.write_uvw_and_coverage_programs[i] = None;
          self.sdf_baker_resources.write_triangle_ids_to_voxels_programs[i] = None;
        }
      }
      if self.sdf_baker_resources.render_targets[i].is_none() {
        self.sdf_baker_resources.render_targets[i] = Some(
          hala_gfx::HalaImage::new_2d(
            Rc::clone(&self.resources.context.borrow().logical_device),
            hala_gfx::HalaImageUsageFlags::COLOR_ATTACHMENT | hala_gfx::HalaImageUsageFlags::SAMPLED,
            hala_gfx::HalaFormat::R8G8B8A8_SRGB,
            width[i],
            height[i],
            1,
            1,
            hala_gfx::HalaMemoryLocation::GpuOnly,
            &format!("3_ways_{}.render_target", i),
          )?
        );
      }
      if self.sdf_baker_resources.write_uvw_and_coverage_programs[i].is_none() {
        let write_uvw_and_coverage_desc = self.baker_config.graphics_programs.get("write_uvw_and_coverage")
          .ok_or(HalaRendererError::new("Failed to get graphics program \"write_uvw_and_coverage\".", None))?;
        let mut write_uvw_and_coverage_descriptor_set_layouts = vec![&self.sdf_baker_resources.static_descriptor_set.layout];
        if !write_uvw_and_coverage_desc.bindings.is_empty() {
          let write_uvw_and_coverage_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("write_uvw_and_coverage")
            .ok_or(HalaRendererError::new("Failed to get the write_uvw_and_coverage descriptor set.", None))?;
          write_uvw_and_coverage_descriptor_set_layouts.push(&write_uvw_and_coverage_descriptor_set.layout);
        }
        self.sdf_baker_resources.write_uvw_and_coverage_programs[i] = Some(HalaGraphicsProgram::with_formats_and_size(
          Rc::clone(&self.resources.context.borrow().logical_device),
          &[hala_gfx::HalaFormat::R8G8B8A8_SRGB],
          None,
          width[i],
          height[i],
          write_uvw_and_coverage_descriptor_set_layouts.as_slice(),
          hala_gfx::HalaPipelineCreateFlags::default(),
          &[] as &[hala_gfx::HalaVertexInputAttributeDescription],
          &[] as &[hala_gfx::HalaVertexInputBindingDescription],
          &[
            hala_gfx::HalaDynamicState::VIEWPORT,
          ],
          write_uvw_and_coverage_desc,
          None,
          &format!("write_uvw_and_coverage_{}", i),
        )?);
      }
      if self.sdf_baker_resources.write_triangle_ids_to_voxels_programs[i].is_none() {
        let write_triangle_ids_to_voxels_desc = self.baker_config.graphics_programs.get("write_triangle_ids_to_voxels")
          .ok_or(HalaRendererError::new("Failed to get graphics program \"write_triangle_ids_to_voxels\".", None))?;
        let mut write_triangle_ids_to_voxels_descriptor_set_layouts = vec![&self.sdf_baker_resources.static_descriptor_set.layout];
        if !write_triangle_ids_to_voxels_desc.bindings.is_empty() {
          let write_triangle_ids_to_voxels_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("write_triangle_ids_to_voxels")
            .ok_or(HalaRendererError::new("Failed to get the write_triangle_ids_to_voxels descriptor set.", None))?;
          write_triangle_ids_to_voxels_descriptor_set_layouts.push(&write_triangle_ids_to_voxels_descriptor_set.layout);
        }
        self.sdf_baker_resources.write_triangle_ids_to_voxels_programs[i] = Some(HalaGraphicsProgram::with_formats_and_size(
          Rc::clone(&self.resources.context.borrow().logical_device),
          &[hala_gfx::HalaFormat::R8G8B8A8_SRGB],
          None,
          width[i],
          height[i],
          write_triangle_ids_to_voxels_descriptor_set_layouts.as_slice(),
          hala_gfx::HalaPipelineCreateFlags::default(),
          &[] as &[hala_gfx::HalaVertexInputAttributeDescription],
          &[] as &[hala_gfx::HalaVertexInputBindingDescription],
          &[
            hala_gfx::HalaDynamicState::VIEWPORT,
          ],
          write_triangle_ids_to_voxels_desc,
          None,
          &format!("write_triangle_ids_to_voxels_{}", i),
        )?);
      }
    }
    Ok(())
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn build_geometry_update(
    &self,
    bounds: &HalaBounds,
    index_buffer: &hala_gfx::HalaBuffer,
    vertex_buffer: &hala_gfx::HalaBuffer,
    triangle_uvw_buffer: &hala_gfx::HalaBuffer,
    coord_flip_buffer: &hala_gfx::HalaBuffer,
    aabb_buffer: &hala_gfx::HalaBuffer,
    vertices_buffer: &hala_gfx::HalaBuffer,
    voxels_buffer: &hala_gfx::HalaBuffer,
    counters_buffer: &hala_gfx::HalaBuffer,
    accum_counters_buffer: &hala_gfx::HalaBuffer,
    triangles_in_voxels_buffer: &hala_gfx::HalaBuffer,
  ) -> Result<
    (
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
    ),
    HalaRendererError
  > {
    let mesh_uniform = SDFBakerCSMeshUniform {
      vertex_position_offset: 0,
      vertex_stride: std::mem::size_of::<hala_renderer::scene::HalaVertex>() as u32,
      index_stride: std::mem::size_of::<u32>() as u32,
    };
    log::debug!("Mesh uniform: {:?}", mesh_uniform);
    self.sdf_baker_resources.mesh_uniform_buffer.update_memory(0, std::slice::from_ref(&mesh_uniform))?;

    // Make 3 views conservative rasterization uniform.
    let (xy_plane_mtx, zx_plane_mtx, yz_plane_mtx) = self.get_camera_matrices(bounds);
    let conservative_rasterization_uniform = SDFBakerCSConservativeRasterizationUniform {
      world_to_clip: [xy_plane_mtx, zx_plane_mtx, yz_plane_mtx],
      clip_to_world: [xy_plane_mtx.inverse(), zx_plane_mtx.inverse(), yz_plane_mtx.inverse()],
      conservative_offset: 0.5,
    };
    log::debug!("Conservative rasterization uniform: {:?}", conservative_rasterization_uniform);
    self.sdf_baker_resources.conservative_rasterization_uniform_buffer.update_memory(
      0,
      std::slice::from_ref(&conservative_rasterization_uniform)
    )?;

    let generate_triangles_uvw_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("gen_tri_in_uvw")
      .ok_or(HalaRendererError::new("Failed to get the generate_triangles_uvw descriptor set.", None))?;
    generate_triangles_uvw_descriptor_set.update_uniform_buffers(
      0,
      0,
      &[&self.sdf_baker_resources.mesh_uniform_buffer],
    );
    generate_triangles_uvw_descriptor_set.update_storage_buffers(
      0,
      1,
      &[index_buffer],
    );
    generate_triangles_uvw_descriptor_set.update_storage_buffers(
      0,
      2,
      &[vertex_buffer],
    );
    generate_triangles_uvw_descriptor_set.update_storage_buffers(
      0,
      3,
      &[triangle_uvw_buffer],
    );
    let calculate_triangles_direction_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("cal_tri_dir")
      .ok_or(HalaRendererError::new("Failed to get the calculate_triangles_direction descriptor set.", None))?;
    calculate_triangles_direction_descriptor_set.update_uniform_buffers(
      0,
      0,
      &[&self.sdf_baker_resources.mesh_uniform_buffer],
    );
    calculate_triangles_direction_descriptor_set.update_storage_buffers(
      0,
      1,
      &[index_buffer],
    );
    calculate_triangles_direction_descriptor_set.update_storage_buffers(
      0,
      2,
      &[vertex_buffer],
    );
    calculate_triangles_direction_descriptor_set.update_storage_buffers(
      0,
      3,
      &[coord_flip_buffer],
    );
    let conservative_rasterization_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("conservative_rasterization")
      .ok_or(HalaRendererError::new("Failed to get the conservative_rasterization descriptor set.", None))?;
    conservative_rasterization_descriptor_set.update_uniform_buffers(
      0,
      0,
      &[&self.sdf_baker_resources.mesh_uniform_buffer],
    );
    conservative_rasterization_descriptor_set.update_storage_buffers(
      0,
      1,
      &[index_buffer],
    );
    conservative_rasterization_descriptor_set.update_storage_buffers(
      0,
      2,
      &[vertex_buffer],
    );
    conservative_rasterization_descriptor_set.update_uniform_buffers(
      0,
      3,
      &[&self.sdf_baker_resources.conservative_rasterization_uniform_buffer],
    );
    conservative_rasterization_descriptor_set.update_storage_buffers(
      0,
      4,
      &[coord_flip_buffer],
    );
    conservative_rasterization_descriptor_set.update_storage_buffers(
      0,
      5,
      &[aabb_buffer],
    );
    conservative_rasterization_descriptor_set.update_storage_buffers(
      0,
      6,
      &[vertices_buffer],
    );
    let write_uvw_and_coverage_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("write_uvw_and_coverage")
      .ok_or(HalaRendererError::new("Failed to get the write_uvw_and_coverage descriptor set.", None))?;
    write_uvw_and_coverage_descriptor_set.update_storage_buffers(
      0,
      0,
      &[vertices_buffer],
    );
    write_uvw_and_coverage_descriptor_set.update_storage_buffers(
      0,
      1,
      &[coord_flip_buffer],
    );
    write_uvw_and_coverage_descriptor_set.update_storage_buffers(
      0,
      2,
      &[aabb_buffer],
    );
    write_uvw_and_coverage_descriptor_set.update_storage_buffers(
      0,
      3,
      &[voxels_buffer],
    );
    write_uvw_and_coverage_descriptor_set.update_storage_buffers(
      0,
      4,
      &[counters_buffer],
    );
    let write_triangle_ids_to_voxels_descriptor_set = self.sdf_baker_resources.descriptor_sets.get("write_triangle_ids_to_voxels")
      .ok_or(HalaRendererError::new("Failed to get the write_triangle_ids_to_voxels descriptor set.", None))?;
    write_triangle_ids_to_voxels_descriptor_set.update_storage_buffers(
      0,
      0,
      &[vertices_buffer],
    );
    write_triangle_ids_to_voxels_descriptor_set.update_storage_buffers(
      0,
      1,
      &[coord_flip_buffer],
    );
    write_triangle_ids_to_voxels_descriptor_set.update_storage_buffers(
      0,
      2,
      &[aabb_buffer],
    );
    write_triangle_ids_to_voxels_descriptor_set.update_storage_buffers(
      0,
      4,
      &[accum_counters_buffer],
    );
    write_triangle_ids_to_voxels_descriptor_set.update_storage_buffers(
      0,
      5,
      &[triangles_in_voxels_buffer],
    );

    Ok((
      generate_triangles_uvw_descriptor_set,
      calculate_triangles_direction_descriptor_set,
      conservative_rasterization_descriptor_set,
      write_uvw_and_coverage_descriptor_set,
      write_triangle_ids_to_voxels_descriptor_set,
    ))
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn build_geometry_compute(
    &self,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    triangle_uvw_buffer: &hala_gfx::HalaBuffer,
    coord_flip_buffer: &hala_gfx::HalaBuffer,
    aabb_buffer: &hala_gfx::HalaBuffer,
    vertices_buffer: &hala_gfx::HalaBuffer,
    generate_triangles_uvw_descriptor_set: &hala_gfx::HalaDescriptorSet,
    calculate_triangles_direction_descriptor_set: &hala_gfx::HalaDescriptorSet,
    conservative_rasterization_descriptor_set: &hala_gfx::HalaDescriptorSet,
    num_of_triangles: u32,
  ) -> Result<(), HalaRendererError> {
    // triangle_uvw_buffer and coord_flip_buffer be going to be written by compute shaders.
    {
      // Write after read don't need availability or visibility operations between two compute shaders.
      // So execution dependency is enough. No need to set access mask.
      command_buffers.set_buffer_barriers(
        0,
        &[
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            size: triangle_uvw_buffer.size,
            buffer: triangle_uvw_buffer.raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            size: coord_flip_buffer.size,
            buffer: coord_flip_buffer.raw,
            ..Default::default()
          },
        ]
      );
    }

    // Generate triangles UVW.
    {
      let generate_triangles_uvw_program = self.sdf_baker_resources.compute_programs.get("gen_tri_in_uvw")
        .ok_or(HalaRendererError::new("Failed to get the generate_triangles_uvw program.", None))?;
      generate_triangles_uvw_program.bind(
        0,
        command_buffers,
        &[
          &self.sdf_baker_resources.static_descriptor_set,
          generate_triangles_uvw_descriptor_set,
        ],
      );
      generate_triangles_uvw_program.dispatch(
        0,
        command_buffers,
        (num_of_triangles as f32 / 64.0).ceil() as u32,
        1,
        1,
      );
    }

    // Calculate triangles direction.
    {
      let calculate_triangles_direction_program = self.sdf_baker_resources.compute_programs.get("cal_tri_dir")
        .ok_or(HalaRendererError::new("Failed to get the calculate_triangles_direction program.", None))?;
      calculate_triangles_direction_program.bind(
        0,
        command_buffers,
        &[
          &self.sdf_baker_resources.static_descriptor_set,
          calculate_triangles_direction_descriptor_set,
        ],
      );
      calculate_triangles_direction_program.dispatch(
        0,
        command_buffers,
        (num_of_triangles as f32 / 64.0).ceil() as u32,
        1,
        1,
      );
    }

    // aabb_buffer and vertices_buffer be going to be written by compute shaders.
    // coord_flip_buffer be going to be read by compute shaders.
    {
      command_buffers.set_buffer_barriers(
        0,
        &[
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            size: aabb_buffer.size,
            buffer: aabb_buffer.raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            size: vertices_buffer.size,
            buffer: vertices_buffer.raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            size: coord_flip_buffer.size,
            buffer: coord_flip_buffer.raw,
            ..Default::default()
          },
        ],
      );
    }

    // Conservative rasterization.
    {
      let conservative_rasterization_program = self.sdf_baker_resources.compute_programs.get("conservative_rasterization")
        .ok_or(HalaRendererError::new("Failed to get the conservative_rasterization program.", None))?;
      conservative_rasterization_program.bind(
        0,
        command_buffers,
        &[
          &self.sdf_baker_resources.static_descriptor_set,
          conservative_rasterization_descriptor_set,
        ],
      );
      // Dispatch 3 times for 3 views.
      // One triangle is belong to 1 view ONLY.
      // So here no need to set buffer barriers between 3 views.
      // 0 is XY plane, 1 is ZX plane, 2 is YZ plane.
      for i in 0u32..3u32 {
        conservative_rasterization_program.push_constants(
          0,
          command_buffers,
          0,
          &i.to_le_bytes(),
        );
        conservative_rasterization_program.dispatch(
          0,
          command_buffers,
          (num_of_triangles as f32 / 64.0).ceil() as u32,
          1,
          1,
        );
      }
    }

    Ok(())
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn build_geometry_draw_pass_1(
    &self,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    aabb_buffer: &hala_gfx::HalaBuffer,
    vertices_buffer: &hala_gfx::HalaBuffer,
    voxels_buffer: &hala_gfx::HalaBuffer,
    counters_buffer: &hala_gfx::HalaBuffer,
    render_targets: [&hala_gfx::HalaImage; 3],
    write_uvw_and_coverage_descriptor_set: &hala_gfx::HalaDescriptorSet,
    num_of_triangles: u32,
  ) -> Result<(), HalaRendererError> {
    // aabb_buffer and vertices_buffer be going to be read by other shaders.
    {
      command_buffers.set_buffer_barriers(
        0,
        &[
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            size: aabb_buffer.size,
            buffer: aabb_buffer.raw,
            ..Default::default()
          },
          hala_gfx::HalaBufferBarrierInfo {
            src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
            dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
            dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
            size: vertices_buffer.size,
            buffer: vertices_buffer.raw,
            ..Default::default()
          },
        ],
      );
    }

    // Write UVWs into the voxel buffer.
    // Write triangle coverage(3 times by XY view, ZX view and YZ view) count into the counter buffer.
    {
      // Setup image barriers.
      command_buffers.set_image_barriers(
        0,
        (0..3).map(|i| hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: render_targets[i].raw,
          ..Default::default()
        }).collect::<Vec<_>>().as_slice(),
      );

      // Begin draw something.
      for (axis, &render_target) in render_targets.iter().enumerate() {
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
              src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
              dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
              size: counters_buffer.size,
              buffer: counters_buffer.raw,
              ..Default::default()
            },
          ]
        );

        let clear_color = [0.0, 0.0, 0.0, 0.0];
        command_buffers.begin_rendering_with_rt(
          0,
          &[render_target],
          None,
          (0, 0, render_target.extent.width, render_target.extent.height),
          Some(clear_color),
          None,
          None,
        );

        command_buffers.set_viewport(
          0,
          0,
          &[
            (
              0.,
              render_target.extent.height as f32,
              render_target.extent.width as f32,
              -(render_target.extent.height as f32),
              0.,
              1.
            ),
          ]);

        // Draw.
        let write_uvw_and_coverage_program = self.sdf_baker_resources.write_uvw_and_coverage_programs[axis]
          .as_ref()
          .ok_or(HalaRendererError::new("Failed to get the write_uvw_and_coverage program.", None))?;

        write_uvw_and_coverage_program.bind(
          0,
          command_buffers,
          &[
            self.sdf_baker_resources.static_descriptor_set.as_ref(),
            write_uvw_and_coverage_descriptor_set,
          ],
        );

        write_uvw_and_coverage_program.push_constants(
          0,
          command_buffers,
          0,
          &(axis as u32).to_le_bytes(),
        );

        write_uvw_and_coverage_program.draw(
          0,
          command_buffers,
          num_of_triangles * 3,
          1,
          0,
          0,
        );

        // End draw something and setup image barriers.
        command_buffers.end_rendering(0);
      }

      command_buffers.set_image_barriers(
        0,
        (0..3).map(|i| hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          new_layout: hala_gfx::HalaImageLayout::SHADER_READ_ONLY_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_SAMPLED_READ,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::ALL_COMMANDS,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: render_targets[i].raw,
          ..Default::default()
        }).collect::<Vec<_>>().as_slice(),
      );
    }

    Ok(())
  }

  pub(super) fn build_geometry_draw_pass_2(
    &self,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    accum_counters_buffer: &hala_gfx::HalaBuffer,
    triangles_in_voxels_buffer: &hala_gfx::HalaBuffer,
    render_targets: [&hala_gfx::HalaImage; 3],
    write_triangle_ids_to_voxels_descriptor_set: &hala_gfx::HalaDescriptorSet,
    num_of_triangles: u32,
  ) -> Result<(), HalaRendererError> {
    // Write triangle IDs to the triangles in voxels buffer.
    // Write triangle count to the accum counter buffer.
    {
      // Setup image barriers.
      command_buffers.set_image_barriers(
        0,
        (0..3).map(|i| hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::SHADER_READ_ONLY_OPTIMAL,
          new_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_SAMPLED_READ,
          dst_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::ALL_COMMANDS,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: render_targets[i].raw,
          ..Default::default()
        }).collect::<Vec<_>>().as_slice(),
      );

      // Begin draw something.
      for (axis, &render_target) in render_targets.iter().enumerate() {
        command_buffers.set_buffer_barriers(
          0,
          &[
            hala_gfx::HalaBufferBarrierInfo {
              src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
              size: triangles_in_voxels_buffer.size,
              buffer: triangles_in_voxels_buffer.raw,
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
          ]
        );

        let clear_color = [0.0, 0.0, 0.0, 0.0];
        command_buffers.begin_rendering_with_rt(
          0,
          &[render_target],
          None,
          (0, 0, render_target.extent.width, render_target.extent.height),
          Some(clear_color),
          None,
          None,
        );

        command_buffers.set_viewport(
          0,
          0,
          &[
            (
              0.,
              render_target.extent.height as f32,
              render_target.extent.width as f32,
              -(render_target.extent.height as f32),
              0.,
              1.
            ),
          ]);

        // Draw.
        let write_triangle_ids_to_voxels_program = self.sdf_baker_resources.write_triangle_ids_to_voxels_programs[axis]
          .as_ref()
          .ok_or(HalaRendererError::new("Failed to get the write_triangle_ids_to_voxels program.", None))?;

          write_triangle_ids_to_voxels_program.bind(
          0,
          command_buffers,
          &[
            self.sdf_baker_resources.static_descriptor_set.as_ref(),
            write_triangle_ids_to_voxels_descriptor_set,
          ],
        );

        write_triangle_ids_to_voxels_program.push_constants(
          0,
          command_buffers,
          0,
          &(axis as u32).to_le_bytes(),
        );

        write_triangle_ids_to_voxels_program.draw(
          0,
          command_buffers,
          num_of_triangles * 3,
          1,
          0,
          0,
        );

        // End draw something and setup image barriers.
        command_buffers.end_rendering(0);
      }

      command_buffers.set_image_barriers(
        0,
        (0..3).map(|i| hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          new_layout: hala_gfx::HalaImageLayout::SHADER_READ_ONLY_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_SAMPLED_READ,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::ALL_COMMANDS,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: render_targets[i].raw,
          ..Default::default()
        }).collect::<Vec<_>>().as_slice(),
      );
    }

    Ok(())
  }

}