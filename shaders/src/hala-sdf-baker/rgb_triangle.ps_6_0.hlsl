#include "rgb_triangle.hlsl"

struct FragmentOutput {
  [[vk::location(0)]] float4 color: SV_Target0;
};

FragmentOutput main(ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

  output.color = float4(input.color, 1.0);

  return output;
}