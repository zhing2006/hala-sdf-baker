#include "../baker/sdf_baker.hlsl"
#include "prefix_sum.hlsl"

[numthreads(THREAD_GROUP_SIZE, 1, 1)]
void main(uint3 GTId: SV_GroupThreadID, uint GI: SV_GroupIndex, uint3 GId: SV_GroupID) {
  const uint id = GTId.x + GId.x * THREAD_GROUP_SIZE + GId.y * g_push_constants.dispatch_width * THREAD_GROUP_SIZE;

  uint x;
  if (id >= g_push_constants.num_of_elements) {
    x = 0u;
  } else {
    x = _input_buffer[id];
  }

  prefix_sum(id, GI, x);
}