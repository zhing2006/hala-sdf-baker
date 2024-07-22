#include "baker.hlsl"

[[vk::binding(0, 0)]]
cbuffer GlobalUniformBuffer {
  uint3 _dimensions;
  uint _num_of_voxels;
  uint _num_of_triangles;
  float _initial_distance;
  float _max_size;
  uint _max_dimension;
  float3 _min_bounds_extended;
  float3 _max_bounds_extended;
};

inline uint id3(uint i, uint j, uint k) {
  return i + _dimensions.x * j + _dimensions.x * _dimensions.y * k;
}

inline int3 unpack_id3(uint id) {
  int3 coord;
  coord.z = id / (_dimensions.x * _dimensions.y);
  int remainder = id % (_dimensions.x * _dimensions.y);
  coord.y = remainder / _dimensions.x;
  coord.x = remainder % _dimensions.x;
  return coord;
}

inline uint id3(int3 coord) {
  return id3(coord.x, coord.y, coord.z);
}

// Flip the sign bit of a float to leatest significant bit
inline uint float_flip(float fl) {
  uint f = asuint(fl);
  return (f << 1) | (f >> 31);
}

inline uint float_unflip(uint f2) {
  return (f2 >> 1) | (f2 << 31);
}
