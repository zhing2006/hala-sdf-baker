#include "sdf_visualization.hlsl"

ToFragment main(uint vertex_id: SV_VertexID, uint instance_id : SV_InstanceID) {
  ToFragment output = (ToFragment)0;

  // if (vertex_id == 0) {
  //   printf("Camera position: %f %f %f\n", _camera_position.x, _camera_position.y, _camera_position.z);
  //   printf("Offset: %f\n", _offset);
  //   printf("Dimensions: %d %d %d\n", _dimensions[0], _dimensions[1], _dimensions[2]);
  //   printf("Voxel size: %f\n", _voxel_size);
  // }

  const float3 center = float3(g_push_constants.center[0], g_push_constants.center[1], g_push_constants.center[2]);
  const float3 extents = float3(g_push_constants.extents[0], g_push_constants.extents[1], g_push_constants.extents[2]);

  float3 vertex = g_vertices[instance_id * 2 + vertex_id];
  vertex = vertex * extents;
  output.position = mul(_mvp_mtx, float4(vertex + center, 1.0));

  output.position_ws = mul(_m_mtx, float4(vertex, 1.0)).xyz;

  output.color = float4(g_push_constants.color[0], g_push_constants.color[1], g_push_constants.color[2], g_push_constants.color[3]);

  return output;
}