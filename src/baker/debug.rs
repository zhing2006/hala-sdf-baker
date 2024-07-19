use std::rc::Rc;

use hala_renderer::error::HalaRendererError;
use hala_renderer::scene::HalaBounds;

use crate::baker::SDFBaker;

impl SDFBaker {

  /// Debug draw the vertices buffer.
  /// param index: The index of the current frame.
  /// param command_buffers: The command buffers.
  /// param selected_primitive_index: The selected primitive index. -1 means draw all.
  /// param color: The color in hex(RGBA).
  /// param vertex_buffer: The vertex buffer.
  /// return: The result.
  #[allow(dead_code)]
  pub(super) fn debug_draw_vertices_buffer(
    &self,
    index: usize,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    selected_primitive_index: i32,
    color: u32,
    vertex_buffer: Option<&hala_gfx::HalaBuffer>,
  ) -> Result<(), HalaRendererError> {
    let wireframe_program = self.wireframe_debug_program.as_ref().ok_or(HalaRendererError::new("The wireframe PSO is none!", None))?;
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
        }

        // Draw.
        wireframe_program.draw(
          index,
          command_buffers,
          self.baker_resources.num_of_triangles * 3,
          1,
          0,
          0
        );

        primitive_index += 1;
      }
    };

    Ok(())
  }

  /// Debug draw the image to the screen.
  /// param index: The index of the current frame.
  /// param command_buffers: The command buffers.
  /// param descriptor_set: The descriptor set.
  /// return: The result.
  pub(super) fn debug_draw_image_2_screen(
    &self,
    index: usize,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    descriptor_set: &hala_gfx::HalaDescriptorSet,
    rect: &[f32; 4],
  ) -> Result<(), HalaRendererError> {
    self.baker_resources.image_2_screen_program.bind(
      index,
      command_buffers,
      &[descriptor_set]
    );

    self.baker_resources.image_2_screen_program.push_constants_f32(
      index,
      command_buffers,
      0,
      rect,
    );

    self.baker_resources.image_2_screen_program.draw(
      index,
      command_buffers,
      4,
      1,
      0,
      0
    );

    Ok(())
  }

  /// Debug draw the image3d.
  /// param index: The index of the current frame.
  /// param command_buffers: The command buffers.
  /// param bounds: The bounds to be drawn.
  /// param mvp_mtx: The MVP matrix.
  #[allow(dead_code)]
  pub(super) fn debug_draw_image3d(
    &self,
    index: usize,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    bounds: &HalaBounds,
    mvp_mtx: &[f32; 16],
  ) -> Result<(), HalaRendererError> {
    self.baker_resources.cross_xyz_program.bind(
      index,
      command_buffers,
      &[self.baker_resources.cross_xyz_descriptor_set.as_ref()]
    );

    // Push constants.
    self.baker_resources.cross_xyz_program.push_constants_f32(
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
      ],
    );

    // Draw planes.
    self.baker_resources.cross_xyz_program.draw(
      index,
      command_buffers,
      18,
      1,
      0,
      0,
    );

    Ok(())
  }

  /// Debug draw the sdf.
  /// param index: The index of the current frame.
  /// param command_buffers: The command buffers.
  /// param bounds: The bounds to be drawn.
  /// param color: The color in RGBA.
  /// param offset: The offset.
  /// return: The result.
  #[allow(dead_code)]
  pub(super) fn debug_draw_sdf(
    &self,
    index: usize,
    command_buffers: &hala_gfx::HalaCommandBufferSet,
    bounds: &HalaBounds,
    color: &[f32; 4],
    offset: f32,
  ) -> Result<(), HalaRendererError> {
    self.baker_resources.sdf_visualization_program.bind(
      index,
      command_buffers,
      &[
        self.baker_resources.sdf_visualization_descriptor_set.as_ref()
      ]
    );

    // Push constants.
    self.baker_resources.sdf_visualization_program.push_constants_f32(
      index,
      command_buffers,
      0,
      &[
        bounds.center[0], bounds.center[1], bounds.center[2],
        bounds.extents[0], bounds.extents[1], bounds.extents[2],
        color[0], color[1], color[2], color[3],
        offset,
      ],
    );

    // Draw planes.
    self.baker_resources.sdf_visualization_program.draw(
      index,
      command_buffers,
      36,
      1,
      0,
      0,
    );

    Ok(())
  }

  /// Get the buffer data for debug.
  /// param buffer: The buffer.
  /// return: The buffer data.
  pub(super) fn debug_get_buffer_data<T: Copy + Default>(
    &self,
    buffer: &hala_gfx::HalaBuffer,
  ) -> Result<Vec<T>, HalaRendererError> {
    let context = self.resources.context.borrow();

    let debug_staging_buffer = hala_gfx::HalaBuffer::new(
      Rc::clone(&context.logical_device),
      buffer.size,
      hala_gfx::HalaBufferUsageFlags::TRANSFER_DST,
      hala_gfx::HalaMemoryLocation::GpuToCpu,
      "debug_staging_buffer",
    )?;

    context.logical_device.borrow().transfer_execute_and_submit(
      &self.resources.transfer_command_buffers,
      0,
      |_logical_device, command_buffers, index| {
        command_buffers.copy_buffer_2_buffer(
          index,
          buffer,
          0,
          &debug_staging_buffer,
          0,
        );
      },
      0
    )?;
    let num_of_elements = buffer.size as usize / std::mem::size_of::<T>();
    let mut data = Vec::with_capacity(num_of_elements);
    data.resize(num_of_elements, T::default());
    debug_staging_buffer.download_memory::<T>(0, data.as_mut_slice())?;

    Ok(data)
  }

}