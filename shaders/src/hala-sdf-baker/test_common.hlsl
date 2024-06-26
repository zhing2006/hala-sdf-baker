struct vs_to_ps {
  float4 position: SV_Position;
  [[vk::location(0)]] float2 uv: TEXCOORD0;
  [[vk::location(1)]] float3 color: COLOR0;
};
