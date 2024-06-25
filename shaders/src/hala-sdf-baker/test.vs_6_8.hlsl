#include "common.hlsl"
#include "test_common.hlsl"

struct vs_in {
  [[vk::location(0)]] float3 position: POSITION;
  [[vk::location(1)]] float3 color: COLOR0;
};

vs_to_ps main(vs_in input) {
  vs_to_ps output = (vs_to_ps)0;
  float4 position = mul(m_mtx, float4(input.position, 1.0));
  output.position = mul(vp_mtx, position);
  output.color = input.color;
  return output;
}