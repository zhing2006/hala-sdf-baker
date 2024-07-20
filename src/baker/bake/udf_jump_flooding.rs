use hala_renderer::error::HalaRendererError;

use crate::baker::SDFBaker;

impl SDFBaker {

  pub(super) fn jump_flooding_update(
    &self,
    _distance_texture: &hala_gfx::HalaImage,
    _jump_buffer: &hala_gfx::HalaBuffer,
    _jump_buffer_bis: &hala_gfx::HalaBuffer,
  ) -> Result<
    (
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
      &hala_gfx::HalaDescriptorSet,
    ),
    HalaRendererError,
  > {
    let jump_flooding_initialize_descriptor_set = self.udf_baker_resources.descriptor_sets.get("jump_flooding_init")
      .ok_or(HalaRendererError::new("Failed to get the jump floodinginitialize descriptor set.", None))?;
    let jump_flooding_descriptor_set = self.udf_baker_resources.descriptor_sets.get("jump_flooding")
      .ok_or(HalaRendererError::new("Failed to get the jump_flooding descriptor set.", None))?;
    let jump_flooding_finalize_descriptor_set = self.udf_baker_resources.descriptor_sets.get("jump_flooding_final")
      .ok_or(HalaRendererError::new("Failed to get the jump flooding finalize descriptor set.", None))?;

    Ok((
      jump_flooding_initialize_descriptor_set,
      jump_flooding_descriptor_set,
      jump_flooding_finalize_descriptor_set,
    ))
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn jump_flooding_compute(
    &self,
    _command_buffers: &hala_gfx::HalaCommandBufferSet,
    _distance_texture: &hala_gfx::HalaImage,
    _jump_buffer: &hala_gfx::HalaBuffer,
    _jump_buffer_bis: &hala_gfx::HalaBuffer,
    _jump_flooding_initialize_descriptor_set: &hala_gfx::HalaDescriptorSet,
    _jump_flooding_descriptor_set: &hala_gfx::HalaDescriptorSet,
    _jump_flooding_finalize_descriptor_set: &hala_gfx::HalaDescriptorSet,
    _dimensions: &[u32; 3]
  ) -> Result<(), HalaRendererError> {
    Ok(())
  }

}