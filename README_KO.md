# hala-sdf-baker
[![License](https://img.shields.io/badge/License-GPL3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)
[![MSRV](https://img.shields.io/badge/rustc-1.70.0+-ab6000.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

[English](README.md) | [中文](README_CN.md) | [日本語](README_JP.md) | [한국어](README_KO.md)

## 소개

현대 컴퓨터 그래픽스와 게임 개발에서 널리 필수적인 기술로 여겨지는 것이 있습니다. 그것은 바로 유도 거리 필드(Signed Distance Fields, SDF)와 무유도 거리 필드(Unsigned Distance Fields, UDF)를 사용하는 것입니다. SDF와 UDF는 복잡한 기하학적 형태를 표현하고 조작하는 데 효율적이고 강력한 수단을 제공합니다. 이들은 렌더링, 충돌 감지, 모델 생성 등 여러 분야에서 중요한 역할을 합니다.

SDF는 각 공간의 점에 대해 해당 점에서 가장 가까운 표면까지의 유도 거리를 나타내는 실수 값을 할당하는 전형적인 표현 방법입니다. 이러한 구조는 효율적으로 형태를 모델링하는 데 사용할 수 있을 뿐만 아니라 평활화, 팽창 또는 축소와 같은 기하학적 작업을 수행하는 데도 사용할 수 있습니다. 이에 반해, UDF는 표면까지의 절대 최단 거리를 기록하며, 이는 불규칙하거나 복잡한 위상을 가진 모델을 처리할 때 특히 유용합니다.

SDF와 UDF는 단순한 데이터 구조가 아니라 다차원 공간에서 형태를 표현하는 방법입니다. 비디오 게임 개발에서는 SDF를 이용한 실시간 그림자 계산과 환경 광 차폐가 인기 있는 기술이 되었습니다. 이는 SDF가 광선과 기하학적 표면의 접촉점을 빠르게 결정할 수 있어 부드러운 그림자와 기타 시각 효과를 효과적으로 생성할 수 있기 때문입니다. 또한, 실시간 그래픽에서 SDF를 사용하면 캐릭터의 동적 변형이나 개발 중 흔히 볼 수 있는 파괴 효과와 같은 효율적인 기하학적 모델링 및 수정이 가능합니다. 산업 비전 및 과학 시각화 분야에서는 UDF가 형태 재구성과 데이터 적합에 자주 사용되며, 특히 스캐닝 장치나 기타 측정 장치에서 데이터를 처리할 때 유용합니다. 정확한 UDF를 구축함으로써 연구자들은 이산 데이터 포인트 집합에서 연속적인 3D 표면을 추론할 수 있으며, 이는 복잡한 생물 형태나 기타 과학적 구조를 재구성하는 데 매우 중요합니다. 본 프로젝트에서는 Rust와 Vulkan을 사용하여 3D Mesh 데이터를 SDF와 UDF로 베이킹하는 방법을 구현할 것입니다.

![Image Intro](images/intro.png)

그림 1: https://arxiv.org/abs/2011.02570에서 가져옴. 상단은 UDF로, 표면까지의 절대 최단 거리만 기록합니다. 하단은 SDF로, 최단 거리 외에도 부호를 통해 "내부"인지 "외부"인지를 나타냅니다.

## 개발 환경 설정

현재 전체 개발 환경은 Windows 플랫폼에서 RTX 4090과 Radeon 780M에서만 테스트되었습니다(개인 장비 제한으로 인해 더 많은 호환성을 테스트할 수 없었습니다). `hala-gfx`, `hala-renderer`, `hala-imgui`를 기반으로 개발되었습니다.

* `hala-gfx`는 Vulkan 호출 및 래핑을 담당합니다.
* `hala-renderer`는 glTF 파일에서 Mesh 정보를 읽어 GPU에 업로드하는 역할을 합니다.
* `hala-imgui`는 imGUI의 Rust 브리지로, 사용자 인터페이스의 표시 및 상호작용을 담당합니다.

Rust 1.70+을 설치합니다. 이미 설치되어 있다면 `rustup update`로 최신 버전으로 업데이트합니다. `git clone --recursive` 명령어를 사용하여 저장소 및 서브모듈을 클론합니다. `cargo build`로 디버그 버전을 빌드하거나 `cargo build -r`로 릴리스 버전을 빌드합니다.

컴파일이 완료되면 바로 실행할 수 있습니다.

    ./target/(debug 또는 release)/hala-sdf-baker -c conf/config.yaml -o ./out/output.txt

"Bake" 버튼을 클릭하여 베이킹을 수행하고, "Save" 버튼을 클릭하여 베이킹 결과를 "./out/output.txt"에 저장할 수 있습니다.

출력 파일 형식은 다음과 같습니다:

    X축 해상도 Y축 해상도 Z축 해상도
    1번 보셀의 값
    2번 보셀의 값
    ...
    n-1번 보셀의 값
    n번 보셀의 값

## UDF 베이킹

알고리즘 구현에서 UDF는 상대적으로 간단합니다. 여기서는 먼저 UDF 베이킹에 대해 설명하겠습니다.

### 첫 번째 단계: 초기화

베이킹을 시작하기 전에 리소스를 할당해야 합니다. UDF는 복셀 저장소로, 3D 형식으로 이미지를 저장할 수도 있고, 선형 형식으로 버퍼를 저장할 수도 있습니다. 여기서는 후속 시각적 디버깅을 위해 3D 형식으로 저장합니다.

베이킹 전에 몇 가지 베이킹 매개변수를 설정해야 합니다. 그 구체적인 역할은 다음 코드의 주석에 설명되어 있습니다.
```rust
pub selected_mesh_index: i32, // glTF에 여러 Mesh 데이터가 저장될 수 있으며, 이 필드는 베이킹할 Mesh의 인덱스를 결정합니다.
pub max_resolution: i32,      // 베이킹 출력 복셀의 최대 축 해상도입니다. 예를 들어 크기가 (1, 2, 4)인 Mesh 범위가 있고, 이 필드가 64인 경우 최종 복셀 해상도는 [16, 32, 64]가 됩니다.
pub surface_offset: f32,      // 이 오프셋 값은 최종 베이킹된 데이터에 추가됩니다.
pub center: [f32; 3],         // 베이킹할 데이터의 BoundingBox 중심 위치입니다.
pub desired_size: [f32; 3],   // Mesh의 BoundingBox 크기, max_resolution 및 지정된 여유 공간 padding을 기반으로 계산된 계획된 베이킹 공간 크기입니다.
pub actual_size: [f32; 3],    // desired_size를 복셀 크기의 정수 배수로 조정한 크기로, 최종 저장 데이터의 크기입니다.
pub padding: [f32; 3],        // Mesh의 BoundingBox 외부에 추가할 복셀 수입니다.
```

center와 desired_size의 계산 방법은 다음과 같습니다:
```rust
fn fit_box_to_bounds(&mut self) {
  // 베이킹할 Mesh의 BoundingBox를 가져옵니다.
  let bounds = self.get_selected_mesh_bounds().unwrap();

  // 가장 긴 변의 길이를 계산합니다.
  let max_size = bounds.get_size().iter().fold(0.0, |a: f32, b| a.max(*b));
  // 지정된 최대 해상도를 통해 단일 복셀의 크기를 계산합니다.
  let voxel_size = max_size / self.settings.max_resolution as f32;
  // 복셀 크기를 기반으로 외부 경계 크기를 계산합니다.
  let padding = [
    self.settings.padding[0] * voxel_size,
    self.settings.padding[1] * voxel_size,
    self.settings.padding[2] * voxel_size,
  ];

  // 최종적으로 전체 베이킹 영역의 중심과 크기를 얻습니다.
  let center = [
    bounds.center[0],
    bounds.center[1],
    bounds.center[2]
  ];
  let size = [
    (bounds.extents[0] + padding[0]) * 2.0,
    (bounds.extents[1] + padding[1]) * 2.0,
    (bounds.extents[2] + padding[2]) * 2.0
  ];
  self.settings.center = center;
  self.settings.desired_size = size;
}
```

actual_size의 계산 방법은 다음과 같습니다:
```rust
fn snap_box_to_bounds(&mut self) {
  // 베이킹할 영역의 가장 긴 변의 길이를 계산합니다.
  let max_size = self.settings.desired_size.iter().fold(0.0, |a: f32, b| a.max(*b));
  // 가장 긴 변이 있는 축을 참조 축으로 설정합니다. 이 축의 복셀 수는 설정된 최대 해상도 값이 됩니다.
  let ref_axis = if max_size == self.settings.desired_size[0] {
    Axis::X
  } else if max_size == self.settings.desired_size[1] {
    Axis::Y
  } else {
    Axis::Z
  };

  // 참조 축에 따라 단일 복셀 크기를 계산한 후, 복셀 크기의 정수 배수로 베이킹 영역의 크기를 계산합니다.
  self.settings.actual_size = match ref_axis {
    Axis::X => {
      let dim_x = (self.settings.max_resolution as f32 * self.settings.desired_size[0] / max_size).round().max(1.0);
      let dim_y = (self.settings.max_resolution as f32 * self.settings.desired_size[1] / max_size).ceil().max(1.0);
      let dim_z = (self.settings.max_resolution as f32 * self.settings.desired_size[2] / max_size).ceil().max(1.0);
      let voxel_size = max_size / dim_x;
      [dim_x * voxel_size, dim_y * voxel_size, dim_z * voxel_size]
    },
    Axis::Y => {
      let dim_x = (self.settings.max_resolution as f32 * self.settings.desired_size[0] / max_size).ceil().max(1.0);
      let dim_y = (self.settings.max_resolution as f32 * self.settings.desired_size[1] / max_size).round().max(1.0);
      let dim_z = (self.settings.max_resolution as f32 * self.settings.desired_size[2] / max_size).ceil().max(1.0);
      let voxel_size = max_size / dim_y;
      [dim_x * voxel_size, dim_y * voxel_size, dim_z * voxel_size]
    },
    Axis::Z => {
      let dim_x = (self.settings.max_resolution as f32 * self.settings.desired_size[0] / max_size).ceil().max(1.0);
      let dim_y = (self.settings.max_resolution as f32 * self.settings.desired_size[1] / max_size).ceil().max(1.0);
      let dim_z = (self.settings.max_resolution as f32 * self.settings.desired_size[2] / max_size).round().max(1.0);
      let voxel_size = max_size / dim_z;
      [dim_x * voxel_size, dim_y * voxel_size, dim_z * voxel_size]
    },
  }
}
```

다음으로, 전체 베이킹 과정에서 필요한 몇 가지 매개변수를 저장하기 위해 전역 UBO를 준비합니다. 구체적인 내용은 다음 코드의 주석에 설명되어 있습니다.
```rust
pub struct GlobalUniform {
  pub dimensions: [u32; 3],   // 베이킹할 Mesh의 BoundingBox 정보와 베이킹 복셀 최대 해상도를 기반으로 계산된 세 축의 크기입니다.
  pub num_of_voxels: u32,     // 전체 복셀의 수로, 값은 dimensions[0] * dimensions[1] * dimensions[2]입니다.
  pub num_of_triangles: u32,  // 베이킹할 Mesh의 총 삼각형 수입니다.
  pub initial_distance: f32,  // UDF의 초기 값입니다. 전체 베이킹 영역의 가장 긴 변의 길이를 기반으로, 정규화된 베이킹 BoundingBox의 대각선 길이의 1.01배입니다 (전체 UDF에서 이 값을 초과하는 값은 없습니다).
  pub max_size: f32,          // 전체 베이킹 영역의 가장 긴 변의 길이입니다.
  pub max_dimension: u32,     // 전체 복셀 공간의 가장 긴 변의 복셀 수입니다.
  pub center: [f32; 3],       // 베이킹 영역 BoundingBox의 중심 좌표입니다.
  pub extents: [f32; 3],      // 베이킹 영역 BoundingBox의 반 길이입니다.
}
```

위에서 계산된 복셀 공간의 세 축의 복셀 수를 기반으로 Image 리소스를 생성합니다. 여기서 Usage를 Storage로 설정한 이유는 이후 Shader에서 이를 쓰기 위해서이며, Sampled로 설정한 이유는 읽기 위해서입니다.
```rust
hala_gfx::HalaImage::new_3d(
  Rc::clone(&self.resources.context.borrow().logical_device),
  hala_gfx::HalaImageUsageFlags::SAMPLED | hala_gfx::HalaImageUsageFlags::STORAGE,
  hala_gfx::HalaFormat::R32_SFLOAT,
  dimensions[0],
  dimensions[1],
  dimensions[2],
  hala_gfx::HalaMemoryLocation::GpuOnly,
  "distance_texture.image3d",
)?
```

### 두 번째 단계: 초기 값 입력

이 단계는 가장 간단합니다. 여기서 주의할 점은 초기 거리의 float 형식이 아닌 uint 형식으로 쓰인다는 점입니다. 이는 다음 Shader에서 자세히 설명됩니다.
```hlsl
_distance_texture_rw[int3(id.x, id.y, id.z)] = float_flip(_initial_distance);
```

다음은 Mesh의 모든 삼각형을 순회하는 과정입니다. id.x는 현재 순회 중인 삼각형의 인덱스 번호입니다.
```hlsl
Triangle tri_uvw;
tri_uvw.a = (get_vertex_pos(id.x, 0) - _center + _extents) / _max_size;
tri_uvw.b = (get_vertex_pos(id.x, 1) - _center + _extents) / _max_size;
tri_uvw.c = (get_vertex_pos(id.x, 2) - _center + _extents) / _max_size;
```
먼저 get_vertex_pos 함수를 통해 Mesh의 index buffer와 vertex buffer에서 정점의 위치 정보를 읽어옵니다.
그리고 center와 extents를 통해 정점을 3D 공간의 첫 번째 사분면으로 평행 이동시킵니다.
마지막으로 max_size 값을 기반으로 [0, 1] 범위의 uvw 공간으로 정규화합니다.

| 단계 | 설명 |
|------|------|
|![Image Bound 0](images/bound_0.png)| *원래 영역* |
|![Image Bound 1](images/bound_1.png)| *첫 번째 사분면으로 평행 이동* |
|![Image Bound 2](images/bound_2.png)| *UVW 공간으로 정규화* |

다음으로 삼각형이 커버하는 영역의 AABB를 계산한 후, _max_dimension을 통해 복셀 공간으로 변환하고 한 겹 더 확장합니다.
```hlsl
const float3 aabb_min = min(tri_uvw.a, min(tri_uvw.b, tri_uvw.c));
const float3 aabb_max = max(tri_uvw.a, max(tri_uvw.b, tri_uvw.c));
int3 voxel_min = int3(aabb_min * _max_dimension) - GRID_MARGIN;
int3 voxel_max = int3(aabb_max * _max_dimension) + GRID_MARGIN;
voxel_min = max(0, min(voxel_min, int3(_dimensions) - 1));
voxel_max = max(0, min(voxel_max, int3(_dimensions) - 1));
```

마지막으로 AABB가 커버하는 모든 복셀을 순회하며, 복셀 중심이 삼각형까지의 거리를 계산하고 이를 Distance Texture에 씁니다.
```hlsl
for (int z = voxel_min.z; z <= voxel_max.z; ++z) {
  for (int y = voxel_min.y; y <= voxel_max.y; ++y) {
    for (int x = voxel_min.x; x <= voxel_max.x; ++x) {
      const float3 voxel_coord = (float3(x, y, z) + float3(0.5, 0.5, 0.5)) / _max_dimension;
      float distance = point_distance_to_triangle(voxel_coord, tri_uvw);
      uint distance_as_uint = float_flip(distance);
      InterlockedMin(_distance_texture_rw[int3(x, y, z)], distance_as_uint);
    }
  }
}
```
여기서 InterlockedMin 원자 비교 쓰기 함수를 사용한 이유는 여러 GPU 스레드가 동시에 동일한 복셀을 업데이트할 수 있기 때문입니다.
또한 float_flip을 사용하여 float 형식의 거리를 uint로 변환했습니다. 이는 InterlockedMin이 uint 형식의 데이터를 조작해야 하기 때문입니다 (모든 하드웨어가 float의 InterlockedMin을 지원하는 것은 아닙니다).
여기서 float_flip 함수의 구현을 자세히 살펴보겠습니다.
```hlsl
inline uint float_flip(float fl) {
  uint f = asuint(fl);
  return (f << 1) | (f >> 31);
}
```
이 함수는 float 값의 첫 번째 비트, 즉 부호 비트를 마지막으로 이동시킵니다. 이렇게 하면 InterlockedMin 비교 시 절대값이 가장 작은 값을 얻을 수 있어 UDF의 정의에 부합합니다.

![Image IEEE 754](images/ieee_754.png)

float 형식의 정의를 보면, 부호 비트를 마지막 비트로 이동시키면 uint와 동일하게 비교할 수 있음을 알 수 있습니다.

모든 삼각형 처리가 완료된 후, float_unflip 함수를 사용하여 부호 비트를 원래 위치로 이동시킵니다.

```hlsl
const int3 uvw = int3(id.x, id.y, id.z);
const uint distance = _distance_texture_rw[uvw];
_distance_texture_rw[uvw] = float_unflip(distance);
```

이로써 Distance Texture에서 삼각형이 커버한 복셀은 Mesh 표면까지의 가장 가까운 거리(부호 없는 값)를 기록하게 됩니다. 하지만 삼각형이 커버하지 않은 영역은 여전히 초기 값으로 남아 있습니다. 다음 단계에서는 이러한 영역을 처리할 것입니다.

### 세 번째 단계: 점프 플러딩

점프 플러딩(Jump Flooding)은 거리 변환(Distance Transform)과 보로노이 다이어그램(Voronoi Diagram)을 계산하는 데 사용되는 효율적인 알고리즘으로, 이미지 처리 및 계산 기하학 분야에서 자주 사용됩니다. 전통적인 픽셀 단위 전파 방법과 달리, 점프 플러딩 알고리즘은 지수적으로 증가하는 스텝 크기로 "점프"하여 계산 속도를 크게 향상시킵니다.

#### 작동 원리

점프 플러딩 알고리즘의 핵심 아이디어는 일련의 감소하는 "점프" 단계를 통해 거리 정보를 전파하는 것입니다. 구체적으로, 알고리즘은 초기 시드 포인트에서 시작하여 큰 스텝 크기로 여러 거리 값을 동시에 업데이트한 후, 점차 스텝 크기를 줄여 더 세밀하게 업데이트합니다. 각 점프 과정에서 알고리즘은 현재 픽셀의 이웃을 검사하고 거리 값을 업데이트하여 최적의 솔루션이 전파되도록 합니다.

먼저 플러딩 알고리즘은 두 개의 버퍼를 교대로 사용해야 합니다. 여기서 Usage를 TRANSFER_SRC로 설정하는 이유는 이후에 GPU에서 CPU로 데이터를 전송한 후 파일로 저장할 수 있도록 하기 위함입니다.
```rust
let jump_buffer_size = num_of_voxels as u64 * std::mem::size_of::<u32>() as u64;
hala_gfx::HalaBuffer::new(
  Rc::clone(&self.resources.context.borrow().logical_device),
  jump_buffer_size,
  hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
  hala_gfx::HalaMemoryLocation::GpuOnly,
  "jump_buffer.buffer",
)?

self.udf_baker_resources.jump_buffer_bis = Some(
  hala_gfx::HalaBuffer::new(
    Rc::clone(&self.resources.context.borrow().logical_device),
    jump_buffer_size,
    hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
    hala_gfx::HalaMemoryLocation::GpuOnly,
    "jump_buffer_bis.buffer",
  )?
);
```

두 개의 버퍼를 번갈아 사용하기 때문에, 미리 두 개의 DescriptorSet을 생성하여 각각 다른 순서로 버퍼를 바인딩하여 후속 사용을 용이하게 합니다.
```rust
// 홀수 단계 점프 시, jump_buffer에서 데이터를 읽어 jump_buffer_bis에 씁니다.
jump_flooding_odd_descriptor_set.update_storage_buffers(
  0,
  0,
  &[jump_buffer],
);
jump_flooding_odd_descriptor_set.update_storage_images(
  0,
  1,
  &[distance_texture],
);
jump_flooding_odd_descriptor_set.update_storage_buffers(
  0,
  2,
  &[jump_buffer_bis],
);

// 짝수 단계 점프 시, jump_buffer_bis에서 데이터를 읽어 jump_buffer에 씁니다.
jump_flooding_even_descriptor_set.update_storage_buffers(
  0,
  0,
  &[jump_buffer_bis],
);
jump_flooding_even_descriptor_set.update_storage_images(
  0,
  1,
  &[distance_texture],
);
jump_flooding_even_descriptor_set.update_storage_buffers(
  0,
  2,
  &[jump_buffer],
);
```

다음으로 점프 플러딩의 초기화를 진행하며, 초기 시드는 자신이 최적의 해라고 간주합니다.
```hlsl
  const float distance = _distance_texture[int3(id.x, id.y, id.z)];
  const uint voxel_index = id3(id.x, id.y, id.z);
  _jump_buffer_rw[voxel_index] = voxel_index;
```

최대 해상도에 대해 log2를 구해 총 몇 번의 점프가 필요한지 계산합니다. 각 단계의 시작 오프셋은 이전 단계의 절반으로 줄어듭니다.
```rust
let num_of_steps = self.settings.max_resolution.ilog2();
for i in 1..=num_of_steps {
  let offset = (1 << (num_of_steps - i)) as u32;
  // 각 단계마다 한 버퍼에서 다른 버퍼로 데이터를 전파합니다.
  ...
}
```

현재 복셀에서 주변 26개 방향으로 점프 샘플링을 수행하고, Mesh 표면까지의 최단 거리를 기록하여 점프 버퍼를 업데이트합니다.
```hlsl
void main(uint3 id) {
  float best_distance = _initial_distance;
  int best_index = 0xFFFFFFFF;

  [unroll(3)]
  for (int z = -1; z <= 1; ++z)
    [unroll(3)]
    for (int y = -1; y <= 1; ++y)
      [unroll(3)]
      for (int x = -1; x <= 1; ++x)
        jump_sample(id, int3(x, y, z) * g_push_constants.offset, best_distance, best_index);

  if (best_index != 0xFFFFFFFF) {
    _jump_buffer_rw[id3(id.x, id.y, id.z)] = best_index;
  }
}
```
*여기서 x == 0 && y == 0 && z == 0인지 여부를 판단하지 않는 이유는, 현재 복셀이 이미 최단 거리라면 후속 업데이트에 영향을 미치지 않기 때문입니다.*

구체적인 점프 샘플링 코드는 다음과 같습니다:
```hlsl
void jump_sample(int3 center_coord, int3 offset, inout float best_distance, inout int best_index) {
  // 현재 좌표에 오프셋을 더해 샘플 좌표를 얻습니다.
  int3 sample_coord = center_coord + offset;
  // 샘플 좌표가 전체 복셀 범위를 초과하면 아무 작업도 하지 않습니다.
  if (
    sample_coord.x < 0 || sample_coord.y < 0 || sample_coord.z < 0 ||
    sample_coord.x >= _dimensions.x || sample_coord.y >= _dimensions.y || sample_coord.z >= _dimensions.z
  ) {
    return;
  }
  // 샘플 좌표에서 시드 인덱스를 가져옵니다.
  uint voxel_sample_index = _jump_buffer[id3(sample_coord)];
  // 인덱스를 x, y, z 좌표 형태로 변환합니다.
  int3 voxel_sample_coord = unpack_id3(voxel_sample_index);
  // 이 좌표에서 Mesh 표면까지의 최단 거리를 가져옵니다.
  float voxel_sample_distance = _distance_texture[voxel_sample_coord];
  // 총 거리는 현재 좌표에서 샘플 좌표까지의 거리와 샘플 좌표에서 Mesh 표면까지의 거리를 더한 값입니다.
  // 주: 여기서 max_dimension으로 나누는 이유는 UVW 공간에서 계산하기 위함입니다. Distance Texture에는 UVW 공간에서의 거리가 저장되어 있습니다.
  float distance = length(float3(center_coord) / _max_dimension - float3(voxel_sample_coord) / _max_dimension) + voxel_sample_distance;
  // 위 계산에서 나온 점프 거리가 이전보다 작으면 최적의 해를 업데이트합니다.
  if (distance < best_distance) {
    best_distance = distance;
    best_index = voxel_sample_index;
  }
}
```

이 알고리즘을 num_of_steps 번 반복하면, 각 복셀 그리드가 최적의 해를 전파하게 됩니다. 여기서 1차원 공간을 예로 들어 최대 해상도가 8이라고 가정하면, log2(8)=3으로 세 번의 점프가 필요하며, 각 점프의 거리는 각각 4, 2, 1입니다.

    첫 번째 단계:
    복셀 0은 0->4가 최적의 해인지 계산
    복셀 1은 1->5가 최적의 해인지 계산
    복셀 2는 2->6이 최적의 해인지 계산
    복셀 3은 3->7이 최적의 해인지 계산
    복셀 4는 4->0이 최적의 해인지 계산
    복셀 5는 5->1이 최적의 해인지 계산
    복셀 6은 6->2가 최적의 해인지 계산
    복셀 7은 7->3이 최적의 해인지 계산
    두 번째 단계:
    복셀 0은 0->2가 최적의 해인지 계산
    복셀 1은 1->3이 최적의 해인지 계산
    복셀 2는 2->4, 2->0이 최적의 해인지 계산
    복셀 3은 3->5, 3->1이 최적의 해인지 계산
    복셀 4는 4->6, 4->2가 최적의 해인지 계산
    복셀 5는 5->7, 5->3이 최적의 해인지 계산
    복셀 6은 6->4가 최적의 해인지 계산
    복셀 7은 7->5가 최적의 해인지 계산
    세 번째 단계:
    복셀 0은 0->1이 최적의 해인지 계산
    복셀 1은 1->2, 1->0이 최적의 해인지 계산
    복셀 2는 2->3, 2->1이 최적의 해인지 계산
    복셀 3은 3->4, 3->2가 최적의 해인지 계산
    복셀 4는 4->5, 4->3이 최적의 해인지 계산
    복셀 5는 5->6, 5->4가 최적의 해인지 계산
    복셀 6은 6->7, 6->5가 최적의 해인지 계산
    복셀 7은 7->6이 최적의 해인지 계산

여기서 4가 삼각형으로 덮이지 않은 복셀이라고 가정하면, 전체 계산 과정에서 4->0, 4->2, 4->3, 4->5, 4->6을 계산하게 됩니다. 그렇다면 1이 삼각형으로 덮인 복셀이라고 가정하면, 4는 계산되지 않을까요?
첫 번째 단계에서 5->1이 최적의 해인지 계산했기 때문에, 이 시점에서 5의 인덱스는 이미 1로 업데이트되었습니다. 따라서 세 번째 단계에서 4->5를 계산할 때 실제로는 4->1이 최적의 해인지 계산하게 됩니다.

이상의 단계를 완료한 후, 최종적으로 Distance Texture를 업데이트해야 합니다.
```hlsl
// 현재 복셀 좌표.
const uint voxel_index = id3(id.x, id.y, id.z);

// 점프 버퍼를 통해 최적의 복셀 인덱스를 가져옵니다.
const uint cloest_voxel_index = _jump_buffer[voxel_index];
// 인덱스를 좌표로 변환합니다.
const int3 cloest_voxel_coord = unpack_id3(cloest_voxel_index);
// 이 최적의 복셀 좌표에 저장된 Mesh까지의 최단 거리를 가져옵니다.
const float cloest_voxel_distance = _distance_texture_rw[cloest_voxel_coord];

// 현재 복셀에서 최적의 복셀까지의 거리(UVW 공간, 이유는 앞서 설명한 바와 같습니다).
const float distance_to_cloest_voxel = length(float3(id) / _max_dimension - float3(cloest_voxel_coord) / _max_dimension);

// 최종 거리는 현재 복셀에서 최적의 복셀까지의 거리와 최적의 복셀에서 Mesh까지의 거리, 그리고 베이킹 설정에서 지정한 오프셋을 더한 값입니다.
_distance_texture_rw[int3(id.x, id.y, id.z)] = cloest_voxel_distance + distance_to_cloest_voxel + g_push_constants.offset;
```
*주의: 점프 플러딩(Jump Flooding) 알고리즘은 빠른 근사 방법으로, 모든 복셀이 최단 거리로 업데이트된다는 보장은 없습니다.*

이로써 Distance Texture에 계산된 UDF 데이터가 저장되었습니다. 이제 시각화할 수 있습니다.

![Image UDF](images/udf.png)

이미지에서 볼 수 있듯이, Mesh 표면에 가까운 곳일수록 색이 짙고(값이 작아 거리가 가까움), 멀리 떨어진 곳일수록 밝습니다(값이 커서 거리가 멀음).

또한 등고선을 통해 Mesh를 재구성할 수도 있습니다.

![Image UDF Mesh](images/udf_mesh.png)


## SDF 베이킹

UDF와 비교하여 SDF의 베이킹은 훨씬 더 복잡합니다. 여기서의 구현은 Unity의 [Visual Effect Graph](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@14.0/manual/sdf-in-vfx-graph.html) 방안을 참조하였습니다.

### 첫 번째 단계: 초기화

베이킹 설정 항목 추가:
```rust
pub sign_passes_count: i32, // 기호 패스(기호가 양수인지 음수인지 찾는)의 반복 횟수.
pub in_out_threshold: f32,  // 메쉬 내부인지 외부인지 판단하는 임계값.
```

다음으로 전체 베이킹 과정에서 필요한 몇 가지 매개변수를 저장하기 위한 전역 UBO를 준비합니다. 구체적인 내용은 아래 코드의 주석을 참조하십시오.
```rust
pub struct GlobalUniform {
  pub dimensions: [u32; 3],   // 베이킹할 메쉬의 BoundingBox 정보와 베이킹 보셀의 최대 해상도를 기반으로 세 차원의 크기를 계산합니다.
  pub upper_bound_count: u32, // 각 보셀에 포함된 삼각형의 버퍼 상한을 저장합니다.
  pub num_of_triangles: u32,  // 베이킹할 메쉬의 총 삼각형 수.
  pub max_size: f32,          // 전체 베이킹 영역의 가장 긴 변의 길이를 기반으로 합니다.
  pub max_dimension: u32,     // 전체 보셀 공간의 가장 긴 변의 보셀 수.
  pub center: [f32; 3],       // 베이킹 영역 BoundingBox의 중심 좌표.
  pub extents: [f32; 3],      // 베이킹 영역 BoundingBox의 반길이.
}
```
다른 값의 계산은 UDF와 동일하며, upper_bound_count의 경우 각 보셀이 얼마나 많은 삼각형을 포함할지 확실하지 않기 때문에 최대값을 추정할 수밖에 없습니다.
```rust
// 우선 절반의 보셀에 삼각형이 있다고 가정합니다.
let num_of_voxels_has_triangles = dimensions[0] as f64 * dimensions[1] as f64 * dimensions[2] as f64 / 2.0f64;
// 하나의 삼각형이 인접한 8개의 보셀에 공유된다고 가정합니다. 각 보셀이 총 삼각형 수의 제곱근 수의 삼각형을 가질 것이라고 가정합니다.
// 여기서 두 가정 중 최대값을 취합니다.
let avg_triangles_per_voxel = (num_of_triangles as f64 / num_of_voxels_has_triangles * 8.0f64).max((num_of_triangles as f64).sqrt());
// 총 저장해야 할 삼각형 수.
let upper_bound_count64 = (num_of_voxels_has_triangles * avg_triangles_per_voxel) as u64;
// 최대값을 1536 * 2^18로 제한합니다.
let upper_bound_count = (1536 * (1 << 18)).min(upper_bound_count64) as u32;
// 최소값을 1024로 제한합니다.
let upper_bound_count = upper_bound_count.max(1024);
```
*주의: 이는 보수적인 추정일 뿐이며, 실제로 필요한 수량은 이 값보다 훨씬 적을 수 있습니다. 보수적인 추정은 더 많은 경계 상황을 커버하기 위함입니다.*

SDF의 베이킹 과정에서는 많은 임시 버퍼가 필요합니다. 지면을 절약하기 위해 여기서는 소개하지 않지만, 자세한 내용은 소스 코드 파일을 참조하십시오.

### 두 번째 단계: 기하학적 구조 구축

우선, UDF와 마찬가지로 메쉬의 Vertex Buffer와 Index Buffer에서 삼각형 정보를 읽어와서 정규화된 UVW 공간으로 변환한 후 Triangle UVW Buffer에 저장합니다.
```hlsl
Triangle tri_uvw;
tri_uvw.a = (get_vertex_pos(id.x, 0) - _center + _extents) / _max_size;
tri_uvw.b = (get_vertex_pos(id.x, 1) - _center + _extents) / _max_size;
tri_uvw.c = (get_vertex_pos(id.x, 2) - _center + _extents) / _max_size;

_triangles_uvw_rw[id.x] = tri_uvw;
```

다음으로, 각 삼각형의 "방향"을 계산합니다. 여기서 "방향"은 삼각형이 대체로 어떤 축을 향하고 있는지를 나타내며, XY, ZX, YZ 중 어떤 평면에 더 가까운지를 의미합니다. 결과는 Coord Flip Buffer에 저장됩니다.
```hlsl
const float3 a = get_vertex_pos(id.x, 0);
const float3 b = get_vertex_pos(id.x, 1);
const float3 c = get_vertex_pos(id.x, 2);
const float3 edge0 = b - a;
const float3 edge1 = c - b;
const float3 n = abs(cross(edge0, edge1));
if (n.x > max(n.y, n.z) + 1e-6f) {  // 비교를 더 안정적으로 만들기 위해 epsilon을 더합니다.
  // 삼각형이 거의 YZ 평면과 평행합니다.
  _coord_flip_buffer_rw[id.x] = 2;
} else if (n.y > max(n.x, n.z) + 1e-6f) {
  // 삼각형이 거의 ZX 평면과 평행합니다.
  _coord_flip_buffer_rw[id.x] = 1;
} else {
  // 삼각형이 거의 XY 평면과 평행합니다.
  _coord_flip_buffer_rw[id.x] = 0;
}
```
여기서 ZX 평면이 아니라 XZ 평면인 이유는 이후 세 방향에서 각각 계산이 필요하기 때문입니다. ZX 평면은 Y축 방향에서 계산할 때, 로컬 X축이 실제로는 Z이고, 로컬 Y축이 실제로는 X임을 나타냅니다.

각 삼각형에 방향을 할당한 후, 다음 단계는 각 방향에서 삼각형을 보수적으로 래스터화하는 것입니다.
그 전에 세 방향의 직교 및 투영 행렬을 계산합니다.
```rust
// 시점 위치, 회전 축, 너비, 높이, 근평면 거리 및 원평면 거리를 기반으로 View 행렬과 Proj 행렬을 구성합니다.
let calculate_world_to_clip_matrix = |eye, rot, width: f32, height: f32, near: f32, far: f32| {
  let proj = glam::Mat4::orthographic_rh(-width / 2.0, width / 2.0, -height / 2.0, height / 2.0, near, far);
  let view = glam::Mat4::from_scale_rotation_translation(glam::Vec3::ONE, rot, eye).inverse();
  proj * view
};
```

Z 방향의 XY 평면은 아래 그림과 같습니다. 로컬 X축은 세계의 X축이고, 로컬 Y축은 세계의 Y축입니다.

![Image XY Plane](images/xy_plane.png)

```rust
let xy_plane_mtx = {
  // 시점이 Z 방향으로 1만큼 위치한 곳에서 아래를 봅니다.
  let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(0.0, 0.0, bounds.extents[2] + 1.0);
  // View 공간은 기본적으로 아래를 보므로 회전이 필요 없습니다.
  let rot = glam::Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0);
  // 근평면은 1에 위치하므로 시점 위치가 1만큼 더해집니다. 근평면에 공간을 남겨둡니다.
  let near = 1.0f32;
  // 원평면은 근평면에서 시작하여 전체 BoundingBox의 Z 방향 길이를 연장합니다.
  let far = near + bounds.extents[2] * 2.0;
  calculate_world_to_clip_matrix(pos, rot, bounds.extents[0] * 2.0, bounds.extents[1] * 2.0, near, far)
};
```

Y 방향의 ZX 평면은 아래 그림과 같습니다. 로컬 X축은 세계의 Z축이고, 로컬 Y축은 세계의 X축입니다.

![Image ZX Plane](images/zx_plane.png)

```rust
let zx_plane_mtx = {
  // 시점이 Y 방향으로 1만큼 위치한 곳에서 외부를 봅니다(Y축의 양의 방향에서 음의 방향을 봅니다).
  let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(0.0, bounds.extents[1] + 1.0, 0.0);
  // 먼저 Y축을 따라 -90도 회전한 다음 X축을 따라 -90도 회전합니다. 로컬 X축을 세계의 Z축과 정렬하고, 로컬 Y축을 세계의 X축과 정렬합니다.
  let rot = glam::Quat::from_euler(glam::EulerRot::YXZ, -std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2, 0.0);
  let near = 1.0f32;
  let far = near + bounds.extents[1] * 2.0;
  calculate_world_to_clip_matrix(pos, rot, bounds.extents[2] * 2.0, bounds.extents[0] * 2.0, near, far)
};
```

X 방향의 YZ 평면은 아래 그림과 같습니다. 로컬 X축은 세계의 Y축이고, 로컬 Y축은 세계의 Z축입니다.

![Image YZ Plane](images/yz_plane.png)

```rust
let yz_plane_mtx = {
  // 시점이 X 방향으로 1만큼 위치한 곳에서 왼쪽을 봅니다.
  let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(bounds.extents[0] + 1.0, 0.0, 0.0);
  // 먼저 X축을 따라 90도 회전한 다음 Y축을 따라 90도 회전합니다. 로컬 X축을 세계의 Y축과 정렬하고, 로컬 Y축을 세계의 Z축과 정렬합니다.
  let rot = glam::Quat::from_euler(glam::EulerRot::XYZ, std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2, 0.0);
  let near = 1.0f32;
  let far = near + bounds.extents[0] * 2.0;
  calculate_world_to_clip_matrix(pos, rot, bounds.extents[1] * 2.0, bounds.extents[2] * 2.0, near, far)
};
```

다음으로는 위의 세 방향에서 해당 방향의 삼각형을 보수적으로 래스터화합니다.
우선 삼각형의 커버 범위의 2D AABB를 계산하여 float4에 저장합니다. xy는 min을 저장하고, zw는 max를 저장합니다.
```hlsl
// 삼각형의 세 꼭지점을 가져와서 clip 공간으로 변환합니다.
[unroll(3)]
for (i = 0; i < 3; i++) {
  vertex_in_clip[i] = mul(_world_to_clip[current_axis], float4(get_vertex_pos(id.x, i), 1.0));
}

// AABB의 크기를 계산합니다.
float4 aabb = float4(1.0, 1.0, -1.0, -1.0);
aabb.xy = min(aabb.xy, min(vertex_in_clip[0].xy, min(vertex_in_clip[1].xy, vertex_in_clip[2].xy)));
aabb.zw = max(aabb.xy, max(vertex_in_clip[0].xy, max(vertex_in_clip[1].xy, vertex_in_clip[2].xy)));
float2 conservative_pixel_size;
// 현재 래스터화 방향에 따라 Conservative Offset 매개변수를 기반으로 실제 필요한 Offset 픽셀 크기를 계산합니다.
if (current_axis == 0) {
  conservative_pixel_size = float2(_conservative_offset / _dimensions.x, _conservative_offset / _dimensions.y);
} else if (current_axis == 1) {
  conservative_pixel_size = float2(_conservative_offset / _dimensions.z, _conservative_offset / _dimensions.x);
} else {
  conservative_pixel_size = float2(_conservative_offset / _dimensions.y, _conservative_offset / _dimensions.z);
}

// AABB 크기를 확대합니다.
_aabb_buffer_rw[id.x] = aabb + float4(-conservative_pixel_size.x, -conservative_pixel_size.y, conservative_pixel_size.x, conservative_pixel_size.y);
```

그런 다음 삼각형을 래스터화하고 설정된 Offset을 확대합니다. 여기서 보수적 래스터화 확대는 float 계산 시의 오차로 인해 "틈새"가 누락되지 않도록 하기 위함입니다.
```hlsl
// 삼각형이 있는 평면을 float4에 저장합니다. xyz는 평면 법선 방향이고, w는 평면이 원점에서 떨어진 거리입니다.
const float3 normal = normalize(cross(vertex_in_clip[1].xyz - vertex_in_clip[0].xyz, vertex_in_clip[2].xyz - vertex_in_clip[0].xyz));
const float4 triangle_plane = float4(normal, -dot(vertex_in_clip[0].xyz, normal));

// 법선 방향이 Z 양의 방향(1)인지 음의 방향(-1)인지 계산합니다.
const float direction = sign(dot(normal, float3(0, 0, 1)));
float3 edge_plane[3];
[unroll(3)]
for (i = 0; i < 3; i++) {
  // 2D 가장자리 평면을 계산합니다. W는 동차 좌표입니다.
  edge_plane[i] = cross(vertex_in_clip[i].xyw, vertex_in_clip[(i + 2) % 3].xyw);
  // 이전에 결정된 방향과 오프셋 픽셀 값을 기반으로 가장자리 평면을 "외부"로 밀어냅니다.
  // 여기서 이해하기 어려운 부분은 나중에 그림을 참조하십시오.
  edge_plane[i].z -= direction * dot(conservative_pixel_size, abs(edge_plane[i].xy));
}

float4 conservative_vertex[3];
bool is_degenerate = false;
[unroll(3)]
for (i = 0; i < 3; i++) {
  _vertices_buffer_rw[3 * id.x + i] = float4(0, 0, 0, 1);

  // 세 가장자리 평면을 기반으로 교차하여 새로운 꼭지점 위치를 계산합니다.
  conservative_vertex[i].xyw = cross(edge_plane[i], edge_plane[(i + 1) % 3]);

  // W 값을 기반으로 삼각형이 퇴화되었는지 판단합니다.
  if (abs(conservative_vertex[i].w) < CONSERVATIVE_RASTER_EPS) {
    is_degenerate |= true;
  } else {
    is_degenerate |= false;
    conservative_vertex[i] /= conservative_vertex[i].w; // 이후, w는 1이 됩니다.
  }
}
if (is_degenerate)
  return;

// 삼각형 위의 점을 통해 평면 공식을 만족하여 세 꼭지점의 Z 값을 계산합니다.
// 평면 공식: ax + by + cz + d = 0.
// Z 계산: z = -(ax + by + d) / c.
// 마지막으로 새로 얻은 세 꼭지점을 Vertices Buffer에 씁니다.
[unroll(3)]
for (i = 0; i < 3; i++) {
  conservative_vertex[i].z = -(triangle_plane.x * conservative_vertex[i].x + triangle_plane.y * conservative_vertex[i].y + triangle_plane.w) / triangle_plane.z;
  _vertices_buffer_rw[3 * id.x + i] = conservative_vertex[i];
}
```
컴퓨터 그래픽스에서 평면은 네 개의 차원을 가진 벡터로 표현될 수 있습니다: float4(plane) = (a, b, c, d), 여기서 평면의 방정식은 ax + by + cz + d = 0입니다. "가장자리 평면" 개념은 2D 투영상의 기하학적 구조(예: 삼각형)를 처리할 때, 삼각형의 경계를 나타내기 위해 공간을 분할하는 평면을 사용하는 아이디어에 기반합니다.

    edge_plane[i] = cross(vertex_in_clip[i].xyw, vertex_in_clip[(i + 2) % 3].xyw);

이 코드에서 가장자리 평면을 구체적으로 구성하는 방법은 두 꼭지점의 동차 좌표의 교차 곱을 통해 얻습니다. 여기서 vertex_in_clip은 꼭지점의 동차 좌표입니다. vertex_in_clip[i].xyw는 꼭지점의 x, y, w 성분을 추출하여 3차원 벡터로 간주합니다. cross 함수는 두 3차원 벡터의 교차 곱을 계산하여 이 두 벡터가 있는 평면에 수직인 벡터를 생성합니다. 이렇게 생성된 벡터 edge_plane[i]는 vertex_in_clip[i]에서 vertex_in_clip[(i + 2) % 3]까지의 경계 평면을 나타냅니다(동차 좌표 하에서의 2D 평면 표현).

여기서 보수적 래스터화된 삼각형을 모델 공간으로 복원한 모습입니다. 빨간색 선 프레임은 확대된 삼각형을, 흰색 선 프레임은 원래 삼각형을 나타냅니다. 각 삼각형이 해당 평면을 따라 한 바퀴 확대된 것을 볼 수 있습니다.

![Image Conservative Offset](images/conservative_offset.png)
### 3단계: 삼각형 덮개 복셀 카운트 통계

다음으로 Compute Shader를 잠시 떠나 Vertex Shader와 Fragment Shader를 사용하여 세 방향에서 삼각형 덮개 횟수를 통계내야 합니다.
먼저 Vertex Shader를 살펴보겠습니다.
```hlsl
struct VertexInput {
  // Draw(num_of_triangles * 3)을 통해 Vertex Id를 전달합니다.
  uint vertex_id: SV_VertexID;
};

ToFragment main(VertexInput input) {
  ToFragment output = (ToFragment)0;

  // Vertex Id를 사용하여 이전 단계의 래스터화 결과에서 Clip 공간의 정점 데이터를 직접 읽습니다.
  const float4 pos = _vertices_buffer[input.vertex_id];
  // Vertex Id를 3으로 나누어 삼각형 ID를 얻습니다.
  output.triangle_id = input.vertex_id / 3;
  // 현재 삼각형이 현재 그리는 방향과 다르면 (-1, -1, -1, -1)을 전달하여 Fragment Shader가 건너뛰도록 합니다.
  if (_coord_flip_buffer[output.triangle_id] != g_push_constants.current_axis) {
    output.position = float4(-1, -1, -1, -1);
  } else {
    output.position = pos;
  }

  return output;
}
```

전체적으로 Fragment Shader의 흐름을 살펴보겠습니다.
```hlsl
struct ToFragment {
  float4 position: SV_Position;
  uint triangle_id: TEXCOORD0;
};

FragmentOutput main(ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

  // Vertex Shader에서 전달된 position과 삼각형 ID를 사용하여 현재 처리 중인 픽셀의 복셀 좌표 voxel_coord를 계산합니다.
  // 동시에 깊이 방향으로 안쪽(backward)과 바깥쪽(forward)으로 확장할 수 있는지 판단합니다.
  int3 depth_step, voxel_coord;
  bool can_step_backward, can_step_forward;
  get_voxel_coordinates(input.position, input.triangle_id, voxel_coord, depth_step, can_step_backward, can_step_forward);

  // 복셀 중심 좌표를 정규화된 UVW 공간으로 변환하고, 이를 Voxels Buffer에 저장합니다.
  float3 voxel_uvw = (float3(voxel_coord) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension;
  _voxels_buffer_rw[id3(voxel_coord)] = float4(voxel_uvw, 1.0f);
  // 현재 복셀 좌표의 Counter Buffer에서 누적하여 이 복셀이 삼각형에 의해 한 번 덮였음을 표시합니다.
  InterlockedAdd(_counter_buffer_rw[id3(voxel_coord)], 1u);
  // 바깥쪽으로 확장할 수 있다면, 바깥쪽 복셀에 대해 동일한 작업을 수행합니다.
  if (can_step_forward) {
    _voxels_buffer_rw[id3(voxel_coord + depth_step)] = float4(voxel_uvw, 1.0f);
    InterlockedAdd(_counter_buffer_rw[id3(voxel_coord + depth_step)], 1u);
  }
  // 안쪽으로 확장할 수 있다면, 안쪽 복셀에 대해 동일한 작업을 수행합니다.
  if (can_step_backward) {
    _voxels_buffer_rw[id3(voxel_coord - depth_step)] = float4(voxel_uvw, 1.0f);
    InterlockedAdd(_counter_buffer_rw[id3(voxel_coord - depth_step)], 1u);
  }

  // 여기서 RT의 출력은 베이킹 과정에 참여하지 않고, 디버깅 용도로만 사용됩니다.
  output.color = float4(voxel_uvw, 1);
  return output;
}
```
전체적인 흐름은 VS와 FS를 사용하여 삼각형 덮개 영역에서 Counter Buffer를 누적하는 작업입니다.
이제 `get_voxel_coordinates`의 구현을 자세히 살펴보겠습니다.
```hlsl
void get_voxel_coordinates(
  float4 screen_position,
  uint triangle_id,
  out int3 voxel_coord,
  out int3 depth_step,
  out bool can_step_backward,
  out bool can_step_forward
) {
  // 현재 화면 해상도를 가져옵니다. 이는 현재 방향의 복셀 너비와 높이입니다.
  // 예를 들어 현재 복셀 공간이 [2, 3, 4]라면, Z 방향의 XY 평면에서 처리할 때 2 x 3을 반환합니다.
  const float2 screen_params = get_custom_screen_params();
  // Vertex Shader에서 전달된 Position을 UVW 공간으로 변환합니다.
  screen_to_uvw(screen_position, screen_params);
  // 삼각형 ID를 사용하여 이전에 계산된 삼각형 덮개 영역의 AABB를 가져와, AABB 범위 내에 있지 않으면 현재 Fragment Shader의 후속 실행을 중단합니다.
  cull_with_aabb(screen_position, triangle_id);
  // 복셀 좌표를 계산하고, 앞뒤로 확장할 수 있는지 결정합니다.
  compute_coord_and_depth_step(
    screen_params,
    screen_position,
    voxel_coord,
    depth_step,
    can_step_backward,
    can_step_forward
  );
}
```
이제 `compute_coord_and_depth_step`의 구현을 자세히 살펴보겠습니다.
```hlsl
void compute_coord_and_depth_step(
  float2 screen_params,
  float4 screen_position,
  out int3 voxel_coord,
  out int3 depth_step,
  out bool can_step_backward,
  out bool can_step_forward
) {
  // 여기서는 삼각형이 인접한 앞뒤 복셀에 의해 공유될 수 있다고 보수적으로 가정합니다. 이렇게 하면 후속 표시 문제를 피할 수 있습니다.
  can_step_forward = true;
  can_step_backward = true;

  if (g_push_constants.current_axis == 1) {
    // UVW 공간의 Position을 사용하여 복셀 좌표를 계산합니다.
    voxel_coord = (screen_position.xyz * float3(screen_params, _dimensions[1]));
    voxel_coord.xyz = voxel_coord.yzx;

    // 경계인지 확인하고, 경계가 아니면 안쪽과 바깥쪽으로 확장할 수 있습니다.
    depth_step = int3(0, 1, 0);
    if (voxel_coord.y <= 0) {
      can_step_backward = false;
    }
    if (voxel_coord.y >= _dimensions[1] - 1) {
      can_step_forward = false;
    }
  } else if (g_push_constants.current_axis == 2) {
    // 위와 동일하지만 축 방향이 다릅니다.
  } else {
    // 위와 동일하지만 축 방향이 다릅니다.
  }
}
```
깊이 쓰기와 깊이 테스트가 비활성화되었기 때문에, 이 시점에서 세 방향에서 삼각형 덮개 복셀은 모두 InterlockedAdd를 통해 Counter Buffer에 카운트되었습니다.
동시에 덮개된 복셀의 UVW 좌표도 Voxels Buffer에 저장되었습니다.

다음은 Prefix Sum 알고리즘을 사용하여 Counter Buffer를 누적하고, 최종 결과를 Accum Counter Buffer에 저장하는 것입니다. 기본 아이디어는 배열의 각 위치 이전의 모든 요소의 합을 미리 계산하여 후속 조회 작업을 상수 시간 내에 완료할 수 있도록 하는 것입니다.
Prefix Sum 알고리즘은 베이킹 자체와 직접적인 관련이 없으므로 관련 알고리즘의 소개 링크만 제공합니다:
* [위키백과](https://en.wikipedia.org/wiki/Prefix_sum),
* [GPU Gems 3 - Chapter 39. Parallel Prefix Sum (Scan) with CUDA](https://developer.nvidia.com/gpugems/gpugems3/part-vi-gpu-computing/chapter-39-parallel-prefix-sum-scan-cuda)

이 시점에서 Accum Counter Buffer에는 현재 복셀 이전의 모든 복셀이 포함된 삼각형 수가 저장되어 있습니다.
예를 들어, 복셀 0, 1, 2, 3, 4가 각각 4, 2, 5, 0, 3개의 삼각형에 의해 덮여 있다고 가정합니다. 그러면 이 시점에서 카운트 Buffer의 값은 다음과 같습니다:

    0 (현재 복셀 이전에 다른 복셀이 없음)
    4 (현재 복셀 이전에 0번 복셀이 있으며, 0번 복셀은 4개의 삼각형이 있음)
    6 (현재 복셀 이전에 0번과 1번 복셀이 있으며, 0번 복셀은 4개의 삼각형, 1번 복셀은 2개의 삼각형, 총 6개)
    11 (위와 동일한 알고리즘)
    11 (위와 동일한 알고리즘)

다음은 이러한 삼각형을 Triangle Id Buffer에 저장하고, Accum Counter Buffer를 통해 각 복셀에 포함된 삼각형 목록을 탐색하는 것입니다. 여기서도 Vertex Shader와 Fragment Shader를 사용합니다.

Vertex Shader는 이전과 동일하므로 반복하지 않겠습니다. Fragment의 다른 점만 살펴보겠습니다.
```hlsl
// 여기서 계산된 복셀 좌표를 사용하여 Counter Buffer를 1 증가시키고, 원래 값을 반환하여 Triangle Ids Buffer의 인덱스로 사용합니다.
uint index = 0u;
InterlockedAdd(_counter_buffer_rw[id3(voxel_coord)], 1u, index);
// 여기서 경계를 방지하기 위해 처음 계산된 복셀당 삼각형 Buffer의 상한을 사용합니다.
if (index < _upper_bound_count)
_triangle_ids_buffer_rw[index] = input.triangle_id;
// 동일하게 바깥쪽과 안쪽 복셀에 대해 확장합니다.
if (can_step_forward) {
  InterlockedAdd(_counter_buffer_rw[id3(voxel_coord + depth_step)], 1u, index);
  if (index < _upper_bound_count)
  _triangle_ids_buffer_rw[index] = input.triangle_id;
}
if (can_step_backward) {
  InterlockedAdd(_counter_buffer_rw[id3(voxel_coord - depth_step)], 1u, index);
  if (index < _upper_bound_count)
  _triangle_ids_buffer_rw[index] = input.triangle_id;
}
```
여기서 이해하기 쉽습니다. 복셀 i 이전의 복셀에 몇 개의 삼각형이 덮여 있는지는 이미 Counter Buffer에 저장되어 있으므로, `_counter_buffer_rw[id3(voxel_coord)]`에서 가져온 값은 현재 복셀 i에 삼각형 인덱스를 쓸 수 있는 시작 위치입니다.

### 4단계: Ray Map 계산

위의 모든 계산을 완료한 후, 다음 코드를 통해 지정된 보셀의 삼각형 목록을 순회할 수 있습니다.
```hlsl
uint start_triangle_id = 0;
[branch]
if (id3(id) > 0) {
  start_triangle_id = _accum_counter_buffer[id3(id) - 1];
}
uint end_triangle_id = _accum_counter_buffer[id3(id)];

for (uint i = start_triangle_id; i < end_triangle_id && (i < _upper_bound_count - 1); i++) {
  const uint triangle_index = _triangles_in_voxels[i];
  const Triangle tri = _triangles_uvw[triangle_index];
  ...
}
```

Ray Map을 계산하기 전에 몇 가지 보조 함수를 소개합니다.
```hlsl
// 선분과 삼각형의 교점을 계산합니다. 교차하지 않으면 0을 반환하고, 삼각형 가장자리와 교차하면 0.5 또는 -0.5를 반환하며, 삼각형 내부와 교차하면 1.0 또는 -1.0을 반환합니다.
// 기호는 삼각형의 앞면 또는 뒷면과 교차하는지를 나타냅니다. t는 교점 매개변수를 반환합니다.
float intersect_segment_to_triangle_with_face_check(float3 segment_start, float3 segment_end, Triangle tri, out float t_value) {
  /*
   * 삼각형 평면 방정식: n * (P - A) = 0
   * 선분 방정식: P(t) = Q + t(S - Q)
   * n dot ((Q + t(S - Q)) - A) = 0
   * n dot (Q - A + t(S - Q)) = 0
   * n dot (Q - A) + t(n dot (S - Q)) = 0
   * 𝑣 = 𝑄 - 𝐴, 𝑑 = 𝑆 − 𝑄
   * t = - (n dot 𝑣) / (n dot d)
   *
   * 여기서:
   * n - 삼각형 평면의 법선 벡터
   * P - 삼각형 평면상의 임의의 점
   * A - 삼각형의 한 꼭짓점
   * Q, S - 선분의 두 끝점
   * t - 교점 매개변수, 선분과 삼각형의 교점을 설명하는 데 사용됨
   * 𝑣 - 벡터 Q - A
   * 𝑑 - 벡터 S - Q
   */

  // 삼각형의 두 변을 계산합니다.
  const float3 edge1 = tri.b - tri.a;
  const float3 edge2 = tri.c - tri.a;
  // 여기서는 실제로 -d = Q - S를 계산합니다.
  const float3 end_to_start = segment_start - segment_end;

  // 교차 곱셈을 통해 삼각형 평면의 법선 벡터를 계산합니다.
  const float3 normal = cross(edge1, edge2);
  // 선분 방향과 삼각형 법선 벡터의 점곱을 계산합니다.
  const float dot_product = dot(end_to_start, normal);
  // 이 점곱 결과의 기호는 선분이 삼각형의 앞면 또는 뒷면과 교차하는지를 나타냅니다.
  const float side = sign(dot_product);
  // 역수를 취합니다.
  const float inverse_dot_product = 1.0f / dot_product;

  // v = Q - A
  const float3 vertex0_to_start = segment_start - tri.a;
  // 공식을 사용하여 교점의 t 값을 계산합니다.
  // t = - (n dot v) / (n dot d)
  //   = (n dot v) / (n dot -d)
  float t = dot(vertex0_to_start, normal) * inverse_dot_product;

  // t 값이 0보다 작거나 1보다 크면 선분과 삼각형 평면이 교차하지 않음을 의미합니다.
  if (t < -INTERSECT_EPS || t > 1 + INTERSECT_EPS) {
    t_value = 1e10f;
    return 0;
  } else {
    // 중심 좌표를 계산하여 교점이 삼각형 내부에 있는지 확인합니다.
    const float3 cross_product = cross(end_to_start, vertex0_to_start);
    const float u = dot(edge2, cross_product) * inverse_dot_product;
    const float v = -dot(edge1, cross_product) * inverse_dot_product;
    float edge_coefficient = 1.0f;

    // 중심 좌표가 지정된 범위에 있지 않으면 교점이 삼각형 외부에 있습니다.
    if (u < -BARY_EPS || u > 1 + BARY_EPS || v < -BARY_EPS || u + v > 1 + BARY_EPS) {
      t_value = 1e10f;
      return 0;
    } else {
      const float w = 1.0f - u - v;
      // 중심 좌표가 삼각형 경계에 있으면 계수를 0.5로 조정합니다.
      if (abs(u) < BARY_EPS || abs(v) < BARY_EPS || abs(w) < BARY_EPS) {
        edge_coefficient = 0.5f;
      }

      // t 값과 교차 결과를 반환합니다.
      t_value = t;
      return side * edge_coefficient;
    }
  }
}

// 지정된 보셀 내에서, 앞뒤 좌우 상하 세 방향으로 삼각형과 교점을 계산합니다.
// 정방향(+x +y +z)과 역방향(-x -y -z)에서 앞면과 교차하는 삼각형 수와 뒷면과 교차하는 삼각형 수의 차이를 반환합니다.
void calculate_triangle_intersection_with_3_rays(
  in Triangle tri,
  in int3 voxel_id,
  out float3 intersect_forward,
  out float3 intersect_backward
) {
  // 초기 카운트는 모두 0입니다.
  intersect_forward = float3(0.0f, 0.0f, 0.0f);
  intersect_backward = float3(0.0f, 0.0f, 0.0f);

  // 교차 매개변수 t.
  float t = 1e10f;
  // 정규화된 UVW 공간에서 선분의 시작점과 끝점.
  float3 p, q;
  // 교차 방향을 누적하는 카운트 변수.
  float intersect = 0;

  // UVW 공간에서 X 방향으로, 보셀의 중심에서 선분의 두 끝점을 생성합니다.
  p = (float3(voxel_id) + float3(0.0f, 0.5f, 0.5f)) / _max_dimension;
  q = (float3(voxel_id) + float3(1.0f, 0.5f, 0.5f)) / _max_dimension;
  // 선분이 왼쪽에서 오른쪽으로 이동할 때, 삼각형이 오른쪽을 향하고 있으면 왼쪽이 내부(-), 오른쪽이 외부(+)입니다.
  // 그러나 이때 선분이 삼각형의 뒷면을 통과하므로 결과를 반대로 취합니다.
  intersect = -intersect_segment_to_triangle_with_face_check(p, q, tri, t);
  if (t < 0.5f) {
    // t가 0.5보다 작으면 교점이 왼쪽에 가까움을 의미하므로 Backward에 기호 카운트를 누적합니다.
    intersect_backward.x += float(intersect);
  } else {
    // 반대로 교점이 오른쪽에 가까움을 의미하므로 Forward에 기호 카운트를 누적합니다.
    intersect_forward.x += float(intersect);
  }

  // Y 방향은 X 방향과 동일하지만 축이 다릅니다.
  ...

  // Z 방향은 X 방향과 동일하지만 축이 다릅니다.
  ...
}
```

위의 두 보조 함수를 사용하여 2x2 보셀 단위로 모든 보셀을 계산하여 인접한 보셀이 삼각형의 앞면이 더 많은지 뒷면이 더 많은지를 확인할 수 있습니다.
8번 반복하여 (0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 0), (1, 0, 1), (0, 1, 1), (1, 1, 1) 위치에서 모든 보셀을 순회합니다.
```hlsl
for (uint i = start_triangle_id; i < end_triangle_id && (i < _upper_bound_count - 1); i++) {
  // 위의 보조 함수를 통해 보셀 [x, y, z]에 포함된 삼각형이 "앞"에서 정면과 교차하는지 뒷면과 교차하는지의 차이와 "뒤"에서 정면과 교차하는지 뒷면과 교차하는지의 차이를 계산합니다.
  calculate_triangle_intersection_with_3_rays(tri, int3(id.xyz), intersect_forward, intersect_backward);

  // "앞"의 경우를 Ray Map의 보셀 [x, y, z]에 누적합니다.
  _ray_map_rw[id.xyz] += float4(intersect_forward, 1.0f);

  // 경계를 넘지 않는 경우 "뒤"의 경우를 Ray Map의 인접 보셀에 누적합니다.
  if (id.x > 0) {
    _ray_map_rw[int3(id.x - 1, id.y, id.z)] += float4(intersect_backward.x, 0.0f, 0.0f, 1.0f);
  }
  if (id.y > 0) {
    _ray_map_rw[int3(id.x, id.y - 1, id.z)] += float4(0.0f, intersect_backward.y, 0.0f, 1.0f);
  }
  if (id.z > 0) {
    _ray_map_rw[int3(id.x, id.y, id.z - 1)] += float4(0.0f, 0.0f, intersect_backward.z, 1.0f);
  }
}
```
*주의: 선분과 삼각형이 교차하지 않으면 `intersect_segment_to_triangle_with_face_check`의 반환 값은 0입니다. 따라서 누적을 수행해도 영향이 없으므로 여기서는 아무런 판단을 하지 않았습니다.*

다음으로 세 방향에서 이러한 값을 더합니다. 여기서는 X 방향의 계산만 나열합니다.
```hlsl
// 정방향에서 역방향으로 누적합니다.
for (int t = _dimensions.x - 2; t >= 0; t--) {
  float count = _ray_map_rw[int3(t + 1, id.y, id.z)].x;
  _ray_map_rw[int3(t, id.y, id.z)] += float4(count, 0, 0, count != 0 ? 1 : 0);
}
```

이로써 일련의 계산을 통해 Ray Map을 통해 임의의 보셀이 오른쪽에서 왼쪽, 위에서 아래, 뒤에서 앞으로 삼각형의 앞면과 뒷면을 얼마나 많이 통과했는지를 알 수 있습니다.
다음 기호 판정을 위한 데이터를 준비했습니다.

아래 그림에서 Ray Map의 시각화를 통해 모델 내부 영역과 외부 영역을 구분할 수 있음을 알 수 있습니다.

![Image Ray Map](images/ray_map.png)

### 다섯 번째 단계: 기호 계산

먼저 Sign Map을 초기화합니다.
```hlsl
const float4 self_ray_map = _ray_map[id.xyz];
// 현재 복셀에 해당하는 Ray Map에는 오른쪽에서 왼쪽, 뒤에서 앞으로, 위에서 아래로의 교차 차이가 저장되어 있습니다.
const float right_side_intersection = self_ray_map.x;
const float back_side_intersection = self_ray_map.y;
const float top_side_intersection = self_ray_map.z;
// 각 방향의 첫 번째 평면 내의 복셀은 오른쪽에서 왼쪽, 뒤에서 앞으로, 위에서 아래로의 교차 차이를 저장합니다.
// 현재 복셀의 값을 빼면 남은 왼쪽, 앞쪽, 아래쪽의 교차 차이가 됩니다.
const float left_side_intersection = _ray_map[int3(0, id.y, id.z)].x - self_ray_map.x;
const float front_side_intersection = _ray_map[int3(id.x, 0, id.z)].y - self_ray_map.y;
const float bottom_side_intersection = _ray_map[int3(id.x, id.y, 0)].z - self_ray_map.z;
// 이들을 모두 더하면 현재 복셀에서 양의 교차가 많은지 음의 교차가 많은지를 대략적으로 나타낼 수 있으며, 이는 내부인지 외부인지를 의미합니다.
_sign_map_rw[id.xyz] =
  right_side_intersection - left_side_intersection +
  back_side_intersection - front_side_intersection +
  top_side_intersection - bottom_side_intersection;
```

이때 각 복셀 주변의 축 방향 교차 영향만 고려했으므로, 정확도가 충분하지 않습니다. 따라서 n번 반복하여, 매번 8개의 이웃 복셀을 무작위로 선택하고 6가지 경로를 따라 샘플링하여 정확도를 높입니다.
여기서 normalize_factor의 초기값은 6입니다. 왜냐하면 다음으로 6가지 다른 경로를 통해 누적됩니다.
계속해서 두 배로 증가하며, 각 반복마다 이전 값에 누적되며, 마지막 반복에서만 정규화가 이루어집니다.
```rust
let num_of_neighbors = 8u32;
let mut normalize_factor = 6.0f32;
for i in 1..=self.settings.sign_passes_count {
  // Compute Shader를 디스패치합니다.
  ...
  normalize_factor += num_of_neighbors as f32 * 6.0 * normalize_factor;
}
```
Compute Shader는 다음과 같습니다:
```hlsl
const float4 self_ray_map = _ray_map[id.xyz];
// 8개의 이웃 복셀을 반복하여 선택합니다.
for (uint i = 0; i < g_push_constants.num_of_neighbors; i++) {
  int3 neighbors_offset = generate_random_neighbor_offset((i * g_push_constants.num_of_neighbors) + g_push_constants.pass_id, _max_dimension * 0.05f);
  int3 neighbors_index;
  neighbors_index.x = min((int)(_dimensions.x - 1), max(0, (int)id.x + neighbors_offset.x));
  neighbors_index.y = min((int)(_dimensions.y - 1), max(0, (int)id.y + neighbors_offset.y));
  neighbors_index.z = min((int)(_dimensions.z - 1), max(0, (int)id.z + neighbors_offset.z));

  // 6가지 다른 경로를 통해 이웃 복셀에 도달하는 기호 누적 값을 계산합니다.
  float accum_sign = 0.0f;
  // xyz
  accum_sign += (self_ray_map.x - _ray_map[int3(neighbors_index.x, id.y, id.z)].x);
  accum_sign += (_ray_map[int3(neighbors_index.x, id.y, id.z)].y - _ray_map[int3(neighbors_index.x, neighbors_index.y, id.z)].y);
  accum_sign += (_ray_map[int3(neighbors_index.x, neighbors_index.y, id.z)].z - _ray_map[neighbors_index].z);

  // xzy
  accum_sign += (self_ray_map.x - _ray_map[int3(neighbors_index.x, id.y, id.z)].x);
  accum_sign += (_ray_map[int3(neighbors_index.x, id.y, id.z)].z - _ray_map[int3(neighbors_index.x, id.y, neighbors_index.z)].z);
  accum_sign += (_ray_map[int3(neighbors_index.x, id.y, neighbors_index.z)].y - _ray_map[neighbors_index].y);

  // yxz
  accum_sign += (self_ray_map.y - _ray_map[int3(id.x, neighbors_index.y, id.z)].y);
  accum_sign += (_ray_map[int3(id.x, neighbors_index.y, id.z)].x - _ray_map[int3(neighbors_index.x, neighbors_index.y, id.z)].x);
  accum_sign += (_ray_map[int3(neighbors_index.x, neighbors_index.y, id.z)].z - _ray_map[neighbors_index].z);

  // yzx
  accum_sign += (self_ray_map.y - _ray_map[int3(id.x, neighbors_index.y, id.z)].y);
  accum_sign += (_ray_map[int3(id.x, neighbors_index.y, id.z)].z - _ray_map[int3(id.x, neighbors_index.y, neighbors_index.z)].z);
  accum_sign += (_ray_map[int3(id.x, neighbors_index.y, neighbors_index.z)].x - _ray_map[neighbors_index].x);

  // zyx
  accum_sign += (self_ray_map.z - _ray_map[int3(id.x, id.y, neighbors_index.z)].z);
  accum_sign += (_ray_map[int3(id.x, id.y, neighbors_index.z)].y - _ray_map[int3(id.x, neighbors_index.y, neighbors_index.z)].y);
  accum_sign += (_ray_map[int3(id.x, neighbors_index.y, neighbors_index.z)].x - _ray_map[neighbors_index].x);

  // zxy
  accum_sign += (self_ray_map.z - _ray_map[int3(id.x, id.y, neighbors_index.z)].z);
  accum_sign += (_ray_map[int3(id.x, id.y, neighbors_index.z)].x - _ray_map[int3(neighbors_index.x, id.y, neighbors_index.z)].x);
  accum_sign += (_ray_map[int3(neighbors_index.x, id.y, neighbors_index.z)].y - _ray_map[neighbors_index].y);

  _sign_map_rw[id.xyz] += g_push_constants.normalize_factor * accum_sign + 6 * _sign_map[neighbors_index];
}

// 마지막 반복이 끝날 때 정규화를 수행합니다.
if (g_push_constants.need_normalize) {
  const float normalize_factor_final = g_push_constants.normalize_factor + g_push_constants.num_of_neighbors * 6 * g_push_constants.normalize_factor;
  _sign_map_rw[id.xyz] /= normalize_factor_final;
}
```

이 시점에서 Sign Map을 시각화하면 모델 내부 영역과 외부 영역을 명확하게 구분할 수 있습니다.

![Image Sign Map](images/sign_map.png)

### 여섯 번째 단계: 폐쇄 표면

먼저 메쉬가 완전히 폐쇄되지 않을 수 있으므로, 내부와 외부의 "구멍" 경계를 찾아야 합니다. 이는 지정된 임계값 근처의 Sign Map에서 양수와 음수가 인접한 곳입니다.
```hlsl
// 설정된 임계값을 기준으로 점수를 계산합니다.
const float self_sign_score = _sign_map[id.xyz] - g_push_constants.threshold;
// 점수가 임계값의 10% 미만인 경우.
if (abs(self_sign_score / g_push_constants.threshold) < 0.1f) {
  // 현재 복셀의 점수와 오른쪽 복셀의 점수가 반대인 경우, 경계를 찾은 것입니다.
  if (self_sign_score * (_sign_map[id.xyz + uint3(1, 0, 0)] - g_push_constants.threshold) < 0) {
    // 현재 복셀의 점수를 기준으로 자신 또는 오른쪽 복셀에 기록합니다.
    const uint3 write_coord = id.xyz + (self_sign_score < 0 ? uint3(1, 0, 0) : uint3(0, 0, 0));
    // 기록 내용은 복셀 좌표를 UVW 공간으로 정규화한 값입니다.
    _voxels_texture_rw[write_coord.xyz] = float4((float3(write_coord) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension, 1.0f);
  }
  // Y축도 동일하게 처리합니다.
  ...
  // Z축도 동일하게 처리합니다.
  ...
}
```

다음으로 이전에 처리된 Voxels Buffer의 복셀(삼각형으로 덮인)을 Voxels에 기록합니다.
```hlsl
const float4 voxel = _voxels_buffer[id3(id.xyz)];
if (voxel.w != 0.0f)
  _voxels_texture_rw[id.xyz] = voxel;
```

이로써 모든 경계의 복셀과 삼각형으로 덮인 복셀이 Voxels Texture에 저장되었습니다. 점프 플러딩을 준비합니다.
```hlsl
float best_distance = 1e6f;
float3 best_coord = float3(0.0f, 0.0f, 0.0f);
[unroll(3)]
for (int z = -1; z <= 1; z++) {
  [unroll(3)]
  for (int y = -1; y <= 1; y++) {
    [unroll(3)]
    for (int x = -1; x <= 1; x++) {
      int3 sample_coord;
      sample_coord.x = min((int)(_dimensions.x - 1), max(0, (int)id.x + x * g_push_constants.offset));
      sample_coord.y = min((int)(_dimensions.y - 1), max(0, (int)id.y + y * g_push_constants.offset));
      sample_coord.z = min((int)(_dimensions.z - 1), max(0, (int)id.z + z * g_push_constants.offset));

      float3 seed_coord = _voxels_texture[sample_coord].xyz;
      float dist = length(seed_coord - (float3(id.xyz) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension);
      if ((seed_coord.x != 0.0f || seed_coord.y != 0.0f || seed_coord.z != 0.0f) && dist < best_distance) {
        best_coord = seed_coord;
        best_distance = dist;
      }
    }
  }
}

_voxels_texture_rw[id.xyz] = float4(best_coord, best_distance);
```
여기서 점프 플러딩 전파는 UDF와 거의 동일하므로 자세한 설명은 생략합니다.
유일한 차이점은 실제 점프 전에 오프셋이 1인 초기 점프를 수행하여 표면에 가까운 세부 사항을 미리 완성하는 것입니다. 점프 플러딩은 근사 알고리즘이기 때문입니다.
이로써 Voxels Texture에는 각 복셀이 메쉬 표면에서 가장 가까운 거리의 좌표가 저장되었습니다.

### 일곱 번째 단계: 최종 기호 거리장 계산

```hlsl
// Voxels Texture에서 저장된 시드 좌표와 현재 복셀의 정규화된 VUW 좌표를 가져옵니다.
const float3 seed_coord = _voxels_texture[int3(id.x, id.y, id.z)].xyz;
const float3 voxel_coord = (float3(id.xyz) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension;

// 지정된 Threshold 값을 기준으로 현재 복셀의 기호를 결정합니다.
float sign_d = _sign_map[id.xyz] > g_push_constants.threshold ? -1 : 1;

// 시드 좌표를 기준으로 복셀의 인덱스 좌표를 계산합니다.
const int3 id_seed = int3(seed_coord * _max_dimension);

// 시드 복셀의 삼각형 목록의 시작 및 끝 위치를 가져옵니다.
uint start_triangle_id = 0;
[branch]
if(id3(id_seed) > 0) {
  start_triangle_id = _accum_counters_buffer[id3(id_seed) - 1];
}
uint end_triangle_id = _accum_counters_buffer[id3(id_seed)];

// 모든 삼각형을 반복하여 현재 복셀에서 시드 복셀에 덮인 삼각형까지의 최단 거리를 가져옵니다.
float distance = 1e6f;
for (uint i = start_triangle_id; (i < end_triangle_id) && (i < _upper_bound_count - 1); i++) {
  const uint triangle_index = _triangles_in_voxels[i];
  Triangle tri = _triangles_uvw[triangle_index];
  distance = min(distance, point_distance_to_triangle(voxel_coord, tri));
}
// 특수한 경우, 시드 복셀에 삼각형이 없는 경우, 거리는 현재 복셀에서 시드 복셀의 UVW 좌표 거리로 사용합니다.
if (1e6f - distance < COMMON_EPS) {
  distance = length(seed_coord - voxel_coord);
}
// 기호와 오프셋을 적용하여 기호 거리를 얻습니다.
distance = sign_d * distance - g_push_constants.offset;

// 기호 거리를 Image와 Buffer에 저장합니다. Image는 렌더링에 사용될 수 있으며, Buffer는 내보내기에 사용될 수 있습니다.
_voxels_buffer_rw[id3(id)] = float4(distance, distance, distance, distance);
_distance_texture_rw[id] = distance;
```

이로써 모든 베이킹 계산이 완료되어 SDF 데이터를 얻었습니다. 다음은 SDF를 사용하여 Ray Marching으로 노멀 방향을 색상으로 렌더링한 시각화 결과입니다.

![Image SDF Normal 0](images/sdf_normal_0.png)

![Image SDF Normal 1](images/sdf_normal_1.png)
