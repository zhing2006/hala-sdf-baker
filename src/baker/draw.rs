use hala_renderer::{
  scene,
  error::HalaRendererError,
};

use crate::baker::SDFBaker;

impl SDFBaker {

  /// Record all command to the command buffer.
  /// param index: The index of the current frame.
  /// param command_buffers: The command buffers.
  /// param ui_fn: The draw UI function.
  /// return: The result.
  pub(super) fn record_command_buffer<F>(&self, index: usize, command_buffers: &hala_gfx::HalaCommandBufferSet, ui_fn: F) -> Result<(), HalaRendererError>
    where F: FnOnce(usize, &hala_gfx::HalaCommandBufferSet) -> Result<(), hala_gfx::HalaGfxError>
  {
    let context = self.resources.context.borrow();

    // Prepare the command buffer and timestamp.
    command_buffers.reset(index, false)?;
    command_buffers.begin(index, hala_gfx::HalaCommandBufferUsageFlags::empty())?;
    command_buffers.reset_query_pool(index, &context.timestamp_query_pool, (index * 2) as u32, 2);
    command_buffers.write_timestamp(index, hala_gfx::HalaPipelineStageFlags2::NONE, &context.timestamp_query_pool, (index * 2) as u32);

    if cfg!(debug_assertions) {
      command_buffers.begin_debug_label(index, "Draw", [1.0, 0.0, 0.0, 1.0]);
    }
    // Setup swapchain barrier.
    command_buffers.set_swapchain_image_barrier(
      index,
      &context.swapchain,
      &hala_gfx::HalaImageBarrierInfo {
        old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
        new_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
        dst_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
        src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
        dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
        ..Default::default()
      },
      &hala_gfx::HalaImageBarrierInfo {
        old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
        new_layout: hala_gfx::HalaImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
        dst_access_mask: hala_gfx::HalaAccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
        src_stage_mask: hala_gfx::HalaPipelineStageFlags2::EARLY_FRAGMENT_TESTS | hala_gfx::HalaPipelineStageFlags2::LATE_FRAGMENT_TESTS,
        dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::EARLY_FRAGMENT_TESTS | hala_gfx::HalaPipelineStageFlags2::LATE_FRAGMENT_TESTS,
        aspect_mask: hala_gfx::HalaImageAspectFlags::DEPTH | if context.swapchain.has_stencil { hala_gfx::HalaImageAspectFlags::STENCIL } else { hala_gfx::HalaImageAspectFlags::empty() },
        ..Default::default()
      }
    );

    // Rendering.
    command_buffers.begin_rendering(
      index,
      &context.swapchain,
      (0, 0, self.info.width, self.info.height),
      Some([64.0 / 255.0, 46.0 / 255.0, 122.0 / 255.0, 1.0]),
      Some(0.0),
      Some(0),
    );

    // Draw the scene.
    command_buffers.set_viewport(
      index,
      0,
      &[
        (
          0.,
          self.info.height as f32,
          self.info.width as f32,
          -(self.info.height as f32), // For vulkan y is down.
          0.,
          1.
        ),
      ],
    );

    // Draw debug images to screen.
    if self.settings.show_render_targets {
      if self.sdf_baker_resources.render_targets[0].is_some() {
        self.debug_draw_image_2_screen(
          index,
          command_buffers,
          &self.sdf_baker_resources.image_2_screen_descriptor_sets[0],
          &[-0.75, -0.75, 0.25, 0.25]
        )?;
      }
      if self.sdf_baker_resources.render_targets[1].is_some() {
        self.debug_draw_image_2_screen(
          index,
          command_buffers,
          &self.sdf_baker_resources.image_2_screen_descriptor_sets[1],
          &[0.0, -0.75, 0.25, 0.25]
        )?;
      }
      if self.sdf_baker_resources.render_targets[2].is_some() {
        self.debug_draw_image_2_screen(
          index,
          command_buffers,
          &self.sdf_baker_resources.image_2_screen_descriptor_sets[2],
          &[0.75, -0.75, 0.25, 0.25]
        )?;
      }
    }

    // Draw scene.
    if self.settings.show_wireframe {
      self.draw_scene(index, command_buffers, -1, 0xFFFFFFFF, None)?;
      // if let Some(vertex_buffer) = self.baker_resources.vertices_buffer.as_ref() {
      //   self.debug_draw_vertices_buffer(index, command_buffers, self.settings.selected_mesh_index, 0xFF0000FF, Some(vertex_buffer))?;
      // }
    }

    // Draw debug image3d.
    let mvp_mtx = self.get_mvp_matrix_in_scene(self.settings.selected_mesh_index).to_cols_array();
    if self.settings.show_ray_map && self.sdf_baker_resources.ray_map.is_some() {
      self.debug_draw_image3d(
        index,
        command_buffers,
        &scene::HalaBounds::new_with_size(self.settings.center, self.settings.actual_size),
        &mvp_mtx,
      )?;
    }
    if self.settings.show_sdf && self.sdf_baker_resources.distance_texture.is_some() {
      self.debug_draw_sdf(
        index,
        command_buffers,
        &scene::HalaBounds::new_with_size(self.settings.center, self.settings.actual_size),
        &[1.0, 1.0, 1.0, 1.0],
        0.0,
      )?;
    }

    if self.settings.show_desired_box {
      self.draw_bounds(
        index,
        command_buffers,
        &scene::HalaBounds::new_with_size(self.settings.center, self.settings.desired_size),
        &mvp_mtx,
        [0.6, 0.6, 0.0, 0.6]
      )?;
    }

    if self.settings.show_actual_box {
      self.draw_bounds(
        index,
        command_buffers,
        &scene::HalaBounds::new_with_size(self.settings.center, self.settings.actual_size),
        &mvp_mtx,
        [0.0, 0.8, 0.8, 1.0]
      )?;
    }

    // Draw UI.
    if cfg!(debug_assertions) {
      command_buffers.begin_debug_label(index, "Draw UI", [0.0, 0.0, 1.0, 1.0]);
    }
    ui_fn(index, command_buffers)?;
    if cfg!(debug_assertions) {
      command_buffers.end_debug_label(index);
    }

    command_buffers.end_rendering(index);

    // Setup swapchain barrier.
    command_buffers.set_image_barriers(
      index,
      &[hala_gfx::HalaImageBarrierInfo {
        old_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        new_layout: hala_gfx::HalaImageLayout::PRESENT_SRC,
        src_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
        dst_access_mask: hala_gfx::HalaAccessFlags2::NONE,
        src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::BOTTOM_OF_PIPE,
        aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
        image: context.swapchain.images[index],
        ..Default::default()
      }],
    );
    if cfg!(debug_assertions) {
      command_buffers.end_debug_label(index);
    }

    // Write end timestamp and end command buffer.
    command_buffers.write_timestamp(
      index,
      hala_gfx::HalaPipelineStageFlags2::ALL_COMMANDS,
      &context.timestamp_query_pool,
      (index * 2 + 1) as u32);
    command_buffers.end(index)?;

    Ok(())
  }

