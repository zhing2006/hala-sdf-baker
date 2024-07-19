#include "sdf_visualization.hlsl"

inline float sample_surface(float3 position) {
  return g_texture.SampleLevel(g_sampler, position, 0).r + _offset;
}

void ray_marching(float3 ray_origin, float3 ray_direction, float t_min, float t_max, float min_surface_distance, out float4 color, out float depth) {
  const float3 inv_scale = float3(1.0 / g_push_constants.extents[0], 1.0 / g_push_constants.extents[1], 1.0 / g_push_constants.extents[2]);
  float t = t_min;
  for (int i = 0; i < 2048; i++) {
    const float3 position = ray_origin + ray_direction * t;
    float3 uvw = position * inv_scale;
    uvw = uvw * 0.5 + 0.5;
    const float sampled_distance = sample_surface(uvw);

    if (sampled_distance < min_surface_distance || t > t_max) {
      const float3 delta_shift = _inv_resolution * inv_scale; // One voxel in uvw space.
      const float3 delta = float3(sample_surface(uvw + float3(delta_shift.x, 0, 0)),
        sample_surface(uvw + float3(0, delta_shift.y, 0)),
        sample_surface(uvw + float3(0, 0, delta_shift.z))) - sampled_distance;
      const float3 normal = normalize(float3(delta.x / delta_shift.x, delta.y / delta_shift.y, delta.z / delta_shift.z));

      const float3 intersection_point = ray_origin + ray_direction * min(t, t_max);
      const float4 clip_position = mul(_vp_mtx, float4(intersection_point, 1.0));
      color = float4(normal * 0.5 + 0.5, 1);
      depth = clip_position.z / clip_position.w;
      break;
    }

    t += sampled_distance;
  }

  if (t > t_max) {
    color = float4(0, 0, 0, 0);
    depth = 0;
  }
}

FragmentOutput main(ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

  float3 ray_origin = _camera_position;
  float3 ray_direction = normalize(input.position_ws - ray_origin);
  ray_origin = mul(_i_m_mtx, float4(ray_origin, 1.0)).xyz;
  ray_direction = mul(_i_m_mtx, float4(ray_direction, 0.0)).xyz;

  const float3 half_box_size = float3(g_push_constants.extents[0], g_push_constants.extents[1], g_push_constants.extents[2]);
  float2 intersection = ray_box_intersection(ray_origin, ray_direction, half_box_size);
  if (intersection.y < 0.0) {
    output.color = ERROR_COLOR;
    output.depth = 1.0 - input.position.z / input.position.w;
  } else {
    const float min_surface_distance = _inv_resolution * _inv_resolution;
    ray_marching(ray_origin, ray_direction, intersection.x, intersection.y, min_surface_distance, output.color, output.depth);
    output.color *= input.color;
  }

  return output;
}