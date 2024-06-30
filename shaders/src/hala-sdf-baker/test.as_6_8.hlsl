#include "common.hlsl"
#include "test_common.hlsl"

// We don't use pay loads in this sample, but the fn call requires one.
groupshared MeshShaderPayLoad ms_payload;

[numthreads(1, 1, 1)]
void main(uint3 dispatchThreadID : SV_DispatchThreadID) {
  DispatchMesh(g_push_constants.meshlet_count, 1, 1, ms_payload);
}
