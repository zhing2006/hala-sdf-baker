#define THREAD_GROUP_SIZE 512

struct PushConstants {
  uint dispatch_width;
  uint num_of_elements;
  bool need_exclusive_initial_value;
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 1)]]
StructuredBuffer<uint> _input_buffer;

[[vk::binding(1, 1)]]
RWStructuredBuffer<uint> _output_buffer;

[[vk::binding(2, 1)]]
StructuredBuffer<uint> _counters_buffer;

[[vk::binding(3, 1)]]
StructuredBuffer<uint> _aux_buffer;

groupshared uint2 BUCKET[THREAD_GROUP_SIZE];

void prefix_sum(uint id, uint gid, uint x) {
  // Load input into shared memory.
  BUCKET[gid].x = x;
  BUCKET[gid].y = 0;

  // Up sweep.
  uint stride;
  for (stride = 2; stride <= THREAD_GROUP_SIZE; stride <<= 1) {
    GroupMemoryBarrierWithGroupSync();

    if ((gid & (stride - 1)) == (stride - 1)) {
      BUCKET[gid].x += BUCKET[gid - stride / 2].x;
    }
    // Clear the last element
    if (gid == (THREAD_GROUP_SIZE - 1)) {
      BUCKET[gid].x = 0;
    }
  }

  // Down sweep.
  bool n = true;
  [unroll]
  for (stride = THREAD_GROUP_SIZE / 2; stride >= 1; stride >>= 1) {
    GroupMemoryBarrierWithGroupSync();

    uint a = stride - 1;
    uint b = stride | a;

    // Ping-pong between passes.
    if (n) {
      if ((gid & b) == b) {
        BUCKET[gid].y = BUCKET[gid - stride].x + BUCKET[gid].x;
      } else if ((gid & a) == a) {
        BUCKET[gid].y = BUCKET[gid + stride].x;
      } else {
        BUCKET[gid].y = BUCKET[gid].x;
      }
    } else {
      if ((gid & b) == b) {
        BUCKET[gid].x = BUCKET[gid - stride].y + BUCKET[gid].y;
      } else if ((gid & a) == a) {
        BUCKET[gid].x = BUCKET[gid + stride].y;
      } else {
        BUCKET[gid].x = BUCKET[gid].y;
      }
    }
    n = !n;
  }

  // Careful, works for THREAD_GROUP_SIZE = 512 (2^(2n+1))
  _output_buffer[id] = BUCKET[gid].y;
}