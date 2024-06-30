#include "common.hlsl"
#include "test_common.hlsl"

// We don't use pay loads in this sample, but the fn call requires one.
groupshared MeshShaderPayLoad ms_payload;

[numthreads(1, 1, 1)]
void main(uint3 dispatchThreadID : SV_DispatchThreadID) {
  ms_payload.meshlet_index = dispatchThreadID.x;
  DispatchMesh(1, 1, 1, ms_payload);
}
