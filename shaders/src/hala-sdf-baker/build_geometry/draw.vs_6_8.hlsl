#include "../baker/sdf_baker.hlsl"
#include "draw.hlsl"

struct VertexInput {
  [[vk::location(0)]] uint vertex_id: SV_VertexID;
};

ToFragment main(VertexInput input) {
  ToFragment output = (ToFragment)0;

  const float4 pos = _vertices_buffer[input.vertex_id];
  output.triangle_id = input.vertex_id / 3;
  if (_coord_flip_buffer[output.triangle_id] != g_push_constants.current_axis) {
    output.position = float4(-1, -1, -1, -1);
  } else {
    output.position = pos;
  }

  return output;
}