#define USE_MESH_SHADER

#include "scene.hlsl"
#include "test_common.hlsl"

#define MAX_VERTEX_COUNT 64
#define MAX_TRIANGLE_COUNT 124
#define VERTICES_PER_THREAD DIV_UP(MAX_VERTEX_COUNT, MESH_SHADER_GROUP_SIZE)
#define TRIANGLE_PER_THREAD DIV_UP(MAX_TRIANGLE_COUNT, MESH_SHADER_GROUP_SIZE)

inline uint3 load_primitive_index(ByteAddressBuffer primitive_index_buffer, uint index) {
  const uint primitive_index = primitive_index_buffer.Load(index * 4);
  const uint triangle_index0 = (primitive_index & 0xFF);
  const uint triangle_index1 = (primitive_index & 0xFF00) >> 8;
  const uint triangle_index2 = (primitive_index & 0xFF0000) >> 16;
  return uint3(triangle_index0, triangle_index1, triangle_index2);
}

[outputtopology("triangle")]
[numthreads(MESH_SHADER_GROUP_SIZE, 1, 1)]
void main(
  out indices uint3 triangles[MAX_TRIANGLE_COUNT],
  out vertices ToFragment vertices[MAX_VERTEX_COUNT],
  in payload MeshShaderPayLoad ms_payload,
  uint3 group_id : SV_GroupID,
  uint3 group_thread_id : SV_GroupThreadID
) {
  // One meshlet per group.
  const uint meshlet_index = ms_payload.meshlet_indices[group_id.x];
  // printf("[MESH SHADER] meshlet_index: %d\n", meshlet_index);
  // printf("[MESH SHADER] VERTEX_PER_THREAD: %d TRIANGLE_PER_THREAD: %d\n", VERTICES_PER_THREAD, TRIANGLE_PER_THREAD);
  // printf("[MESH SHADER] group_thread_id: %d\n", group_thread_id.x);

  const ObjectUniformBuffer per_object_data = g_per_object_data[g_push_constants.object_index];

  StructuredBuffer<Meshlet> meshlet_buffer = g_meshlets[g_push_constants.primitive_index];
  StructuredBuffer<Vertex> vertex_buffer = g_vertices[g_push_constants.primitive_index];
  StructuredBuffer<uint> vertex_index_buffer = g_unique_vertices[g_push_constants.primitive_index];
  ByteAddressBuffer primitive_index_buffer = g_unique_primitives[g_push_constants.primitive_index];

  const Meshlet meshlet = meshlet_buffer[meshlet_index];

  SetMeshOutputCounts(meshlet.num_of_vertices, meshlet.num_of_primitives);

  // Per thread write one vertex.
  const uint vertex_id = group_thread_id.x;
  if (vertex_id < min(meshlet.num_of_vertices, MAX_VERTEX_COUNT)) {
    const uint vertex_index = vertex_index_buffer[meshlet.offset_of_vertices + vertex_id];
    const Vertex vertex = vertex_buffer[vertex_index];
    const float3 position = float3(vertex.position_x, vertex.position_y, vertex.position_z);
    const float2 uv = float2(vertex.tex_coord_x, vertex.tex_coord_y);
    const float3 normal = float3(vertex.normal_x, vertex.normal_y, vertex.normal_z);
    const float3 tangent = float3(vertex.tangent_x, vertex.tangent_y, vertex.tangent_z);

    vertices[vertex_id].position = mul(per_object_data.mvp_mtx, float4(position, 1.0));
    vertices[vertex_id].uv = uv;
    vertices[vertex_id].normal = normalize(mul(float4(normal, 0.0), per_object_data.i_m_mtx).xyz);
    vertices[vertex_id].tangent = normalize(mul(float4(tangent, 0.0), per_object_data.i_m_mtx).xyz);
  }

  // Per thread write two triangles.
  uint triangle_id = group_thread_id.x * 2;
  if (triangle_id < min(meshlet.num_of_primitives, MAX_TRIANGLE_COUNT)) {
    triangles[triangle_id] = load_primitive_index(primitive_index_buffer, meshlet.offset_of_primitives + triangle_id);
    triangle_id++;
    if (triangle_id < min(meshlet.num_of_primitives, MAX_TRIANGLE_COUNT)) {
      triangles[triangle_id] = load_primitive_index(primitive_index_buffer, meshlet.offset_of_primitives + triangle_id);
    }
  }
}