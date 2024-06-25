#include "test_common.hlsl"

struct vs_in {
  float3 position: POSITION;
  float3 color: COLOR0;
};

vs_to_ps main(vs_in input) {
  vs_to_ps output = (vs_to_ps)0;
  output.position = float4(input.position, 1.0);
  output.color = input.color;
  return output;
}