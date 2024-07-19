struct ToFragment {
  float4 position: SV_Position;
  [[vk::location(0)]] float2 uv: TEXCOORD0;
};