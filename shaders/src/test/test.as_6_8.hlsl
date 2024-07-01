#include "common.hlsl"
#include "test_common.hlsl"

// We don't use pay loads in this sample, but the fn call requires one.
groupshared MeshShaderPayLoad ms_payload;

[numthreads(TASK_SHADER_GROUP_SIZE, 1, 1)]
void main(uint3 group_id : SV_GroupID, uint3 dispatch_thread_id : SV_DispatchThreadID) {
  // One meshlet per thread.
  if (dispatch_thread_id.x >= g_push_constants.meshlet_count) {
    return;
  }

  ms_payload.task_group_id = group_id.x;

  // printf("[TASK SHADER] dispatch_thread_id: %d\n", dispatch_thread_id.x);

  // One meshlet per group.
  DispatchMesh((g_push_constants.meshlet_count + g_push_constants.dispatch_size_x - 1) / g_push_constants.dispatch_size_x, 1, 1, ms_payload);
}
