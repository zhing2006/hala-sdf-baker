struct ToFragment {
  float4 position: SV_Position;
  [[vk::location(0)]] float2 uv: TEXCOORD0;
  [[vk::location(1)]] float3 normal: TEXCOORD1;
  [[vk::location(2)]] float3 tangent: TEXCOORD2;
};
