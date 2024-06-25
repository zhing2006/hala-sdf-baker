#include "test_common.hlsl"

struct ps_out {
  float4 color: SV_Target0;
};

ps_out main(vs_to_ps input) {
  ps_out output = (ps_out)0;
  output.color = float4(input.color, 1.0);
  return output;
}