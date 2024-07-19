use glam::Vec4Swizzles;

use hala_renderer::{
  scene,
  scene::gpu,
  error::HalaRendererError,
};

use crate::baker::{
  Axis,
  SDFBaker,
  SDFBakerResources,
};

use crate::baker::sdf_resources::SDFBakerCSGlobalUniform;
use crate::baker::udf_resources::UDFBakerCSGlobalUniform;

pub mod sdf_initialize;
pub mod build_geometry;
pub mod prefix_sum;
pub mod ray_map;
pub mod find_sign;
pub mod surface_closing;
pub mod distance_transform_winding;
pub mod udf_initialize;
pub mod splat_triangle_distance;

impl SDFBaker {

  /// Get the number of meshes.
  /// return: The number of meshes.
  pub fn get_num_of_meshes(&self) -> usize {
    self.num_of_meshes
  }

  /// Get the model matrix in the scene.
  /// param index: The index of the model matrix.
  /// return: The model matrix.
  pub fn get_model_matrix_in_scene(&self, index: i32) -> &glam::Mat4 {
    self.matrices_in_scene.get(index as usize).unwrap()
  }

  /// Get the VP matrix in the scene.
  /// return: The VP matrix.
  pub fn get_vp_matrix_in_scene(&self) -> glam::Mat4 {
    let scene_in_gpu = self.scene_in_gpu.as_ref().unwrap();
    scene_in_gpu.camera_proj_matrices[0] * scene_in_gpu.camera_view_matrices[0]
  }

  /// Get the MVP matrix in the scene.
  /// param index: The index of the MVP matrix.
  /// return: The MVP matrix.
  pub fn get_mvp_matrix_in_scene(&self, index: i32) -> glam::Mat4 {
    let scene_in_gpu = self.scene_in_gpu.as_ref().unwrap();
    let model_mtx = self.matrices_in_scene.get(index as usize).unwrap();
    scene_in_gpu.camera_proj_matrices[0] * scene_in_gpu.camera_view_matrices[0] * *model_mtx
  }

  /// Get the camera position from the view matrix.
  /// param index: The index of the camera.
  /// return: The camera position.
  pub fn get_camera_position(&self, index: i32) -> glam::Vec3 {
    let scene_in_gpu = self.scene_in_gpu.as_ref().unwrap();
    scene_in_gpu.camera_view_matrices[index as usize].inverse().col(3).xyz()
  }

  /// Fit the desired box to the bounds.
  pub fn fit_box_to_bounds(&mut self) {
    let bounds = self.get_selected_mesh_bounds().unwrap();

    let max_size = bounds.get_size().iter().fold(0.0, |a: f32, b| a.max(*b));
    let voxel_size = max_size / self.settings.max_resolution as f32;
    let padding = [
      self.settings.padding[0] * voxel_size,
      self.settings.padding[1] * voxel_size,
      self.settings.padding[2] * voxel_size,
    ];

    let center = [
      bounds.center[0],
      bounds.center[1],
      bounds.center[2]
    ];
    let size = [
      (bounds.extents[0] + padding[0]) * 2.0,
      (bounds.extents[1] + padding[1]) * 2.0,
      (bounds.extents[2] + padding[2]) * 2.0
    ];
    self.settings.center = center;
    self.settings.desired_size = size;
  }

  /// Snap the actual box to the bounds.
  pub fn snap_box_to_bounds(&mut self) {
    let max_extent = self.settings.desired_size.iter().fold(0.0, |a: f32, b| a.max(*b));
    let ref_axis = if max_extent == self.settings.desired_size[0] {
      Axis::X
    } else if max_extent == self.settings.desired_size[1] {
      Axis::Y
    } else {
      Axis::Z
    };

    self.settings.actual_size = match ref_axis {
      Axis::X => {
        let dim_x = (self.settings.max_resolution as f32 * self.settings.desired_size[0] / max_extent).round().max(1.0);
        let dim_y = (self.settings.max_resolution as f32 * self.settings.desired_size[1] / max_extent).ceil().max(1.0);
        let dim_z = (self.settings.max_resolution as f32 * self.settings.desired_size[2] / max_extent).ceil().max(1.0);
        let voxel_size = max_extent / dim_x;
        [dim_x * voxel_size, dim_y * voxel_size, dim_z * voxel_size]
      },
      Axis::Y => {
        let dim_x = (self.settings.max_resolution as f32 * self.settings.desired_size[0] / max_extent).ceil().max(1.0);
        let dim_y = (self.settings.max_resolution as f32 * self.settings.desired_size[1] / max_extent).round().max(1.0);
        let dim_z = (self.settings.max_resolution as f32 * self.settings.desired_size[2] / max_extent).ceil().max(1.0);
        let voxel_size = max_extent / dim_y;
        [dim_x * voxel_size, dim_y * voxel_size, dim_z * voxel_size]
      },
      Axis::Z => {
        let dim_x = (self.settings.max_resolution as f32 * self.settings.desired_size[0] / max_extent).ceil().max(1.0);
        let dim_y = (self.settings.max_resolution as f32 * self.settings.desired_size[1] / max_extent).ceil().max(1.0);
        let dim_z = (self.settings.max_resolution as f32 * self.settings.desired_size[2] / max_extent).round().max(1.0);
        let voxel_size = max_extent / dim_z;
        [dim_x * voxel_size, dim_y * voxel_size, dim_z * voxel_size]
      },
    }
  }