  /// Draw the scene.
  /// param index: The index of the current frame.
  /// param command_buffers: The command buffers.
  /// param selected_primitive_index: The selected primitive index. -1 means draw all.
  /// param color: The color in hex(RGBA).
  /// param vertex_buffer: The vertex buffer.
  /// return: The result.
  fn draw_scene(
    &self,
    index: usize,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    selected_primitive_index: i32,
    color: u32,
    vertex_buffer: Option<&hala_gfx::HalaBuffer>,
  ) -> Result<(), HalaRendererError> {
    let wireframe_program = self.wireframe_program.as_ref().ok_or(HalaRendererError::new("The wireframe PSO is none!", None))?;
    let static_descriptor_set = self.static_descriptor_set.as_ref();
    let dynamic_descriptor_set = self.dynamic_descriptor_set.as_ref().ok_or(HalaRendererError::new("The dynamic descriptor set is none!", None))?;
    let textures_descriptor_set = self.textures_descriptor_set.as_ref().ok_or(HalaRendererError::new("The textures descriptor set is none!", None))?;

    wireframe_program.bind(
      index,
      command_buffers,
      &[
        static_descriptor_set,
        dynamic_descriptor_set,
        textures_descriptor_set,
      ]
    );

    let mut primitive_index = 0i32;
    let scene = self.scene_in_gpu.as_ref().ok_or(HalaRendererError::new("The scene in GPU is none!", None))?;
    for (mesh_index, mesh) in scene.meshes.iter().enumerate() {
      for primitive in mesh.primitives.iter() {
        if selected_primitive_index != -1 && primitive_index != selected_primitive_index {
          primitive_index += 1;
          continue;
        }

        let material_type = scene.material_types[primitive.material_index as usize] as usize;
        if material_type >= scene.materials.len() {
          return Err(HalaRendererError::new("The material type index is out of range!", None));
        }

        // Push constants.
        let mut push_constants = Vec::new();
        push_constants.extend_from_slice(&(mesh_index as u32).to_le_bytes());
        push_constants.extend_from_slice(&color.to_le_bytes());
        push_constants.extend_from_slice(&primitive_index.to_le_bytes());
        wireframe_program.push_constants(
          index,
          command_buffers,
          0,
          push_constants.as_slice(),
        );

        // Bind vertex buffers.
        if let Some(vertex_buffer) = vertex_buffer {
          command_buffers.bind_vertex_buffers(
            index,
            0,
            &[vertex_buffer],
            &[0]);
        } else {
          command_buffers.bind_vertex_buffers(
            index,
            0,
            &[primitive.vertex_buffer.as_ref()],
            &[0]);
        }

        // Bind index buffer.
        command_buffers.bind_index_buffers(
          index,
          &[primitive.index_buffer.as_ref()],
          &[0],
          hala_gfx::HalaIndexType::UINT32);

        // Draw.
        wireframe_program.draw_indexed(
          index,
          command_buffers,
          primitive.index_count,
          1,
          0,
          0,
          0,
        );

        primitive_index += 1;
      }
    };

    Ok(())
  }

