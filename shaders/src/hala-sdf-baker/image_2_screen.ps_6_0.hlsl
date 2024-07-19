#include "image_2_screen.hlsl"

[[vk::combinedImageSampler]]
[[vk::binding(0, 0)]]
Texture2D<float4> g_texture;

[[vk::combinedImageSampler]]
[[vk::binding(0, 0)]]
SamplerState g_sampler;

struct FragmentOutput {
  [[vk::location(0)]] float4 color: SV_Target0;
};

FragmentOutput main(ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

  output.color = g_texture.Sample(g_sampler, input.uv);

  return output;
}