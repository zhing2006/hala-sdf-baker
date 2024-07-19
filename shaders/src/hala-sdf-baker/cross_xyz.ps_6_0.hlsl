#include "cross_xyz.hlsl"

[[vk::combinedImageSampler]]
[[vk::binding(0, 0)]]
Texture3D<float4> g_texture;

[[vk::combinedImageSampler]]
[[vk::binding(0, 0)]]
SamplerState g_sampler;

struct FragmentOutput {
  [[vk::location(0)]] float4 color: SV_Target0;
};

FragmentOutput main(ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

  float4 color = g_texture.Sample(g_sampler, input.uvw);
  if (color.a < 1e-5f) {
    discard;
  }
  output.color = color;

  return output;
}