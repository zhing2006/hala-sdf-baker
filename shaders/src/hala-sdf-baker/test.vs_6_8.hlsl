#include "common.hlsl"
#include "test_common.hlsl"

struct vs_in {
  [[vk::location(0)]] float4 position: POSITION;
  [[vk::location(1)]] float4 normal: NORMAL;
  [[vk::location(2)]] float4 tangent: TANGENT;
  [[vk::location(3)]] float4 uv: TEXCOORD0;
};

vs_to_ps main(vs_in input) {
  vs_to_ps output = (vs_to_ps)0;
  float4 position = mul(m_mtx, float4(input.position.xyz, 1.0));
  output.position = mul(vp_mtx, position);
  output.uv = input.uv.xy;
  output.color = input.normal.xyz * 0.5 + 0.5;
  return output;
}