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

1. `hala-gfx`负责Vulkan调用和封装。
2. `hala-renderer`负责从glTF文件中读取Mesh信息并上传到GPU。
3. `hala-imgui`是imGUI的Rust桥接，负责用户界面的显示和互动。

安装1.70+的Rust，如果已经安装`rustup update`为最新版本。使用`git clone --recursive`拉取仓库及其submodule。`cargo build`编译构建Debug版，或者`cargo build -r`构建Release版。

完成编译后可以直接`./target/（debug或release）/hala-sdf-baker -c conf/config.yaml -o ./out/output.txt`运行。点击“Bake”按钮进行烘焙，点击“Save”按钮可以把烘焙结果保存到"./out/output.txt"。

输出文件格式为：
```
X轴分辨率 Y轴分辨率 Z轴分辨率
1号体素的值
2号体素的值
。。。
n-1号体素的值
n号体素的值
```

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

准备全局的UBO，用于存储整个烘焙过程中都需要用到的一些参数，具体如下代码中的注释。
```rust
pub struct GlobalUniform {
  pub dimensions: [u32; 3],         // 根据需要烘焙Mesh的BoundingBox信息和烘焙体素最大分辨率计算出三个维度的大小。
  pub num_of_voxels: u32,           // 总体素的数量，其值为dimensions[0] * dimensions[1] * dimensions[2]。
  pub num_of_triangles: u32,        // 待烘焙Mesh的总三角形数量。
  pub initial_distance: f32,        // 初始化UDF的值。根据整个烘焙区域最长边的长度，归一化后的烘焙BoundingBox的对角线长度的1.01倍（整个UDF中不可能有值大于此值）。
  pub max_size: f32,                // 根据整个烘焙区域最长边的长度。
  pub max_dimension: u32,           // 整个体素空间最长边的体素数量。
  pub min_bounds_extended: [f32; 3],// 烘焙区域BoundingBox的最小坐标。
  pub max_bounds_extended: [f32; 3],// 烘焙区域BoundingBox的最大坐标。
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

To be continue...