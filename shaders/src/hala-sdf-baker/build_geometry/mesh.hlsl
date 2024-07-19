[[vk::binding(0, 1)]]
cbuffer MeshUniformBuffer {
  int _vertex_position_offset;
  int _vertex_stride;
  int _index_stride;
};

[[vk::binding(1, 1)]]
ByteAddressBuffer _indices_buffer;

[[vk::binding(2, 1)]]
ByteAddressBuffer _vertices_buffer;

uint load_index16(uint id) {
  const uint entry_offset = id & 1u;
  id = id >> 1;
  const uint read = _indices_buffer.Load(id << 2);
  return entry_offset == 1 ? read >> 16 : read & 0xffff;
}

uint load_index32(uint id) {
  return _indices_buffer.Load(id << 2);
}

float3 get_vertex_pos(uint triangle_id, uint vertex_id) {
  const uint index_id = (3 * triangle_id + vertex_id);
  const uint index = _index_stride == 2 ? load_index16(index_id) : load_index32(index_id);
  const uint vert_index = _vertex_position_offset + index * _vertex_stride;
  const uint3 vert_raw = _vertices_buffer.Load3(vert_index);
  return asfloat(vert_raw);
}
