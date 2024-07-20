#include "../baker/udf_baker.hlsl"

struct PushConstants {
  int offset;
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 1)]]
StructuredBuffer<uint> _jump_buffer;

[[vk::binding(1, 1)]]
RWStructuredBuffer<uint> _jump_buffer_rw;

[numthreads(8, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z) {
    return;
  }
}