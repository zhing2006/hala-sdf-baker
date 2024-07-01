#include "common.hlsl"
#include "test_common.hlsl"

#define TASK_SHADER_GROUP_SIZE 32

// We don't use pay loads in this sample, but the fn call requires one.
groupshared MeshShaderPayLoad ms_payload;

[numthreads(TASK_SHADER_GROUP_SIZE, 1, 1)]
void main(uint3 dispatchThreadID : SV_DispatchThreadID) {
  // One meshlet per thread.
  if (dispatchThreadID.x >= g_push_constants.meshlet_count) {
    return;
  }

  // printf("[TASK SHADER] dispatchThreadID: %d\n", dispatchThreadID.x);

  // One meshlet per group.
  DispatchMesh(g_push_constants.meshlet_count, 1, 1, ms_payload);
}
