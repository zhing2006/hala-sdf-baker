#include "../baker/sdf_baker.hlsl"
#include "mesh.hlsl"

struct PushConstants {
  uint current_axis;
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(3, 1)]]
cbuffer ConservativeRasterizationUniformBuffer {
  float4x4 _world_to_clip[3];
  float4x4 _clip_to_world[3];
  float _conservative_offset;
};

[[vk::binding(4, 1)]]
StructuredBuffer<uint> _coord_flip_buffer;

[[vk::binding(5, 1)]]
RWStructuredBuffer<float4> _aabb_buffer_rw;

[[vk::binding(6, 1)]]
RWStructuredBuffer<float4> _vertices_buffer_rw;

[numthreads(64, 1, 1)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _num_of_triangles) {
    return;
  }

  // 0 is XY plane, 1 is ZX plane, 2 is YZ plane.
  const uint current_axis = g_push_constants.current_axis;
  if (_coord_flip_buffer[id.x] != current_axis) {
    return;
  }

  uint i;
  float4 vertex_in_clip[3];

  // Get the vertices of the triangle in clip space.
  [unroll(3)]
  for (i = 0; i < 3; i++) {
    vertex_in_clip[i] = mul(_world_to_clip[current_axis], float4(get_vertex_pos(id.x, i), 1.0));
  }

  // Calculate the AABB of the triangle in clip space.
  float4 aabb = float4(1.0, 1.0, -1.0, -1.0);
  aabb.xy = min(aabb.xy, min(vertex_in_clip[0].xy, min(vertex_in_clip[1].xy, vertex_in_clip[2].xy)));
  aabb.zw = max(aabb.xy, max(vertex_in_clip[0].xy, max(vertex_in_clip[1].xy, vertex_in_clip[2].xy)));
  float2 conservative_pixel_size;
  if (current_axis == 0) {
    conservative_pixel_size = float2(_conservative_offset / _dimensions.x, _conservative_offset / _dimensions.y);
  } else if (current_axis == 1) {
    conservative_pixel_size = float2(_conservative_offset / _dimensions.z, _conservative_offset / _dimensions.x);
  } else {
    conservative_pixel_size = float2(_conservative_offset / _dimensions.y, _conservative_offset / _dimensions.z);
  }

  // Add offset of some pixel size to AABB.
  _aabb_buffer_rw[id.x] = aabb + float4(-conservative_pixel_size.x, -conservative_pixel_size.y, conservative_pixel_size.x, conservative_pixel_size.y);

  // Plane's xyz is the normal of the triangle, w is the distance from the origin.
  const float3 normal = normalize(cross(vertex_in_clip[1].xyz - vertex_in_clip[0].xyz, vertex_in_clip[2].xyz - vertex_in_clip[0].xyz));
  const float4 triangle_plane = float4(normal, -dot(vertex_in_clip[0].xyz, normal));

  // Conservative rasterization.
  const float direction = sign(dot(normal, float3(0, 0, 1)));
  float3 edge_plane[3];
  [unroll(3)]
  for (i = 0; i < 3; i++) {
    // Calculate the 2D edge plane normal of the edge.
    // w is for the homogeneous coordinate.
    edge_plane[i] = cross(vertex_in_clip[i].xyw, vertex_in_clip[(i + 2) % 3].xyw);
    // Move the edge plane some pixel size outward.
    edge_plane[i].z -= direction * dot(conservative_pixel_size, abs(edge_plane[i].xy));
  }

  float4 conservative_vertex[3];
  bool is_degenerate = false;
  [unroll(3)]
  for (i = 0; i < 3; i++) {
    // Prefill the vertex output buffer.
    _vertices_buffer_rw[3 * id.x + i] = float4(0, 0, 0, 1);

    // Calculate the intersection point(new vertex position) of the three edge planes.
    conservative_vertex[i].xyw = cross(edge_plane[i], edge_plane[(i + 1) % 3]);

    // Normalize the vertex position.
    if (abs(conservative_vertex[i].w) < CONSERVATIVE_RASTER_EPS) {
      is_degenerate |= true;
    } else {
      is_degenerate |= false;
      conservative_vertex[i] /= conservative_vertex[i].w; // after this, w is 1.
    }
  }
  if (is_degenerate)
    return;

  // Calculate the new z-Coordinate derived from a point on a triangle plane.
  // The triangle plane equation is ax + by + cz + d = 0, so z = -(ax + by + d) / c.
  [unroll(3)]
  for (i = 0; i < 3; i++) {
    conservative_vertex[i].z = -(triangle_plane.x * conservative_vertex[i].x + triangle_plane.y * conservative_vertex[i].y + triangle_plane.w) / triangle_plane.z;
    // Write the vertices to the out buffer.
    _vertices_buffer_rw[3 * id.x + i] = conservative_vertex[i];
    // _vertices_buffer_rw[3 * id.x + i] = mul(_clip_to_world[current_axis], conservative_vertex[i]);
  }
}