  /// Estimate the grid size.
  /// return: The grid size.
  pub fn estimate_grid_size(&self) -> [u32; 3] {
    let max_extent = self.settings.desired_size.iter().fold(0.0, |a: f32, b| a.max(*b));
    let ref_axis = if max_extent == self.settings.desired_size[0] {
      Axis::X
    } else if max_extent == self.settings.desired_size[1] {
      Axis::Y
    } else {
      Axis::Z
    };

    match ref_axis {
      Axis::X => {
        let dim_x = (self.settings.max_resolution as f32 * self.settings.desired_size[0] / max_extent).round().max(1.0);
        let dim_y = (self.settings.max_resolution as f32 * self.settings.desired_size[1] / max_extent).ceil().max(1.0);
        let dim_z = (self.settings.max_resolution as f32 * self.settings.desired_size[2] / max_extent).ceil().max(1.0);
        [dim_x as u32, dim_y as u32, dim_z as u32]
      },
      Axis::Y => {
        let dim_x = (self.settings.max_resolution as f32 * self.settings.desired_size[0] / max_extent).ceil().max(1.0);
        let dim_y = (self.settings.max_resolution as f32 * self.settings.desired_size[1] / max_extent).round().max(1.0);
        let dim_z = (self.settings.max_resolution as f32 * self.settings.desired_size[2] / max_extent).ceil().max(1.0);
        [dim_x as u32, dim_y as u32, dim_z as u32]
      },
      Axis::Z => {
        let dim_x = (self.settings.max_resolution as f32 * self.settings.desired_size[0] / max_extent).ceil().max(1.0);
        let dim_y = (self.settings.max_resolution as f32 * self.settings.desired_size[1] / max_extent).ceil().max(1.0);
        let dim_z = (self.settings.max_resolution as f32 * self.settings.desired_size[2] / max_extent).round().max(1.0);
        [dim_x as u32, dim_y as u32, dim_z as u32]
      },
    }
  }

  /// Get the selected mesh primitive.
  /// return: The selected mesh primitive.
  fn get_selected_mesh_primitive(&self) -> Result<&gpu::HalaPrimitive, HalaRendererError> {
    let mut index = 0;
    let scene_in_gpu = self.scene_in_gpu.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the scene in the GPU.", None))?;

    for mesh in scene_in_gpu.meshes.iter() {
      for prim in mesh.primitives.iter() {
        if index == self.settings.selected_mesh_index {
          return Ok(prim);
        }
        index += 1;
      }
    }

    Err(HalaRendererError::new("Failed to get the mesh buffers.", None))
  }

  /// Get selected mesh's index and vertex buffer.
  /// return: The selected mesh's index and vertex buffer.
  fn get_selected_mesh_buffers(&self) -> Result<(&hala_gfx::HalaBuffer, &hala_gfx::HalaBuffer), HalaRendererError> {
    let mut index = 0;
    let scene_in_gpu = self.scene_in_gpu.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the scene in the GPU.", None))?;

    for mesh in scene_in_gpu.meshes.iter() {
      for prim in mesh.primitives.iter() {
        if index == self.settings.selected_mesh_index {
          return Ok((&prim.index_buffer, &prim.vertex_buffer));
        }
        index += 1;
      }
    }

    Err(HalaRendererError::new("Failed to get the mesh buffers.", None))
  }

