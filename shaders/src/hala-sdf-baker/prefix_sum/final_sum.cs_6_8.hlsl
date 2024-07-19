#include "prefix_sum.hlsl"

[numthreads(THREAD_GROUP_SIZE, 1, 1)]
void main(uint3 GTId: SV_GroupThreadID, uint3 GId: SV_GroupID) {
  const uint id = GTId.x + GId.x * THREAD_GROUP_SIZE + GId.y * g_push_constants.dispatch_width * THREAD_GROUP_SIZE;
  if (id >= g_push_constants.num_of_elements)
    return;

  const uint flattened_group_id = GId.x + g_push_constants.dispatch_width * GId.y;

  if (g_push_constants.need_exclusive_initial_value) {
    // Exclusive prefix sum by subtracting the initial value of the counter.
    _output_buffer[id] = _input_buffer[id] + _aux_buffer[flattened_group_id] - _counters_buffer[id];
  } else {
    _output_buffer[id] = _input_buffer[id] + _aux_buffer[flattened_group_id];
  }
}