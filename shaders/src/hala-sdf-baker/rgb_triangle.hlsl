struct ToFragment {
  float4 position: SV_Position;
  [[vk::location(0)]] float3 color: COLOR0;
};