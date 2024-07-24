# hala-sdf-baker
[![License](https://img.shields.io/badge/License-GPL3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)
[![MSRV](https://img.shields.io/badge/rustc-1.70.0+-ab6000.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

[English(TODO)](README_EN.md) | [中文](README.md) | [日本語(TODO)](README_JP.md) | [한국어(TODO)](README_KO.md)

## 引言

在现代计算机图形学和游戏开发中，有一个技术被广泛认为是不可或缺的，那就是使用有向距离场（Signed Distance Fields, SDF）和无向距离场（Unsigned Distance Fields, UDF）。SDF和UDF提供了一种高效而强大的手段来表达和操作复杂的几何形状。它们在渲染、碰撞检测、模型生成等多个领域中扮演着重要角色。

SDF是一种典型的表示方法，它为每个点在空间中分配一个实数值，表示该点到最近表面的有向距离。这种结构不但可以用来高效地进行形状建模，还可以用于执行几何操作如平滑、膨胀或缩小形状等。与之相对的，UDF记录的是距离表面的绝对最短距离，这在处理具有不规则或复杂拓扑的模型时特别有用。

SDF和UDF不仅仅是数据结构，它们更是在多维空间中表示形状的一种方法。在视频游戏开发中，利用SDF进行实时阴影计算和环境光遮蔽已成为一种流行的技术。这是因为SDF可以迅速确定光线与几何表面的接触点，从而有效地生成软阴影和其他视觉效果。此外，在实时图形中，采用SDF可以进行高效的几何建模和修改，如角色动态变形，或是开发中常见的破坏效果等。在工业视觉和科学可视化领域，UDF常被用于形状重建和数据拟合，尤其是在处理来自扫描设备或其他测量设备的数据时。通过构建一个准确的UDF，研究者可以从一组离散的数据点中推断出一个连继的三维表面，这对于重建复杂的生物形态或其他科学结构尤为关键。本项目，将通过Rust和Vulkan实现将三维Mesh数据烘焙为SDF和UDF。

![Image Intro](images/intro.png)

图一：来自https://arxiv.org/abs/2011.02570。上半为UDF，只记录了距离表面的绝对最短距离。下半为SDF，除了记录最短距离，正负号还表示了是在“内”还是“外”。

## 开发环境搭建

目前整个开发环境仅在Windows平台上使用RTX 4090和Radeon 780M测试通过（由于本人设备有限暂时无法测试更多的兼容性）。基于`hala-gfx`、`hala-renderer`和`hala-imgui`开发。

* `hala-gfx`负责Vulkan调用和封装。
* `hala-renderer`负责从glTF文件中读取Mesh信息并上传到GPU。
* `hala-imgui`是imGUI的Rust桥接，负责用户界面的显示和互动。

安装1.70+的Rust，如果已经安装`rustup update`为最新版本。使用`git clone --recursive`拉取仓库及其submodule。`cargo build`编译构建Debug版，或者`cargo build -r`构建Release版。

完成编译后可以直接运行。

    ./target/（debug或release）/hala-sdf-baker -c conf/config.yaml -o ./out/output.txt

点击“Bake”按钮进行烘焙，点击“Save”按钮可以把烘焙结果保存到"./out/output.txt"。

输出文件格式为：

    X轴分辨率 Y轴分辨率 Z轴分辨率
    1号体素的值
    2号体素的值
    。。。
    n-1号体素的值
    n号体素的值

## UDF烘焙

算法实现上UDF相对简单，这里先从UDF烘焙讲起。

### 第一步：初始化

在开始烘焙前，需要先分配资源。UDF是体素存储，可以选择Image存储为3D形式，也可以选择Buffer存储为线性形式。这里为了方便后续的可视化调试，存储为3D形式。

烘焙前需要对一些烘焙参数进行设置，其具体作用如下代码中的注释。
```rust
pub selected_mesh_index: i32, // glTF中可能保存着多个Mesh数据，此字段决定将要被烘焙的是第几个Mesh。
pub max_resolution: i32,      // 烘焙输出的体素的体的最长轴的分辨率。比如大小为(1, 2, 4）的Mesh范围，此字段如果为64，那么最终体素的分辨率将是[16, 32, 64]。
pub surface_offset: f32,      // 此偏移值会叠加到最终烘焙出的数据上。
pub center: [f32; 3],         // 待烘焙数据的BoundingBox的中心位置。
pub desired_size: [f32; 3],   // 根据Mesh的BoundingBox大小、max_resolution和指定的边缘预留大小padding计算出的计划烘焙空间的大小。
pub actual_size: [f32; 3],    // 根据desired_size调整大小为体素大小的整倍数，也是最终保存数据的大小。
pub padding: [f32; 3],        // 在Mesh的BoundingBox外扩大多少个体素作为边界。
```

center和desired_size计算方法如下：
```rust
fn fit_box_to_bounds(&mut self) {
  // 获取待烘焙Mesh的BoundingBox。
  let bounds = self.get_selected_mesh_bounds().unwrap();

  // 计算最长边长。
  let max_size = bounds.get_size().iter().fold(0.0, |a: f32, b| a.max(*b));
  // 通过指定的最大分辨率计算出单个体素的大小。
  let voxel_size = max_size / self.settings.max_resolution as f32;
  // 根据体素大小计算出外扩边界的大小。
  let padding = [
    self.settings.padding[0] * voxel_size,
    self.settings.padding[1] * voxel_size,
    self.settings.padding[2] * voxel_size,
  ];

  // 最终获得整个待烘焙区域的中心和大小。
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

actual_size的计算方法如下：
```rust
fn snap_box_to_bounds(&mut self) {
  // 计算待烘焙区域的最长边长
  let max_size = self.settings.desired_size.iter().fold(0.0, |a: f32, b| a.max(*b));
  // 将最长边所在轴确定为参考轴，此轴向的体素数将为设定的最大分辨率值。
  let ref_axis = if max_size == self.settings.desired_size[0] {
    Axis::X
  } else if max_size == self.settings.desired_size[1] {
    Axis::Y
  } else {
    Axis::Z
  };

  // 根据参考轴的不同，先计算出单个体素的大小，然后计算出体素大小整倍数的待烘焙区域的大小。
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

接下来准备全局的UBO，用于存储整个烘焙过程中都需要用到的一些参数，具体如下代码中的注释。
```rust
pub struct GlobalUniform {
  pub dimensions: [u32; 3],   // 根据需要烘焙Mesh的BoundingBox信息和烘焙体素最大分辨率计算出三个维度的大小。
  pub num_of_voxels: u32,     // 总体素的数量，其值为dimensions[0] * dimensions[1] * dimensions[2]。
  pub num_of_triangles: u32,  // 待烘焙Mesh的总三角形数量。
  pub initial_distance: f32,  // 初始化UDF的值。根据整个烘焙区域最长边的长度，归一化后的烘焙BoundingBox的对角线长度的1.01倍（整个UDF中不可能有值大于此值）。
  pub max_size: f32,          // 根据整个烘焙区域最长边的长度。
  pub max_dimension: u32,     // 整个体素空间最长边的体素数量。
  pub center: [f32; 3],       // 烘焙区域BoundingBox的中心坐标。
  pub extents: [f32; 3],      // 烘焙区域BoundingBox的半长。
}
```

根据以上计算的体素空间的三个轴向的体素数量，创建一个Image资源。这里设置Usage为Storage是为了其后在Shader中对其进行写入，设置为Sampled是为了进行读取。
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

### 第二步：填入初始值

这一步最为简单。唯一需要注意的是这里写入的不是初始距离的float形式，而是uint。这在下一个Shader中会详细解释。
```hlsl
_distance_texture_rw[int3(id.x, id.y, id.z)] = float_flip(_initial_distance);
```

接下来是遍历Mesh中的所有三角形，id.x是正在遍历的三角形的索引号。
```hlsl
Triangle tri_uvw;
tri_uvw.a = (get_vertex_pos(id.x, 0) - _center + _extents) / _max_size;
tri_uvw.b = (get_vertex_pos(id.x, 1) - _center + _extents) / _max_size;
tri_uvw.c = (get_vertex_pos(id.x, 2) - _center + _extents) / _max_size;
```
首先通过get_vertex_pos函数从Mesh的index buffer和vertex buffer中读取顶点的位置信息。
然后通过传入的center和extents将顶点平移到三维空间中的第一卦限。
最后根据max_size的值归一化到[0, 1]范围的uvw空间。

| 阶段 | 描述 |
|------|------|
|![Image Bound 0](images/bound_0.png)| *原始区域* |
|![Image Bound 1](images/bound_1.png)| *平移到第一卦限* |
|![Image Bound 2](images/bound_2.png)| *归一化到UVW空间* |

紧接着计算三角形所覆盖区域的AABB，然后通过_max_dimension变换到体素空间并向外扩大一圈。
```hlsl
const float3 aabb_min = min(tri_uvw.a, min(tri_uvw.b, tri_uvw.c));
const float3 aabb_max = max(tri_uvw.a, max(tri_uvw.b, tri_uvw.c));
int3 voxel_min = int3(aabb_min * _max_dimension) - GRID_MARGIN;
int3 voxel_max = int3(aabb_max * _max_dimension) + GRID_MARGIN;
voxel_min = max(0, min(voxel_min, int3(_dimensions) - 1));
voxel_max = max(0, min(voxel_max, int3(_dimensions) - 1));
```

最后循环遍历AABB所覆盖的所有体素，计算体素中心离三角形的距离，并写入到Distance Texture中。
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
注意，这里使用了InterlockedMin原子比较写入最小值函数，因为此时多个GPU线程可能在同时更新同一个体素。
此外还使用float_flip将float类型的距离转换为了uint，原因是InterlockedMin需要操作uint类型数据（并不是所有硬件都支持float的InterlockedMin）。
这里详细看一下float_flip函数的实现。
```hlsl
inline uint float_flip(float fl) {
  uint f = asuint(fl);
  return (f << 1) | (f >> 31);
}
```
此函数将float数值的第一位也就是符号位移动到了最后，这样通过InterlockedMin比较的时候就能够获取到绝对值最小的值，符合UDF的定义。

![Image IEEE 754](images/ieee_754.png)

通过float类型的定义可以看出，只要将符号位放到最后一位，就可以和uint一样比较大小了。

完成所有三角形的处理后，再使用float_unflip函数将符号位移动回原来的位置。

```hlsl
const int3 uvw = int3(id.x, id.y, id.z);
const uint distance = _distance_texture_rw[uvw];
_distance_texture_rw[uvw] = float_unflip(distance);
```

至此Distance Texture中，被三角形覆盖的体素，都记录了到Mesh表面最近的距离（无符号）。但没有被三角形覆盖到的区域还是初始值，接下来将要处理这些区域。

### 第三步：跳跃泛洪

跳跃泛洪（Jump Flooding）是一种用于计算距离变换（Distance Transform）和Voronoi图（Voronoi Diagram）的高效算法，常用于图像处理和计算几何领域。与传统的逐像素传播方法不同，跳跃泛洪算法通过以指数递增的步长“跳跃”而不是逐像素传播，从而极大地提高了计算速度。

#### 工作原理

跳跃泛洪算法的核心思想是通过一系列递减的“跳跃”步骤来传播距离信息。具体来说，算法从初始种子点开始，以较大的步长同时更新多个距离值，然后逐步减小步长进行更细致的更新。每次跳跃过程中，算法会检查当前像素的邻居，并更新其距离值，以确保最优解的传播。

首先泛洪算法需要两个Buffer交替使用。这里设置Usage为TRANSFER_SRC是为了后续可以从GPU传输到CPU端，然后保存成文件。
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

值得一提的是由于两个Buffer来回切换使用，所以预先创建两个DescriptorSet分别按不同的顺序绑定Buffer方便后续使用。
```rust
// 在奇数步跳跃时，从jump_buffer读取数据，写入jump_buffer_bis。
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

// 在偶数步跳跃时，从jump_buffer_bis读取数据，写入jump_buffer。
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

接下来进行泛洪跳跃的初始化，初始种子为认为自己是最优解。
```hlsl
  const float distance = _distance_texture[int3(id.x, id.y, id.z)];
  const uint voxel_index = id3(id.x, id.y, id.z);
  _jump_buffer_rw[voxel_index] = voxel_index;
```

对最大分辨率求log2获得总计需要跳跃多少步。每步开始offset都缩小为前一步的一半。
```rust
let num_of_steps = self.settings.max_resolution.ilog2();
for i in 1..=num_of_steps {
  let offset = ((1 << (num_of_steps - i)) as f32 + 0.5).floor() as i32;
  // 循环迭代，每次从一个Buffer把数据泛洪到另一个Buffer。
  ...
}
```

从当前体素向周围26个方向跳跃采样，并记录距离Mesh表面的最短距离（最优解）更新跳跃Buffer。
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
*注意这里没有对x == 0 && y == 0 && z == 0做判断，因为当前体素如果已经是最短距离后续更新也不会有影响。*

具体的跳跃采样代码如下：
```hlsl
void jump_sample(int3 center_coord, int3 offset, inout float best_distance, inout int best_index) {
  // 当前坐标加上偏移获取采样坐标。
  int3 sample_coord = center_coord + offset;
  // 如果采样坐标超出了整个体素体的范围怎不做任何操作。
  if (
    sample_coord.x < 0 || sample_coord.y < 0 || sample_coord.z < 0 ||
    sample_coord.x >= _dimensions.x || sample_coord.y >= _dimensions.y || sample_coord.z >= _dimensions.z
  ) {
    return;
  }
  // 获取采样坐标下的种子索引。
  uint voxel_sample_index = _jump_buffer[id3(sample_coord)];
  // 将索引转换为x, y, z的坐标形式。
  int3 voxel_sample_coord = unpack_id3(voxel_sample_index);
  // 获取此坐标到Mesh表面的最近距离。
  float voxel_sample_distance = _distance_texture[voxel_sample_coord];
  // 总距离为当前坐标到采样坐标的距离加上采样坐标到Mesh表面的最近距离。
  // 注：此处除以max_dimension是为了转换到UVW空间计算，因为Distance Texture中保存的是UVW空间中的距离。
  float distance = length(float3(center_coord) / _max_dimension - float3(voxel_sample_coord) / _max_dimension) + voxel_sample_distance;
  // 如果以上计算得出的跳跃距离比之前的都要小，则更新最优解。
  if (distance < best_distance) {
    best_distance = distance;
    best_index = voxel_sample_index;
  }
}
```

此算法重复完num_of_steps次后，每个体素格子都完成了最优解的传播。这里以一维空间举例，假设最大分辨率为8，那么log2(8)=3需要三步跳跃，每次跳跃分别距离是4, 2, 1。

    第一步：
    体素0 计算0->4是否存在最优解
    体素1 计算1->5是否存在最优解
    体素2 计算2->6是否存在最优解
    体素3 计算3->7是否存在最优解
    体素4 计算4->0是否存在最优解
    体素5 计算5->1是否存在最优解
    体素6 计算6->2是否存在最优解
    体素7 计算7->3是否存在最优解
    第二步：
    体素0 计算0->2是否存在最优解
    体素1 计算1->3是否存在最优解
    体素2 计算2->4, 2->0是否存在最优解
    体素3 计算3->5, 3->1是否存在最优解
    体素4 计算4->6, 4->2是否存在最优解
    体素5 计算5->7, 5->3是否存在最优解
    体素6 计算6->4是否存在最优解
    体素7 计算7->5是否存在最优解
    第三步：
    体素0 计算0->1是否存在最优解
    体素1 计算1->2, 1->0是否存在最优解
    体素2 计算2->3, 2->1是否存在最优解
    体素3 计算3->4, 3->2是否存在最优解
    体素4 计算4->5, 4->3是否存在最优解
    体素5 计算5->6, 5->4是否存在最优解
    体素6 计算6->7, 6->5是否存在最优解
    体素7 计算7->6是否存在最优解

这里假设4为没有被三角形覆盖的体素，整个计算过程计算过4->0, 4->2, 4->3, 4->5, 4->6，那如果假设1为被三角形覆盖的体素？4是否就没法被计算了呢？
可以看到在第一步中计算过5->1是否存在最优解，那么此时5的索引已经更新成了1，在第三步计算4->5时其实计算的是4->1是否存在最优解。

在执行完以上步骤后，最终需要更新Distance Texture。
```hlsl
// 当前体素坐标。
const uint voxel_index = id3(id.x, id.y, id.z);

// 通过Jump Buffer获取最优的体素Index。
const uint cloest_voxel_index = _jump_buffer[voxel_index];
// 将Index转换为坐标。
const int3 cloest_voxel_coord = unpack_id3(cloest_voxel_index);
// 获取这个最优的体素坐标中保存的到Mesh的最短距离。
const float cloest_voxel_distance = _distance_texture_rw[cloest_voxel_coord];

// 当前体素到最优体素的距离（UVW空间，原因同前）。
const float distance_to_cloest_voxel = length(float3(id) / _max_dimension - float3(cloest_voxel_coord) / _max_dimension);

// 最终距离等于当前体素到最优体素的距离加上最优体素到Mesh的距离再加上烘焙设置中指定的Offset。
_distance_texture_rw[int3(id.x, id.y, id.z)] = cloest_voxel_distance + distance_to_cloest_voxel + g_push_constants.offset;
```
*注意：跳跃泛洪Jump Flooding算法是一种快速近似的方法，并不能保证每个体素都更新为最短距离。*

至此Distance Texture已经保存了计算完成UDF数据。可以进行可视化了。

![Image UDF](images/udf.png)

从图中可以看到越接近Mesh表面的地方颜色越深（数值小距离近），越远离的地方越亮（数值大距离远）。

也可以通过等值面重建Mesh。

![Image UDF Mesh](images/udf_mesh.png)


## SDF烘焙

相比UDF来说，SDF的烘焙则要复杂得多。这里的实现参考自Unity的中[Visual Effect Graph](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@14.0/manual/sdf-in-vfx-graph.html)的方案。

### 第一步：初始化

增加烘焙配置项：
```rust
pub sign_passes_count: i32, // 符号Pass（寻找符号是正还是负）的迭代次数。
pub in_out_threshold: f32,  // 判断是在Mesh内还是外的阈值。
```

接下来准备全局的UBO，用于存储整个烘焙过程中都需要用到的一些参数，具体如下代码中的注释。
```rust
pub struct GlobalUniform {
  pub dimensions: [u32; 3],   // 根据需要烘焙Mesh的BoundingBox信息和烘焙体素最大分辨率计算出三个维度的大小。
  pub upper_bound_count: u32, // 存放每个体素中包含三角形的Buffer的上界。
  pub num_of_triangles: u32,  // 待烘焙Mesh的总三角形数量。
  pub max_size: f32,          // 根据整个烘焙区域最长边的长度。
  pub max_dimension: u32,     // 整个体素空间最长边的体素数量。
  pub center: [f32; 3],       // 烘焙区域BoundingBox的中心坐标。
  pub extents: [f32; 3],      // 烘焙区域BoundingBox的半长。
}
```
其它值的计算都同UDF，关于upper_bound_count，由于无法确定每个体素到底包含多少三角形，所以这里只能估算一个最大值。
```rust
// 首先假设有一半的体素中有三角形。
let num_of_voxels_has_triangles = dimensions[0] as f64 * dimensions[1] as f64 * dimensions[2] as f64 / 2.0f64;
// 假设一个三角形会被相邻的8个体素共享。假设每个体素会拥有总三角形数的平方根数量的三角形。
// 这里对以上两个假设取最大值。
let avg_triangles_per_voxel = (num_of_triangles as f64 / num_of_voxels_has_triangles * 8.0f64).max((num_of_triangles as f64).sqrt());
// 总计需要存储的三角形数。
let upper_bound_count64 = (num_of_voxels_has_triangles * avg_triangles_per_voxel) as u64;
// 限制最大值为1536 * 2^18。
let upper_bound_count = (1536 * (1 << 18)).min(upper_bound_count64) as u32;
// 限制最小值为1024。
let upper_bound_count = upper_bound_count.max(1024);
```
*注意：这里只是一个保守推测，实际需要的数量可能远远小于此值。进行保守推测只是为了覆盖更多的边界情况。*

在整个SDF的烘焙过程中需要大量的临时Buffer，这里就先不做介绍，后续在每一步中再详细介绍。

### 第二步：构建几何体

首先，如同UDF一样从Mesh的Vertex Buffer和Index Buffer中读取三角形信息，并变换到归一化的UVW空间，保存到Triangle UVW Buffer中。
```hlsl
Triangle tri_uvw;
tri_uvw.a = (get_vertex_pos(id.x, 0) - _center + _extents) / _max_size;
tri_uvw.b = (get_vertex_pos(id.x, 1) - _center + _extents) / _max_size;
tri_uvw.c = (get_vertex_pos(id.x, 2) - _center + _extents) / _max_size;

_triangles_uvw_rw[id.x] = tri_uvw;
```

接下来，计算每个三角形的“方向”。这里的“方向”表示三角形大体朝向哪个轴，既和XY、ZX、YZ哪个平面更接近。结果保存到Coord Flip Buffer中。
```hlsl
const float3 a = get_vertex_pos(id.x, 0);
const float3 b = get_vertex_pos(id.x, 1);
const float3 c = get_vertex_pos(id.x, 2);
const float3 edge0 = b - a;
const float3 edge1 = c - b;
const float3 n = abs(cross(edge0, edge1));
if (n.x > max(n.y, n.z) + 1e-6f) {  // Plus epsilon to make comparison more stable.
  // Triangle nearly parallel to YZ plane
  _coord_flip_buffer_rw[id.x] = 2;
} else if (n.y > max(n.x, n.z) + 1e-6f) {
  // Triangle nearly parallel to ZX plane
  _coord_flip_buffer_rw[id.x] = 1;
} else {
  // Triangle nearly parallel to XY plane
  _coord_flip_buffer_rw[id.x] = 0;
}
```
这里为什么是ZX平面而不是XZ平面，是因为后续分别需要在3个方向进行计算，ZX平面表示在Y轴方向计算时，局部的X轴实际是Z，局部的Y轴实际是X。

既然已经为每个三角形分配好了方向，接下来就是在每个方向上对三角形进行保守光栅化。
在此之前先计算三个方向上的正交和投影矩阵。
```rust
// 根据视点位置，旋转轴向，宽度，高度，近平面距离和远平面距离构造View矩阵和Proj矩阵。
let calculate_world_to_clip_matrix = |eye, rot, width: f32, height: f32, near: f32, far: f32| {
  let proj = glam::Mat4::orthographic_rh(-width / 2.0, width / 2.0, -height / 2.0, height / 2.0, near, far);
  let view = glam::Mat4::from_scale_rotation_translation(glam::Vec3::ONE, rot, eye).inverse();
  proj * view
};
```

Z方向的XY平面如下图所示，局部X轴为世界的X轴，局部Y轴为世界的Y轴。

![Image XY Plane](images/xy_plane.png)

```rust
let xy_plane_mtx = {
  // 视点在正Z方向加1的位置向下看。
  let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(0.0, 0.0, bounds.extents[2] + 1.0);
  // View空间默认向下看，不需要旋转。
  let rot = glam::Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0);
  // 近平面在1，这就是视点位置为什么加1，给近平面留出空间。
  let near = 1.0f32;
  // 远平面等于从近平面开始延申出整个包围盒的Z方向长度。
  let far = near + bounds.extents[2] * 2.0;
  calculate_world_to_clip_matrix(pos, rot, bounds.extents[0] * 2.0, bounds.extents[1] * 2.0, near, far)
};
```

Y方向的ZX平面如下图所示，局部X轴为世界的Z轴，局部Y轴为世界的X轴。

![Image ZX Plane](images/zx_plane.png)

```rust
let zx_plane_mtx = {
  // 视点在正Y方向加1的位置向外看（从Y轴的正向向负向看）。
  let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(0.0, bounds.extents[1] + 1.0, 0.0);
  // 首先沿Y轴旋转-90度，再沿X轴旋转-90度。让局部X轴对齐世界Z轴，局部Y轴对齐世界X轴。
  let rot = glam::Quat::from_euler(glam::EulerRot::YXZ, -std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2, 0.0);
  let near = 1.0f32;
  let far = near + bounds.extents[1] * 2.0;
  calculate_world_to_clip_matrix(pos, rot, bounds.extents[2] * 2.0, bounds.extents[0] * 2.0, near, far)
};
```

X方向的YZ平面如下图所示，局部X轴为世界的Y轴，局部Y轴为世界的Z轴。

![Image YZ Plane](images/yz_plane.png)

```rust
let yz_plane_mtx = {
  // 视点再正X方向加1的位置向左看。
  let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(bounds.extents[0] + 1.0, 0.0, 0.0);
  // 首先沿X轴旋转90度，再沿Y轴旋转90度。让局部X轴对齐世界Y轴，局部Y轴对齐世界Z轴。
  let rot = glam::Quat::from_euler(glam::EulerRot::XYZ, std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2, 0.0);
  let near = 1.0f32;
  let far = near + bounds.extents[0] * 2.0;
  calculate_world_to_clip_matrix(pos, rot, bounds.extents[1] * 2.0, bounds.extents[2] * 2.0, near, far)
};
```

接下来就是在以上三个方向上，对对应方向的三角形进行保守光栅化处理。
首先计算三角形覆盖范围的二维AABB保存到float4中，xy保存min，zw保存max。
```hlsl
// 获取三角形的三个顶点，并变换到clip空间。
[unroll(3)]
for (i = 0; i < 3; i++) {
  vertex_in_clip[i] = mul(_world_to_clip[current_axis], float4(get_vertex_pos(id.x, i), 1.0));
}

// 计算AABB的大小。
float4 aabb = float4(1.0, 1.0, -1.0, -1.0);
aabb.xy = min(aabb.xy, min(vertex_in_clip[0].xy, min(vertex_in_clip[1].xy, vertex_in_clip[2].xy)));
aabb.zw = max(aabb.xy, max(vertex_in_clip[0].xy, max(vertex_in_clip[1].xy, vertex_in_clip[2].xy)));
float2 conservative_pixel_size;
// 根据当前光栅化的方向，根据设置的Conservative Offset参数计算实际需要的Offset像素大小。
if (current_axis == 0) {
  conservative_pixel_size = float2(_conservative_offset / _dimensions.x, _conservative_offset / _dimensions.y);
} else if (current_axis == 1) {
  conservative_pixel_size = float2(_conservative_offset / _dimensions.z, _conservative_offset / _dimensions.x);
} else {
  conservative_pixel_size = float2(_conservative_offset / _dimensions.y, _conservative_offset / _dimensions.z);
}

// 对AABB大小进行扩大。
_aabb_buffer_rw[id.x] = aabb + float4(-conservative_pixel_size.x, -conservative_pixel_size.y, conservative_pixel_size.x, conservative_pixel_size.y);
```

然后对三角形进行光栅化，并扩大设置的Offset。这里之所以保守光栅化扩大，是防止float计算时的误差导致漏“缝隙”。
```hlsl
// 构建三角形所在平面存入float4，xyz为平面法线方向，w为平面距离原点的距离。
const float3 normal = normalize(cross(vertex_in_clip[1].xyz - vertex_in_clip[0].xyz, vertex_in_clip[2].xyz - vertex_in_clip[0].xyz));
const float4 triangle_plane = float4(normal, -dot(vertex_in_clip[0].xyz, normal));

// 计算法线方向是向Z正方向（1）还是负方向（-1）。
const float direction = sign(dot(normal, float3(0, 0, 1)));
float3 edge_plane[3];
[unroll(3)]
for (i = 0; i < 3; i++) {
  // 计算2D边平面。W是齐次坐标。
  edge_plane[i] = cross(vertex_in_clip[i].xyw, vertex_in_clip[(i + 2) % 3].xyw);
  // 根据之前确定的方向和偏移像素值将边平面向“外”推动一段距离。
  // 这里不好理解后面可以看图。
  edge_plane[i].z -= direction * dot(conservative_pixel_size, abs(edge_plane[i].xy));
}

float4 conservative_vertex[3];
bool is_degenerate = false;
[unroll(3)]
for (i = 0; i < 3; i++) {
  _vertices_buffer_rw[3 * id.x + i] = float4(0, 0, 0, 1);

  // 根据三条边的边平面，进行相交得到新的顶点位置。
  conservative_vertex[i].xyw = cross(edge_plane[i], edge_plane[(i + 1) % 3]);

  // 根据W值判断三角形是否退化。
  if (abs(conservative_vertex[i].w) < CONSERVATIVE_RASTER_EPS) {
    is_degenerate |= true;
  } else {
    is_degenerate |= false;
    conservative_vertex[i] /= conservative_vertex[i].w; // after this, w is 1.
  }
}
if (is_degenerate)
  return;

// 通过三角形上的点，满足平面公式计算三个顶点的Z值。
// 平面公式：ax + by + cz + d = 0。
// 计算Z：z = -(ax + by + d) / c。
// 最后将新得到的三个顶点写入Vertices Buffer。
[unroll(3)]
for (i = 0; i < 3; i++) {
  conservative_vertex[i].z = -(triangle_plane.x * conservative_vertex[i].x + triangle_plane.y * conservative_vertex[i].y + triangle_plane.w) / triangle_plane.z;
  _vertices_buffer_rw[3 * id.x + i] = conservative_vertex[i];
}
```
在计算机图形学中，一个平面可以用一个四维向量来表示：float4(plane) = (a, b, c, d)，其中平面的方程为 ax + by + cz + d = 0。一个“边平面”的概念是基于这样一个想法：当处理2D投影上的几何体（比如三角形），可以用分割空间的平面来代表三角形的边界。

    edge_plane[i] = cross(vertex_in_clip[i].xyw, vertex_in_clip[(i + 2) % 3].xyw);

在这段代码中，具体构建边平面的方法是通过两个顶点的齐次坐标的叉积来获得。这里，vertex_in_clip 是顶点的齐次坐标。vertex_in_clip[i].xyw 提取的是顶点的 x, y, w 分量，将其视为 3 维向量。cross 函数计算两个3维向量的叉积，生成一个垂直于这两个向量所在平面的向量。这个生成的向量 edge_plane[i] 就代表了从 vertex_in_clip[i] 到 vertex_in_clip[(i + 2) % 3] 的边界平面（注意是2D平面在齐次坐标下的表示）。

这里将保守光栅化后的三角形还原到模型空间，红色线框为放大后的三角形，白色线框为原始三角形。可以看到每个三角形都沿其所在平面扩大了一圈。

![Image Conservative Offset](images/conservative_offset.png)

### 第三步：三角形覆盖体素计数统计

接下来暂时离开一下Compute Shader，我们要利用Vertex Shader和Fragment Shader统计一下三个方向上的三角形覆盖次数。
先来看一下Vertex Shader。
```hlsl
struct VertexInput {
  // 通过Draw(num_of_triangles * 3)，传递进Vertex Id。
  uint vertex_id: SV_VertexID;
};

ToFragment main(VertexInput input) {
  ToFragment output = (ToFragment)0;

  // 根据Vertex Id去上一步光栅化结果的Vertices Buffer中直接读取Clip空间中的顶点数据。
  const float4 pos = _vertices_buffer[input.vertex_id];
  // Vertex Id简单的除以3得到三角形ID。
  output.triangle_id = input.vertex_id / 3;
  // 如果当前三角形与当前绘制方向不同，则传递(-1, -1, -1, -1)使得Fragment Shader被跳过。
  if (_coord_flip_buffer[output.triangle_id] != g_push_constants.current_axis) {
    output.position = float4(-1, -1, -1, -1);
  } else {
    output.position = pos;
  }

  return output;
}
```

先总体看一下Fragment Shader的流程。
```hlsl
struct ToFragment {
  float4 position: SV_Position;
  uint triangle_id: TEXCOORD0;
};

FragmentOutput main(ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

  // 根据Vertex Shader传过来的position和三角形ID计算当前处理像素的体素坐标voxel_coord。
  // 同时判断是否可以同时处理在深度方向上向里backword和向外forward进行扩展处理。
  int3 depth_step, voxel_coord;
  bool can_step_backward, can_step_forward;
  get_voxel_coordinates(input.position, input.triangle_id, voxel_coord, depth_step, can_step_backward, can_step_forward);

  // 将体素中心坐标转换到归一化的UVW空间。并存储到Voxels Buffer中。
  float3 voxel_uvw = (float3(voxel_coord) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension;
  _voxels_buffer_rw[id3(voxel_coord)] = float4(voxel_uvw, 1.0f);
  // 在当前体素坐标的Counter Buffer进行累加，标记此体素被三角形覆盖一次。
  InterlockedAdd(_counter_buffer_rw[id3(voxel_coord)], 1u);
  // 如果能向外扩展，对向外的一个体素进行同样操作。
  if (can_step_forward) {
    _voxels_buffer_rw[id3(voxel_coord + depth_step)] = float4(voxel_uvw, 1.0f);
    InterlockedAdd(_counter_buffer_rw[id3(voxel_coord + depth_step)], 1u);
  }
  // 如果能向内扩展，对向内的一个体素进行同样操作。
  if (can_step_backward) {
    _voxels_buffer_rw[id3(voxel_coord - depth_step)] = float4(voxel_uvw, 1.0f);
    InterlockedAdd(_counter_buffer_rw[id3(voxel_coord - depth_step)], 1u);
  }

  // 这里RT的输出并不参与到烘焙过程中，仅仅作为调试使用。
  output.color = float4(voxel_uvw, 1);
  return output;
}
```
总体流程大概就是利用VS和FS，在三角形覆盖区域对Counter Buffer进行累加操作。
现在来详细看一下`get_voxel_coordinates`的实现。
```hlsl
void get_voxel_coordinates(
  float4 screen_position,
  uint triangle_id,
  out int3 voxel_coord,
  out int3 depth_step,
  out bool can_step_backward,
  out bool can_step_forward
) {
  // 获取当前屏幕分辨率，既当前方向上的体素宽和高。
  // 比如当前体素空间如果是[2, 3, 4]，那么在进行Z方向XY平面进行处理时返回2 x 3。
  const float2 screen_params = get_custom_screen_params();
  // 将Vertex Shader传递过来的Position转换到UVW空间。
  screen_to_uvw(screen_position, screen_params);
  // 根据三角形ID获取之前计算得到的三角形覆盖区域的AABB，判断如果不在AABB范围内将会Discard掉当前Fragment Shader的后续执行。
  cull_with_aabb(screen_position, triangle_id);
  // 计算体素坐标和决定是否可以向前和向后扩展。
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
再来详细看一下`compute_coord_and_depth_step`的实现。
```hlsl
void compute_coord_and_depth_step(
  float2 screen_params,
  float4 screen_position,
  out int3 voxel_coord,
  out int3 depth_step,
  out bool can_step_backward,
  out bool can_step_forward
) {
  // 这里我们保守的认为三角形会被相邻的前后体素共享，这样可以避免后续一些显示上的问题。
  can_step_forward = true;
  can_step_backward = true;

  if (g_push_constants.current_axis == 1) {
    // 通过UVW空间中的Position计算体素坐标。
    voxel_coord = (screen_position.xyz * float3(screen_params, _dimensions[1]));
    voxel_coord.xyz = voxel_coord.yzx;

    // 判断是否是边界，不是的话则可以向内和向外扩展。
    depth_step = int3(0, 1, 0);
    if (voxel_coord.y <= 0) {
      can_step_backward = false;
    }
    if (voxel_coord.y >= _dimensions[1] - 1) {
      can_step_forward = false;
    }
  } else if (g_push_constants.current_axis == 2) {
    // 基本同上，只是具体轴的方向不同。
  } else {
    // 基本同上，只是具体轴的方向不同。
  }
}
```
由于深度写入和深度测试被关闭，至此在三个方向上，三角形覆盖的体素都通过InterlockedAdd对Counter Buffer进行了计数。
同时这些被覆盖体素的UVW坐标也存入到了Voxels Buffer中。

接下来是利用Prefix Sum算法对Counter Buffer进行累加，最终结果存入Accum Counter Buffer中。其基本思想是通过预处理步骤，将数组中的每个位置之前所有元素之和存储起来，从而使得后续的查询操作可以在常数时间内完成。
由于Prefix Sum算法和烘焙本身并无直接关系，这只给出相关算法的介绍连接：
* [维基百科](https://en.wikipedia.org/wiki/Prefix_sum)，
* [GPU Gems 3 - Chapter 39. Parallel Prefix Sum (Scan) with CUDA](https://developer.nvidia.com/gpugems/gpugems3/part-vi-gpu-computing/chapter-39-parallel-prefix-sum-scan-cuda)

此时Accum Counter Buffer中已经保存当前体素之前的所有体素包含（被覆盖）的三角形数。
举个例子，体素0，1，2，3，4。分别被4，2，5，0，3个三角形覆盖。那么此时计数Buffer中的值为：

    0（当前体素之前没有其它体素）
    4（当前体素之前是0号，0号有4个三角形）
    6（当前体素之前是0号和1号，0号有4个三角形，1号有2个三角形，总和6个）
    11（算法同上）
    11（算法同上）

接下来就是把这些三角形存入Triangle Id Buffer，并可以通过Accum Counter Buffer遍历每个体素所包含的三角形列表。这里同样使用Vertex Shader和Fragment Shader。

Vertex Shader和之前一样，这里就不再重复了。只看一下Fragment的不同点。
```hlsl
// 此处通过计算出的体素坐标对Counter Buffer加1后返回原值用于写入Triangle Ids Buffer的索引。
uint index = 0u;
InterlockedAdd(_counter_buffer_rw[id3(voxel_coord)], 1u, index);
// 这里为了防止越界，用到了一开始计算的每体素三角形Buffer的上限。
if (index < _upper_bound_count)
_triangle_ids_buffer_rw[index] = input.triangle_id;
// 同样对向外和向内的体素做扩展。
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
这里很好理解，体素i之前的体素一共有多少个三角形覆盖已经存入了Counter Buffer，那么`_counter_buffer_rw[id3(voxel_coord)]`取到的就是当前体素i可以写入三角形索引的开始位置。

### 第四步：计算Ray Map

完成以上所有计算后，后续都可以通过如下代码遍历指定体素的三角形列表了。
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

在开始计算Ray Map之前先引入几个辅助函数。
```hlsl
// 计算线段与三角形的交点，不相交返回0，与三角形边缘相交返回0.5或-0.5，与三角形内部相交返回1.0或-1.0。
// 符号表示是与三角形正面还是反面相交。t返回交点参数。
float intersect_segment_to_triangle_with_face_check(float3 segment_start, float3 segment_end, Triangle tri, out float t_value) {
  /*
   * 三角形平面方程：n * (P - A) = 0
   * 线段方程：P(t) = Q + t(S - Q)
   * n dot ((Q + t(S - Q)) - A) = 0
   * n dot (Q - A + t(S - Q)) = 0
   * n dot (Q - A) + t(n dot (S - Q)) = 0
   * 𝑣 = 𝑄 - 𝐴, 𝑑 = 𝑆 − 𝑄
   * t = - (n dot 𝑣) / (n dot d)
   *
   * 其中：
   * n - 三角形平面的法向量
   * P - 三角形平面上的任意点
   * A - 三角形的一个顶点
   * Q, S - 线段的两个端点
   * t - 交点参数，用于描述线段与三角形的交点
   * 𝑣 - 向量 Q - A
   * 𝑑 - 向量 S - Q
   */

  // 计算三角形的两条边。
  const float3 edge1 = tri.b - tri.a;
  const float3 edge2 = tri.c - tri.a;
  // 这里实际计算的是 -d = Q - S。
  const float3 end_to_start = segment_start - segment_end;

  // 通过叉乘计算出三角形平面的法向量。
  const float3 normal = cross(edge1, edge2);
  // 计算线段方向与三角形法向量的点积。
  const float dot_product = dot(end_to_start, normal);
  // 此点乘结果的符号代表着是线段与三角形正面还是反面相交。
  const float side = sign(dot_product);
  // 取倒数。
  const float inverse_dot_product = 1.0f / dot_product;

  // v = Q - A
  const float3 vertex0_to_start = segment_start - tri.a;
  // 根据公式，计算出交点的t值。
  // t = - (n dot v) / (n dot d)
  //   = (n dot v) / (n dot -d)
  float t = dot(vertex0_to_start, normal) * inverse_dot_product;

  // 如果t值小于0或者大于1则意味着线段和三角形平面没有交点。
  if (t < -INTERSECT_EPS || t > 1 + INTERSECT_EPS) {
    t_value = 1e10f;
    return 0;
  } else {
    // 计算质心坐标检测交点是否在三角形内部。
    const float3 cross_product = cross(end_to_start, vertex0_to_start);
    const float u = dot(edge2, cross_product) * inverse_dot_product;
    const float v = -dot(edge1, cross_product) * inverse_dot_product;
    float edge_coefficient = 1.0f;

    // 如果质心坐标不在指定的范围，则交点在三角形外部。
    if (u < -BARY_EPS || u > 1 + BARY_EPS || v < -BARY_EPS || u + v > 1 + BARY_EPS) {
      t_value = 1e10f;
      return 0;
    } else {
      const float w = 1.0f - u - v;
      // 如果质心坐标在三角形边界上，则调整系数为0.5。
      if (abs(u) < BARY_EPS || abs(v) < BARY_EPS || abs(w) < BARY_EPS) {
        edge_coefficient = 0.5f;
      }

      // 返回t值和求交结果。
      t_value = t;
      return side * edge_coefficient;
    }
  }
}

// 在指定体素内，从前后左右上下三个方向对三角形求交点。
// 返回正方向（+x +y +z）和负方向（-x -y -z）上正面相交三角形数与反面相交三角形数的差值。
void calculate_triangle_intersection_with_3_rays(
  in Triangle tri,
  in int3 voxel_id,
  out float3 intersect_forward,
  out float3 intersect_backward
) {
  // 初始计数全为0。
  intersect_forward = float3(0.0f, 0.0f, 0.0f);
  intersect_backward = float3(0.0f, 0.0f, 0.0f);

  // 相交参数t。
  float t = 1e10f;
  // 归一化UVW空间中线段的开始点和结束点。
  float3 p, q;
  // 用于累计相交方向的计数变量。
  float intersect = 0;

  // 在UVW空间中，X方向上，以体素的中心生成线段的两个端点。
  p = (float3(voxel_id) + float3(0.0f, 0.5f, 0.5f)) / _max_dimension;
  q = (float3(voxel_id) + float3(1.0f, 0.5f, 0.5f)) / _max_dimension;
  // 线段从左到右，如果三角形面向右边，意味着左边是内（-），右边是外（+）。
  // 但此时线段从三角形背面穿过回返负值，因此这里对结果取反。
  intersect = -intersect_segment_to_triangle_with_face_check(p, q, tri, t);
  if (t < 0.5f) {
    // 如果t小于0.5，意味着交点靠近左侧，所以针对Backword累计符号计数。
    intersect_backward.x += float(intersect);
  } else {
    // 相反，意味着交点靠近右侧，则对Forward累计符号计数。
    intersect_forward.x += float(intersect);
  }

  // Y方向同X方向，只是轴不同。
  ...

  // Z方向同X方向，只是轴不同。
  ...
}
```

有了以上两个辅助函数后，就可以以2x2的体素为单位，对全部体素计算其相邻的体素是在三角形正面多还是反面多了。
分8次，分别从(0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 0), (1, 0, 1), (0, 1, 1), (1, 1, 1)位置遍历全部体素。
```hlsl
for (uint i = start_triangle_id; i < end_triangle_id && (i < _upper_bound_count - 1); i++) {
  // 通过上面的辅助函数，计算出体素[x, y, z]所包含的三角形，在“前”面正相交和反相交的差值，和在“后”面正相交和反相交的插值。
  calculate_triangle_intersection_with_3_rays(tri, int3(id.xyz), intersect_forward, intersect_backward);

  // 对于“前”面的情况累计到Ray Map的体素[x, y, z]中。
  _ray_map_rw[id.xyz] += float4(intersect_forward, 1.0f);

  // 如果不越界对于“后”面的情况，累计到Ray Map的相邻体素中。
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
*注意：如果线段和三角形没有相交，`intersect_segment_to_triangle_with_face_check`返回值是0，即使执行累加也没有影响，所以这里没有做任何判断。*

接下来就是分别从三个方向对这些值内加求和，这里只列出X方向的计算。
```hlsl
// 从正方向向负方向开始累加。
for (int t = _dimensions.x - 2; t >= 0; t--) {
  float count = _ray_map_rw[int3(t + 1, id.y, id.z)].x;
  _ray_map_rw[int3(t, id.y, id.z)] += float4(count, 0, 0, count != 0 ? 1 : 0);
}
```

至此，经过一系列计算后，已经可以通过Ray Map知道任意体素从右到左从上到下从后到前，一共经过多少个三角形正面和多少个三角形背面的差值了。
为接下来的符号判定准备好了数据。

通过下图对Ray Map的可视化可以看出，已经基本上能过够分辨模型内部区域和外部区域了。

![Image Ray Map](images/ray_map.png)

### 第五步：计算符号


To be continue...