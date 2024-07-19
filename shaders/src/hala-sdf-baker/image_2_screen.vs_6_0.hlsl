#include "image_2_screen.hlsl"

struct PushConstants {
  float2 offset;
  float2 scale;
};

[[vk::push_constant]]
PushConstants g_push_constants;

ToFragment main(uint vertex_id: SV_VertexID) {
  ToFragment output = (ToFragment)0;

  float2 position = float2((vertex_id << 1) & 2, vertex_id & 2) - float2(1.0, 1.0);
  const float2 uv = position * 0.5 + 0.5;

  position = position * g_push_constants.scale + g_push_constants.offset;

  output.position = float4(position, 0.0, 1.0);
  output.uv = float2(uv.x, 1.0 - uv.y);

  return output;
}