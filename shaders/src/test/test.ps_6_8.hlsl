#include "scene.hlsl"
#include "test_common.hlsl"

struct FragmentOutput {
  [[vk::location(0)]] float4 color: SV_Target0;
};

FragmentOutput main(ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

  uint material_index = g_push_constants.material_index;
  Material mtrl = g_materials[material_index];

  if (mtrl.base_color_map_index != INVALID_INDEX) {
    float3 base_color = g_textures[mtrl.base_color_map_index].Sample(g_samplers[mtrl.base_color_map_index], input.uv).xyz;
    output.color = float4(base_color * input.normal * 0.5 + 0.5, 1.0);
  } else {
    output.color = float4(mtrl.base_color * input.normal * 0.5 + 0.5, 1.0);
  }

  return output;
}