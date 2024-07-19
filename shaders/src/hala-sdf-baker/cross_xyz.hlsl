struct PushConstants {
  float4x4 mvp;
  float center[3];
  float extents[3];
};

[[vk::push_constant]]
PushConstants g_push_constants;

struct ToFragment {
  float4 position: SV_Position;
  [[vk::location(0)]] float3 uvw: TEXCOORD0;
};

static const float3 g_vertices[18] = {
  // XZ plane.
  float3(-1.0, 0.0, -1.0), float3( 1.0, 0.0, -1.0), float3(-1.0, 0.0,  1.0),
  float3(-1.0, 0.0,  1.0), float3( 1.0, 0.0, -1.0), float3( 1.0, 0.0,  1.0),
  // XY plane.
  float3(-1.0, -1.0, 0.0), float3( 1.0, -1.0, 0.0), float3(-1.0,  1.0, 0.0),
  float3(-1.0,  1.0, 0.0), float3( 1.0, -1.0, 0.0), float3( 1.0,  1.0, 0.0),
  // YZ plane.
  float3(0.0, -1.0, -1.0), float3(0.0,  1.0, -1.0), float3(0.0, -1.0,  1.0),
  float3(0.0, -1.0,  1.0), float3(0.0,  1.0, -1.0), float3(0.0,  1.0,  1.0),
};