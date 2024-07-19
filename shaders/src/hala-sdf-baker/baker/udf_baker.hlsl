[[vk::binding(0, 0)]]
cbuffer GlobalUniformBuffer {
  float4x4 _i_m_mtx;
  uint3 _dimensions;
  uint _num_of_voxels;
  uint _num_of_triangles;
  float _max_distance;
  float _initial_distance;
};

inline uint id3(uint i, uint j, uint k) {
  return i + _dimensions.x * j + _dimensions.x * _dimensions.y * k;
}

inline uint id3(int3 coord) {
  return id3(coord.x, coord.y, coord.z);
}

// Flip the sign bit of a float to leatest significant bit
uint float_flip(float fl) {
  uint f = asuint(fl);
  return (f << 1) | (f >> 31);
}

uint float_unflip(uint f2) {
  return (f2 >> 1) | (f2 << 31);
}