  /// Get the bounds of the selected mesh.
  /// return: The bounds of the selected mesh.
  fn get_selected_mesh_bounds(&self) -> Result<&scene::HalaBounds, HalaRendererError> {
    let mut index = 0;
    let scene_in_gpu = self.scene_in_gpu.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the scene in the GPU.", None))?;

    for mesh in scene_in_gpu.meshes.iter() {
      for prim in mesh.primitives.iter() {
        if index == self.settings.selected_mesh_index {
          return Ok(&prim.bounds);
        }
        index += 1;
      }
    }

    Err(HalaRendererError::new("Failed to get the mesh buffers.", None))
  }

  /// Get the camera matrices for top, back and right orthographic views.
  /// param bounds: The bounds of the object.
  /// return: The top, back and right orthographic view matrices. 0 is XY plane, 1 is ZX plane, 2 is YZ plane.
  fn get_camera_matrices(&self, bounds: &scene::HalaBounds) -> (glam::Mat4, glam::Mat4, glam::Mat4) {
    let calculate_world_to_clip_matrix = |eye, rot, width: f32, height: f32, near: f32, far: f32| {
      let proj = glam::Mat4::orthographic_rh(-width / 2.0, width / 2.0, -height / 2.0, height / 2.0, near, far);
      let view = glam::Mat4::from_scale_rotation_translation(glam::Vec3::ONE, rot, eye).inverse();
      proj * view
    };

    let xy_plane_mtx = {
      // XY plane orthographic view.
      let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(0.0, 0.0, bounds.extents[2] + 1.0);
      let rot = glam::Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0);
      let near = 1.0f32;
      let far = near + bounds.extents[2] * 2.0;
      calculate_world_to_clip_matrix(pos, rot, bounds.extents[0] * 2.0, bounds.extents[1] * 2.0, near, far)
    };

    let zx_plane_mtx = {
      // ZX plane orthographic view.
      let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(0.0, bounds.extents[1] + 1.0, 0.0);
      let rot = glam::Quat::from_euler(glam::EulerRot::YXZ, -std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2, 0.0);
      let near = 1.0f32;
      let far = near + bounds.extents[1] * 2.0;
      calculate_world_to_clip_matrix(pos, rot, bounds.extents[2] * 2.0, bounds.extents[0] * 2.0, near, far)
    };

    let yz_plane_mtx = {
      // YZ plane orthographic view.
      let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(bounds.extents[0] + 1.0, 0.0, 0.0);
      let rot = glam::Quat::from_euler(glam::EulerRot::XYZ, std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2, 0.0);
      let near = 1.0f32;
      let far = near + bounds.extents[0] * 2.0;
      calculate_world_to_clip_matrix(pos, rot, bounds.extents[1] * 2.0, bounds.extents[2] * 2.0, near, far)
    };

