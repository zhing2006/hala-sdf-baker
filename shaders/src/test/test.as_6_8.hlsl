#define USE_MESH_SHADER

#include "scene.hlsl"
#include "test_common.hlsl"

// We don't use pay loads in this sample, but the fn call requires one.
groupshared MeshShaderPayLoad ms_payload;

[numthreads(TASK_SHADER_GROUP_SIZE, 1, 1)]
void main(
  uint3 group_id : SV_GroupID,
  uint3 group_thread_id : SV_GroupThreadID,
  uint3 dispatch_thread_id : SV_DispatchThreadID
) {
  // One meshlet per thread.
  const uint meshlet_index = dispatch_thread_id.x;
  if (meshlet_index >= g_push_constants.meshlet_count) {
    return;
  }

  const ObjectUniformBuffer per_object_data = g_per_object_data[g_push_constants.object_index];
  StructuredBuffer<Meshlet> meshlet_buffer = g_meshlets[g_push_constants.primitive_index];
  const Meshlet meshlet = meshlet_buffer[meshlet_index];
  const float3 camera_position = cameras[0].position;

  ms_payload.task_group_id = group_id.x;

  // printf("[TASK SHADER] dispatch_thread_id: %d\n", dispatch_thread_id.x);

  const float3 cone_apex = mul(per_object_data.m_mtx, float4(meshlet.cone_apex, 1.0)).xyz;
  const float3 cone_axis = normalize(mul(float4(meshlet.cone_axis, 0.0), per_object_data.i_m_mtx).xyz);
  if (dot(normalize(cone_apex - camera_position), cone_axis) >= meshlet.cone_cutoff)
    ms_payload.is_visibles[group_thread_id.x] = false;
  else
    ms_payload.is_visibles[group_thread_id.x] = true;

  // One meshlet per group.
  DispatchMesh((g_push_constants.meshlet_count + g_push_constants.dispatch_size_x - 1) / g_push_constants.dispatch_size_x, 1, 1, ms_payload);
}
