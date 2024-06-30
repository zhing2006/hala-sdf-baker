#include "common.hlsl"
#include "test_common.hlsl"

#define MAX_MESHLET_SIZE 128
#define GROUP_SIZE MAX_MESHLET_SIZE

[outputtopology("triangle")]
[numthreads(1, 1, 1)]
void main(
  out indices uint3 triangles[124],
  out vertices to_ps vertices[64],
  in payload MeshShaderPayLoad ms_payload,
  uint threadIndex : SV_GroupIndex,
  uint3 groupId : SV_GroupID,
  uint3 groupThreadId : SV_GroupThreadID,
  uint3 dispatchThreadID : SV_DispatchThreadID
) {
  const uint meshlet_index = dispatchThreadID.x;

  const ObjectUniformBuffer per_object_data = g_per_object_data[g_push_constants.object_index];

  StructuredBuffer<Meshlet> meshlet_buffer = g_meshlets[g_push_constants.primitive_index];
  StructuredBuffer<Vertex> vertex_buffer = g_vertices[g_push_constants.primitive_index];
  StructuredBuffer<uint> vertex_index_buffer = g_unique_vertices[g_push_constants.primitive_index];
  ByteAddressBuffer primitive_index_buffer = g_unique_primitives[g_push_constants.primitive_index];

  const Meshlet meshlet = meshlet_buffer[meshlet_index];
  SetMeshOutputCounts(meshlet.num_of_vertices, meshlet.num_of_primitives);

  for (uint i = 0; i < meshlet.num_of_vertices; i++) {
    const uint vertex_index = vertex_index_buffer[meshlet.offset_of_vertices + i];
    const Vertex vertex = vertex_buffer[vertex_index];
    const float3 position = float3(vertex.position_x, vertex.position_y, vertex.position_z);
    const float2 uv = float2(vertex.tex_coord_x, vertex.tex_coord_y);
    const float3 normal = normalize(float3(vertex.normal_x, vertex.normal_y, vertex.normal_z));
    const float3 tangent = normalize(float3(vertex.tangent_x, vertex.tangent_y, vertex.tangent_z));

    vertices[i].position = mul(per_object_data.mvp_mtx, float4(position, 1.0));
    vertices[i].uv = uv;
    vertices[i].normal = normal;
    vertices[i].tangent = tangent;
  }

  for (uint i = 0; i < meshlet.num_of_primitives; i++) {
    const uint primitive_index = primitive_index_buffer.Load(meshlet.offset_of_primitives + i);
    triangles[i] = uint3(
      (primitive_index & 0xFF),
      (primitive_index & 0xFF00) >> 8,
      (primitive_index & 0xFF0000) >> 16
    );
  }

  // const float3 offset = float3(0.0f, 0.0f, (float)dispatchThreadID.x * 0.1);

  // SetMeshOutputCounts(3, 1);
  // for (uint i = 0; i < 3; i++) {
  //   const uint vertex_index = vertex_index_buffer[meshlet.offset_of_vertices + i];
  //   const Vertex vertex = vertex_buffer[vertex_index];
  //   const float3 position = float3(vertex.position_x, vertex.position_y, vertex.position_z);
  //   const float2 uv = float2(vertex.tex_coord_x, vertex.tex_coord_y);
  //   const float3 normal = normalize(float3(vertex.normal_x, vertex.normal_y, vertex.normal_z));
  //   const float3 tangent = normalize(float3(vertex.tangent_x, vertex.tangent_y, vertex.tangent_z));

  //   vertices[i].position = mul(per_object_data.mvp_mtx, float4(position, 1.0));
  //   vertices[i].uv = float2(0.0f, 0.0f);
  //   vertices[i].normal = float3(0.0f, 0.0f, 1.0f);
  //   vertices[i].tangent = float3(1.0f, 0.0f, 0.0f);
  // }

  // for (uint i = 0; i < 1; i++) {
  //   const uint primitive_index = primitive_index_buffer.Load(i);
  //   triangles[i] = uint3(
  //     (primitive_index & 0xFF),
  //     (primitive_index & 0xFF00) >> 8,
  //     (primitive_index & 0xFF0000) >> 16
  //   );
  // }
}