    (xy_plane_mtx, zx_plane_mtx, yz_plane_mtx)
  }

  /// Create all buffers and images for the baker.
  /// param num_of_triangles: The number of triangles.
  /// param dimensions: The dimensions of the voxels.
  /// param upper_bound_count: The upper bound count.
  /// return: The result.
  fn create_sdf_buffers_images(
    &mut self,
    num_of_triangles: u32,
    dimensions: &[u32; 3],
    upper_bound_count: u32,
  ) -> Result<(), HalaRendererError> {
    let num_of_voxels = dimensions[0] * dimensions[1] * dimensions[2];

    self.build_geometry_create_buffers_images(num_of_triangles, dimensions, upper_bound_count)?;
    self.prefix_sum_create_buffers_images(num_of_voxels)?;
    self.ray_map_create_buffers_images(dimensions)?;
    self.find_sign_create_buffers_images(dimensions)?;
    self.surface_closing_create_buffers_images(dimensions)?;
    self.dtw_create_buffers_images(dimensions)?;

    Ok(())
  }

  fn get_prefix_sum_dispatch_size(num_of_threads: u32) -> (u32, u32) {
    let mut dispatch_size = (0u32, 0u32);
    let num_of_groups = (num_of_threads + SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE - 1) / SDFBakerResources::PREFIX_SUM_THREAD_GROUP_SIZE;
    dispatch_size.1 = 1 + (num_of_groups / 0xFFFF);  //0xffff is maximal number of group of DX11 : D3D11_CS_DISPATCH_MAX_THREAD_GROUPS_PER_DIMENSION
    dispatch_size.0 = num_of_groups / dispatch_size.1;
    dispatch_size
  }

  /// Bake the SDF.
  pub fn bake_sdf(&mut self) -> Result<(), HalaRendererError> {
    let primitive = self.get_selected_mesh_primitive()?;

    // Setup.
    let num_of_triangles = primitive.index_count / 3;
    let dimensions = self.estimate_grid_size();
    let num_of_voxels = dimensions[0] * dimensions[1] * dimensions[2];
    let max_extent = self.settings.actual_size.iter().fold(0.0, |a: f32, b| a.max(*b));
    // Triangle ID buffer max size.
    // Assume only half of the voxels have triangles.
    let num_of_voxels_has_triangles = dimensions[0] as f64 * dimensions[1] as f64 * dimensions[2] as f64 / 2.0f64;
    // Assum one triangle is shared by 8 voxels. Assume the number of triangles in a voxel is sqrt(_numOfTriangles).
    let avg_triangles_per_voxel = (num_of_triangles as f64 / num_of_voxels_has_triangles * 8.0f64).max((num_of_triangles as f64).sqrt());
    let upper_bound_count64 = (num_of_voxels_has_triangles * avg_triangles_per_voxel) as u64;
    let upper_bound_count = (1536 * (1 << 18)).min(upper_bound_count64) as u32; // Limit the buffer size to 1536 * 2^18.
    let upper_bound_count = upper_bound_count.max(1024); // At least 1024 triangle.
    let num_of_jfa_passes = self.settings.max_resolution.ilog2();

    // Create buffers and images.
    self.create_sdf_buffers_images(num_of_triangles, &dimensions, upper_bound_count)?;
    let triangle_uvw_buffer = self.sdf_baker_resources.triangle_uvw_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the triangle_uvw buffer.", None))?;
    let coord_flip_buffer = self.sdf_baker_resources.coord_flip_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the coord_flip buffer.", None))?;
    let aabb_buffer = self.sdf_baker_resources.aabb_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the aabb buffer.", None))?;
    let vertices_buffer = self.sdf_baker_resources.vertices_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the vertices buffer.", None))?;
    let voxels_buffer = self.sdf_baker_resources.voxels_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the voxels buffer.", None))?;
    let counters_buffer = self.sdf_baker_resources.counters_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the counters buffer.", None))?;
    let in_sum_blocks_buffer = self.sdf_baker_resources.in_sum_blocks_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the in_sum_blocks buffer.", None))?;
    let sum_blocks_buffer = self.sdf_baker_resources.sum_blocks_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the sum_blocks buffer.", None))?;
    let accum_sum_blocks_buffer = self.sdf_baker_resources.accum_sum_blocks_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the accum_sum_blocks buffer.", None))?;
    let accum_counters_buffer = self.sdf_baker_resources.accum_counters_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the accum_counters buffer.", None))?;
    let tmp_buffer = self.sdf_baker_resources.tmp_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the tmp buffer.", None))?;
    let additional_sum_blocks_buffer = self.sdf_baker_resources.additional_sum_blocks_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the additional_sum_blocks buffer.", None))?;
    let triangles_in_voxels_buffer = self.sdf_baker_resources.triangles_in_voxels_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the triangles_in_voxels buffer.", None))?;
    let ray_map = self.sdf_baker_resources.ray_map.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the ray_map.", None))?;
    let sign_map = self.sdf_baker_resources.sign_map.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the sign_map.", None))?;
    let sign_map_bis = self.sdf_baker_resources.sign_map_bis.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the sign_map_bis.", None))?;
    let voxels_texture = self.sdf_baker_resources.voxels_texture.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the voxels_texture.", None))?;
    let voxels_texture_bis = self.sdf_baker_resources.voxels_texture_bis.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the voxels_texture_bis.", None))?;
    let distance_texture = self.sdf_baker_resources.distance_texture.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the distance_texture.", None))?;
    let render_targets = [
      self.sdf_baker_resources.render_targets[0].as_ref()
        .ok_or(HalaRendererError::new("Failed to get the render_target 0.", None))?,
      self.sdf_baker_resources.render_targets[1].as_ref()
        .ok_or(HalaRendererError::new("Failed to get the render_target 1.", None))?,
      self.sdf_baker_resources.render_targets[2].as_ref()
        .ok_or(HalaRendererError::new("Failed to get the render_target 2.", None))?,
    ];
    let bounds = self.settings.get_bounds();
    let (index_buffer, vertex_buffer) = self.get_selected_mesh_buffers()?;

    // Update uniform buffers.
    let global_uniform = SDFBakerCSGlobalUniform {
      dimensions,
      max_dimension: self.settings.max_resolution as u32,
      upper_bound_count,
      num_of_triangles,
      max_extent,
      padding0: 0.0,
      min_bounds_extended: bounds.get_min(),
      padding1: 0.0,
      max_bounds_extended: bounds.get_max(),
    };
    log::debug!("Global uniform: {:?}", global_uniform);
    self.sdf_baker_resources.global_uniform_buffer.update_memory(0, std::slice::from_ref(&global_uniform))?;

    // Update the descriptor sets.
    for (index, descriptor_set) in self.sdf_baker_resources.image_2_screen_descriptor_sets.iter().enumerate() {
      descriptor_set.update_combined_image_samplers(
        0,
        0,
        &[(render_targets[index], self.sdf_baker_resources.image_2_screen_sampler.as_ref())],
      );
    }
    self.cross_xyz_descriptor_set.update_combined_image_samplers(
      0,
      0,
      &[
        (distance_texture, self.image3d_sampler.as_ref())
      ],
    );
    self.sdf_visualization_descriptor_set.update_uniform_buffers(
      0,
      0,
      &[self.sdf_visualization_uniform_buffer.as_ref()],
    );
    self.sdf_visualization_descriptor_set.update_combined_image_samplers(
      0,
      1,
      &[
        (distance_texture, self.image3d_sampler.as_ref())
      ],
    );

    let initialize_descriptor_set = self.sdf_initialize_update(
      voxels_buffer,
      counters_buffer,
      accum_counters_buffer,
      ray_map,
      sign_map,
      sign_map_bis,
      voxels_texture,
      voxels_texture_bis,
    )?;
    let (
      generate_triangles_uvw_descriptor_set,
      calculate_triangles_direction_descriptor_set,
      conservative_rasterization_descriptor_set,
      write_uvw_and_coverage_descriptor_set,
      write_triangle_ids_to_voxels_descriptor_set,
    ) = self.build_geometry_update(
      &bounds,
      index_buffer,
      vertex_buffer,
      triangle_uvw_buffer,
      coord_flip_buffer,
      aabb_buffer,
      vertices_buffer,
      voxels_buffer,
      counters_buffer,
      accum_counters_buffer,
      triangles_in_voxels_buffer,
    )?;
    let (
      in_bucket_sum_descriptor_set,
      block_sum_descriptor_set,
      final_sum_descriptor_set,
      to_block_sum_buffer_descriptor_set,
      in_bucket_sum_2_descriptor_set,
      block_sum_2_descriptor_set,
      final_sum_2_descriptor_set,
    ) = self.prefix_sum_update(
      counters_buffer,
      tmp_buffer,
      sum_blocks_buffer,
      in_sum_blocks_buffer,
      additional_sum_blocks_buffer,
      accum_sum_blocks_buffer,
      accum_counters_buffer,
    )?;
    let (
      generate_ray_map_local2x2_descriptor_set,
      ray_map_sum_x_descriptor_set,
      ray_map_sum_y_descriptor_set,
      ray_map_sum_z_descriptor_set,
    ) = self.ray_map_update(
      accum_counters_buffer,
      triangles_in_voxels_buffer,
      triangle_uvw_buffer,
      ray_map,
    )?;
    let (
      sign_pass_6rays_descriptor_set,
      sign_pass_neighbors_descriptor_set,
      sign_pass_neighbors_2_descriptor_set,
    ) = self.find_sign_update(
      ray_map,
      sign_map,
      sign_map_bis,
    )?;
    let (
      in_out_edge_descriptor_set,
      buffer_2_image_descriptor_set,
      jfa_descriptor_set,
      jfa_2_descriptor_set,
    ) = self.surface_closing_update(
      voxels_buffer,
      if self.settings.sign_passes_count % 2 == 0 { sign_map } else { sign_map_bis },
      voxels_texture,
      voxels_texture_bis,
    )?;
    let dtw_descriptor_set = self.dtw_update(
      triangle_uvw_buffer,
      triangles_in_voxels_buffer,
      accum_counters_buffer,
      if self.settings.sign_passes_count % 2 == 0 { sign_map } else { sign_map_bis },
      if num_of_jfa_passes % 2 == 0 { voxels_texture_bis } else { voxels_texture },
      voxels_buffer,
      distance_texture,
    )?;

    // Send commands to the compute queue.
    let command_buffers = &self.bake_command_buffers;
    command_buffers.reset(0, false)?;
    command_buffers.begin(0, hala_gfx::HalaCommandBufferUsageFlags::ONE_TIME_SUBMIT)?;

    // Initialize.
    self.sdf_initialize_compute(
      command_buffers,
      voxels_buffer,
      counters_buffer,
      accum_counters_buffer,
      initialize_descriptor_set,
      &dimensions,
    )?;

    // Build geometry.
    self.build_geometry_compute(
      command_buffers,
      triangle_uvw_buffer,
      coord_flip_buffer,
      aabb_buffer,
      vertices_buffer,
      generate_triangles_uvw_descriptor_set,
      calculate_triangles_direction_descriptor_set,
      conservative_rasterization_descriptor_set,
      num_of_triangles,
    )?;

    // First draw pass.
    self.build_geometry_draw_pass_1(
      command_buffers,
      aabb_buffer,
      vertices_buffer,
      voxels_buffer,
      counters_buffer,
      render_targets,
      write_uvw_and_coverage_descriptor_set,
      num_of_triangles,
    )?;

    // Prefix sum.
    self.prefix_sum_compute(
      command_buffers,
      voxels_buffer,
      counters_buffer,
      tmp_buffer,
      sum_blocks_buffer,
      in_sum_blocks_buffer,
      additional_sum_blocks_buffer,
      accum_sum_blocks_buffer,
      accum_counters_buffer,
      in_bucket_sum_descriptor_set,
      block_sum_descriptor_set,
      final_sum_descriptor_set,
      to_block_sum_buffer_descriptor_set,
      in_bucket_sum_2_descriptor_set,
      block_sum_2_descriptor_set,
      final_sum_2_descriptor_set,
      num_of_voxels,
    )?;

    // Second draw pass.
    self.build_geometry_draw_pass_2(
      command_buffers,
      accum_counters_buffer,
      triangles_in_voxels_buffer,
      render_targets,
      write_triangle_ids_to_voxels_descriptor_set,
      num_of_triangles,
    )?;

    // Ray map.
    self.ray_map_compute(
      command_buffers,
      accum_counters_buffer,
      triangles_in_voxels_buffer,
      triangle_uvw_buffer,
      ray_map,
      generate_ray_map_local2x2_descriptor_set,
      ray_map_sum_x_descriptor_set,
      ray_map_sum_y_descriptor_set,
      ray_map_sum_z_descriptor_set,
      &dimensions,
    )?;

    // Find sign.
    let sign_map = self.find_sign_compute(
      command_buffers,
      ray_map,
      sign_map,
      sign_map_bis,
      sign_pass_6rays_descriptor_set,
      sign_pass_neighbors_descriptor_set,
      sign_pass_neighbors_2_descriptor_set,
      &dimensions,
    )?;

    // Surface closing.
    let voxels_texture = self.surface_closing_compute(
      command_buffers,
      voxels_buffer,
      sign_map,
      voxels_texture,
      voxels_texture_bis,
      in_out_edge_descriptor_set,
      buffer_2_image_descriptor_set,
      jfa_descriptor_set,
      jfa_2_descriptor_set,
      &dimensions,
    )?;

    // Distance transform winding.
    self.dtw_compute(
      command_buffers,
      voxels_texture,
      voxels_buffer,
      distance_texture,
      dtw_descriptor_set,
      &dimensions,
    )?;

    command_buffers.end(0)?;

    // Submit & wait.
    {
      let context = self.resources.context.borrow();
      let logical_device = context.logical_device.borrow();

      logical_device.graphics_submit(command_buffers, 0, 0)?;
      logical_device.graphics_wait(0)?;
    }

    // Validate the triangle IDs buffer size.
    {
      let data = self.debug_get_buffer_data::<u32>(accum_counters_buffer)?;
      let last_counter = data[num_of_voxels as usize - 1];
      if last_counter > upper_bound_count {
        log::error!("The triangle IDs buffer size is too small. The last counter is {}. The upper bound count is {}.", last_counter, upper_bound_count);
      } else {
        log::debug!("The triangle IDs buffer size is OK. The last counter is {}. The upper bound count is {}.", last_counter, upper_bound_count);
      }
    }

    // Debug.
    {
      // let data = self.debug_get_buffer_data::<u32>(accum_counters_buffer)?;
      // for i in 0..num_of_voxels {
      //   log::debug!("Counter[{}] = {}", i, data[i as usize]);
      // }
      // let data = self.debug_get_buffer_data::<[f32; 4]>(triangle_uvw_buffer)?;
      // for i in 0..num_of_triangles {
      //   log::debug!("Triangle[{}] = {:?}, {:?}, {:?}", i, data[i as usize * 3], data[i as usize * 3 + 1], data[i as usize * 3 + 2]);
      // }
    }

    Ok(())
  }

  /// Create all buffers and images for the baker.
  /// param num_of_voxels: The number of triangles.
  /// param dimensions: The dimensions of the voxels.
  /// return: The result.
  fn create_udf_buffers_images(
    &mut self,
    num_of_voxels: u32,
    dimensions: &[u32; 3],
  ) -> Result<(), HalaRendererError> {
    self.udf_initialize_create_buffers_images(num_of_voxels, dimensions)?;

    Ok(())
  }

  /// Bake the UDF.
  pub fn bake_udf(&mut self) -> Result<(), HalaRendererError> {
    let primitive = self.get_selected_mesh_primitive()?;

    // Setup.
    let num_of_triangles = primitive.index_count / 3;
    let max_distance = (self.settings.actual_size[0] * self.settings.actual_size[1] * self.settings.actual_size[2]).powf(1.0 / 3.0);
    let dimensions = self.estimate_grid_size();
    let num_of_voxels = dimensions[0] * dimensions[1] * dimensions[2];

    // Create buffers and images.
    self.create_udf_buffers_images(num_of_voxels, &dimensions)?;
    let distance_texture = self.udf_baker_resources.distance_texture.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the distance_texture.", None))?;
    #[allow(unused_variables)]
    let jump_buffer = self.udf_baker_resources.jump_buffer.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the jump_buffer.", None))?;
    #[allow(unused_variables)]
    let jump_buffer_bis = self.udf_baker_resources.jump_buffer_bis.as_ref()
      .ok_or(HalaRendererError::new("Failed to get the jump_buffer_bis.", None))?;
    let (index_buffer, vertex_buffer) = self.get_selected_mesh_buffers()?;

    // Update uniform buffers.
    let global_uniform = UDFBakerCSGlobalUniform {
      i_m_mtx: self.get_model_matrix_in_scene(self.settings.selected_mesh_index).inverse(),
      dimensions,
      num_of_voxels,
      num_of_triangles,
      max_distance,
      initial_distance: max_distance * 1.01,
    };
    log::debug!("Global uniform: {:?}", global_uniform);
    self.udf_baker_resources.global_uniform_buffer.update_memory(0, std::slice::from_ref(&global_uniform))?;

    // Update the descriptor sets.
    self.udf_baker_resources.static_descriptor_set.update_uniform_buffers(
      0,
      0,
      &[
        &self.udf_baker_resources.global_uniform_buffer
      ],
    );

    // Update the descriptor sets.
    let (
      initialize_descriptor_set,
      finalize_descriptor_set,
    ) = self.udf_initialize_update(
      distance_texture,
    )?;
    let splat_triangle_distance_descriptor_set = self.splat_triangle_distance_update(
      index_buffer,
      vertex_buffer,
      distance_texture,
    )?;

    // Send commands to the compute queue.
    let command_buffers = &self.bake_command_buffers;
    command_buffers.reset(0, false)?;
    command_buffers.begin(0, hala_gfx::HalaCommandBufferUsageFlags::ONE_TIME_SUBMIT)?;

    // Initialize.
    self.udf_initialize_compute_pass_1(
      command_buffers,
      distance_texture,
      initialize_descriptor_set,
      &dimensions,
    )?;

    // Splat triangle distance.
    self.splat_triangle_distance_compute(
      command_buffers,
      distance_texture,
      splat_triangle_distance_descriptor_set,
      &dimensions,
    )?;

    // Finialize
    self.udf_initialize_compute_pass_2(
      command_buffers,
      distance_texture,
      finalize_descriptor_set,
      &dimensions,
    )?;

    command_buffers.end(0)?;

    // Submit & wait.
    {
      let context = self.resources.context.borrow();
      let logical_device = context.logical_device.borrow();

      logical_device.graphics_submit(command_buffers, 0, 0)?;
      logical_device.graphics_wait(0)?;
    }

    Ok(())
  }

}