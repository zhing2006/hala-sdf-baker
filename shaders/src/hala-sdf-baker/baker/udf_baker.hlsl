[[vk::binding(0, 0)]]
cbuffer GlobalUniformBuffer {
  uint3 _dimensions;
  uint _num_of_voxels;
  uint _num_of_triangles;
  float _max_distance;
  float _initial_distance;
  float _voxel_size;
  float3 _min_bounds_extended;
  float _padding0;
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

int3 get_voxel_coord(float3 position) {
  float3 voxel_coord = position;
  voxel_coord -= _min_bounds_extended;
  voxel_coord /= _voxel_size;
  return int3((int)voxel_coord.x, (int)voxel_coord.y, (int)voxel_coord.z);
}

float3 get_position(int3 voxel_coord) {
  float3 position = float3(voxel_coord.x, voxel_coord.y, voxel_coord.z);
  position += 0.5;
  position *= _voxel_size;
  position += _min_bounds_extended;

  return position;
}