  /// Draw the bounds.
  /// param index: The index of the current frame.
  /// param command_buffers: The command buffers.
  /// param bounds: The bounds to be drawn.
  /// param mvp_mtx: The MVP matrix.
  /// param color: The color of the bounds.
  fn draw_bounds(
    &self,
    index: usize,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    bounds: &scene::HalaBounds,
    mvp_mtx: &[f32; 16],
    color: [f32; 4],
  ) -> Result<(), HalaRendererError> {
    self.bounds_program.bind(
      index,
      command_buffers,
      &[] as &[&hala_gfx::HalaDescriptorSet]
    );

    // Push constants.
    self.bounds_program.push_constants_f32(
      index,
      command_buffers,
      0,
      &[
        mvp_mtx[0], mvp_mtx[1], mvp_mtx[2], mvp_mtx[3],
        mvp_mtx[4], mvp_mtx[5], mvp_mtx[6], mvp_mtx[7],
        mvp_mtx[8], mvp_mtx[9], mvp_mtx[10], mvp_mtx[11],
        mvp_mtx[12], mvp_mtx[13], mvp_mtx[14], mvp_mtx[15],
        bounds.center[0], bounds.center[1], bounds.center[2],
        bounds.extents[0], bounds.extents[1], bounds.extents[2],
        color[0], color[1], color[2], color[3],
      ],
    );

    // Draw lines.
    self.bounds_program.draw(
      index,
      command_buffers,
      2,
      12,
      0,
      0,
    );

    Ok(())
  }

}