#include "scene.hlsl"
#include "wireframe.hlsl"

struct VertexInput {
  [[vk::location(0)]] float4 position: POSITION;
};

ToFragment main(VertexInput input) {
  ToFragment output = (ToFragment)0;

  const ObjectUniformBuffer per_object_data = g_per_object_data[g_push_constants.object_index];
  output.position = mul(per_object_data.mvp_mtx, float4(input.position.xyz, 1.0));

  uint color = g_push_constants.material_index;
  float a = (color & 0xFF) / 255.0;
  float b = ((color >> 8) & 0xFF) / 255.0;
  float g = ((color >> 16) & 0xFF) / 255.0;
  float r = ((color >> 24) & 0xFF) / 255.0;

  output.color = float4(r, g, b, a);

  return output;
}