#define THREAD_GROUP_SIZE 512
#define COMMON_EPS 1e-6
#define BARY_EPS 1e-5
#define CONSERVATIVE_RASTER_EPS 1e-6
#define INTERSECT_EPS 0
#define PI 3.14159265359

struct Triangle {
  float3 a, b, c;
};

[[vk::binding(0, 0)]]
cbuffer GlobalUniformBuffer {
  uint3 _dimensions;
  uint _max_dimension;
  uint _upper_bound_count;
  uint _num_of_triangles;
  float _max_extent;
  float _padding0;
  float3 _min_bounds_extended;
  float _padding1;
  float3 _max_bounds_extended;
  float _padding2;
};

inline uint id3(uint i, uint j, uint k) {
  return i + _dimensions.x * j + _dimensions.x * _dimensions.y * k;
}

inline uint id3(int3 coord) {
  return id3(coord.x, coord.y, coord.z);
}