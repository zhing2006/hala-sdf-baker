struct PushConstants {
  float4x4 mvp;
  float center[3];
  float extents[3];
  float color[4];
};

[[vk::push_constant]]
PushConstants g_push_constants;

struct ToFragment {
  float4 position: SV_Position;
  [[vk::location(0)]] float4 color: COLOR0;
};

static const float3 g_vertices[24] = {
  float3(-1, -1, -1), float3( 1, -1, -1),
  float3(-1, -1, -1), float3(-1,  1, -1),
  float3(-1, -1, -1), float3(-1, -1,  1),
  float3( 1, -1, -1), float3( 1,  1, -1),
  float3( 1, -1, -1), float3( 1, -1,  1),
  float3(-1,  1, -1), float3( 1,  1, -1),
  float3(-1,  1, -1), float3(-1,  1,  1),
  float3( 1,  1, -1), float3( 1,  1,  1),
  float3(-1, -1,  1), float3( 1, -1,  1),
  float3(-1, -1,  1), float3(-1,  1,  1),
  float3( 1, -1,  1), float3( 1,  1,  1),
  float3(-1,  1,  1), float3( 1,  1,  1),
};