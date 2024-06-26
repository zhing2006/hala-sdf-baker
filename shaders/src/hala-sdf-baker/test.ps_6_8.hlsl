#include "common.hlsl"
#include "test_common.hlsl"

struct ps_out {
  [[vk::location(0)]] float4 color: SV_Target0;
};

ps_out main(vs_to_ps input) {
  ps_out output = (ps_out)0;
  Material mtrl = g_materials_buffer.Load(0);
  if (mtrl.base_color_map_index != INVALID_INDEX) {
    float3 base_color = g_textures[mtrl.base_color_map_index].Sample(g_samplers[mtrl.base_color_map_index], input.uv).xyz;
    output.color = float4(base_color * input.color, 1.0);
  } else {
    output.color = float4(mtrl.base_color * input.color, 1.0);
  }
  return output;
}