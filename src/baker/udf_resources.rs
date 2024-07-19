use std::rc::Rc;
use std::cell::RefCell;

use hala_renderer::error::HalaRendererError;

use crate::config;

/// The baker resources.
pub(crate) struct UDFBakerResources {
}

impl UDFBakerResources {

  /// Create a new UDF baker resources.
  /// param logical_device: The logical device.
  /// param descriptor_pool: The descriptor pool.
  /// param swapchain: The swapchain.
  /// param baker_config: The baker config.
  /// param pipeline_cache: The pipeline cache.
  /// return: The result.
  pub(crate) fn new(
    _logical_device: Rc<RefCell<hala_gfx::HalaLogicalDevice>>,
    _descriptor_pool: Rc<RefCell<hala_gfx::HalaDescriptorPool>>,
    _swapchain: &hala_gfx::HalaSwapchain,
    _baker_config: &config::BakerConfig,
    _pipeline_cache: &hala_gfx::HalaPipelineCache,
  ) -> Result<Self, HalaRendererError> {
    Ok(Self {
    })
  }

}
