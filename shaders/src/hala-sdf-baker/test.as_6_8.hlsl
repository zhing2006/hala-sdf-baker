#include "common.hlsl"
#include "test_common.hlsl"

struct DummyPayLoad {
  uint dummy;
};

// We don't use pay loads in this sample, but the fn call requires one.
groupshared DummyPayLoad dummyPayload;

[numthreads(1, 1, 1)]
void main() {
  DispatchMesh(3, 1, 1, dummyPayload);
}
