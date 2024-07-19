#include "../baker/sdf_baker.hlsl"
#include "prefix_sum.hlsl"

[numthreads(THREAD_GROUP_SIZE, 1, 1)]
void main(uint3 DTId: SV_DispatchThreadID, uint GI: SV_GroupIndex) {
  const uint id = (DTId.x + 1) * THREAD_GROUP_SIZE - 1;

  uint x;
  if (id >= g_push_constants.num_of_elements) {
    x = 0u;
  } else {
    x = _input_buffer[id] + _counters_buffer[id];
  }

  prefix_sum(DTId.x, GI, x);
}
