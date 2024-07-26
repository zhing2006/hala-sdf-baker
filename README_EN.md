[![License](https://img.shields.io/badge/License-GPL3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)
[![MSRV](https://img.shields.io/badge/rustc-1.70.0+-ab6000.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

[English](README_EN.md) | [中文](README.md) | [日本語](README_JP.md) | [한국어](README_KO.md)

## Introduction

In modern computer graphics and game development, there is a technology widely regarded as indispensable: the use of Signed Distance Fields (SDF) and Unsigned Distance Fields (UDF). SDFs and UDFs provide an efficient and powerful means to represent and manipulate complex geometries. They play a crucial role in rendering, collision detection, model generation, and many other fields.

An SDF is a typical representation method that assigns a real number value to each point in space, indicating the signed distance from that point to the nearest surface. This structure can be used not only for efficient shape modeling but also for performing geometric operations such as smoothing, expanding, or shrinking shapes. In contrast, a UDF records the absolute shortest distance to the surface, which is particularly useful when dealing with models that have irregular or complex topologies.

SDFs and UDFs are not just data structures; they are methods of representing shapes in multi-dimensional space. In video game development, using SDFs for real-time shadow computation and ambient occlusion has become a popular technique. This is because SDFs can quickly determine the contact points between light rays and geometric surfaces, effectively generating soft shadows and other visual effects. Additionally, in real-time graphics, SDFs can be used for efficient geometric modeling and modifications, such as dynamic character deformation or common destruction effects in development. In the fields of industrial vision and scientific visualization, UDFs are often used for shape reconstruction and data fitting, especially when dealing with data from scanning devices or other measurement equipment. By constructing an accurate UDF, researchers can infer a continuous 3D surface from a set of discrete data points, which is critical for reconstructing complex biological forms or other scientific structures. This project aims to bake 3D mesh data into SDFs and UDFs using Rust and Vulkan.

![Image Intro](images/intro.png)

Figure 1: From https://arxiv.org/abs/2011.02570. The upper half shows a UDF, recording only the absolute shortest distance to the surface. The lower half shows an SDF, which records not only the shortest distance but also the sign indicating whether it is "inside" or "outside."

## Setting Up the Development Environment

Currently, the entire development environment has been tested only on the Windows platform using RTX 4090 and Radeon 780M (due to limited personal equipment, more compatibility tests are not possible at the moment). It is developed based on `hala-gfx`, `hala-renderer`, and `hala-imgui`.

* `hala-gfx` is responsible for Vulkan calls and encapsulation.
* `hala-renderer` is responsible for reading mesh information from glTF files and uploading it to the GPU.
* `hala-imgui` is the Rust bridge for imGUI, responsible for displaying and interacting with the user interface.

Install Rust 1.70+; if already installed, use `rustup update` to update to the latest version. Use `git clone --recursive` to pull the repository and its submodules. Use `cargo build` to compile and build the Debug version, or `cargo build -r` to build the Release version.

After compilation, you can run it directly.

    ./target/(debug or release)/hala-sdf-baker -c conf/config.yaml -o ./out/output.txt

Click the "Bake" button to bake, and click the "Save" button to save the baking results to "./out/output.txt".

The output file format is:

    X-axis resolution Y-axis resolution Z-axis resolution
    Value of the 1st voxel
    Value of the 2nd voxel
    ...
    Value of the (n-1)th voxel
    Value of the nth voxel

## UDF Baking

The implementation of UDF is relatively simple in terms of algorithms, so let's start with UDF baking.

### Step 1: Initialization

Before starting the baking process, resources need to be allocated. UDF uses voxel storage, which can be stored in a 3D form using Image or in a linear form using Buffer. For the convenience of subsequent visual debugging, we store it in a 3D form.

Before baking, some baking parameters need to be set, and their specific functions are explained in the comments in the code below.
```rust
pub selected_mesh_index: i32, // The index of the Mesh to be baked, as there may be multiple Mesh data in glTF.
pub max_resolution: i32,      // The resolution of the longest axis of the voxel output. For example, if the Mesh size is (1, 2, 4) and this field is 64, the final voxel resolution will be [16, 32, 64].
pub surface_offset: f32,      // This offset value will be added to the final baked data.
pub center: [f32; 3],         // The center position of the BoundingBox of the data to be baked.
pub desired_size: [f32; 3],   // The planned baking space size calculated based on the BoundingBox size of the Mesh, max_resolution, and the specified padding.
pub actual_size: [f32; 3],    // The size adjusted to be a multiple of the voxel size based on desired_size, which is also the final size of the saved data.
pub padding: [f32; 3],        // The number of voxels to expand outside the BoundingBox of the Mesh as a boundary.
```

The calculation methods for center and desired_size are as follows:
```rust
fn fit_box_to_bounds(&mut self) {
  // Get the BoundingBox of the Mesh to be baked.
  let bounds = self.get_selected_mesh_bounds().unwrap();

  // Calculate the longest edge length.
  let max_size = bounds.get_size().iter().fold(0.0, |a: f32, b| a.max(*b));
  // Calculate the size of a single voxel based on the specified maximum resolution.
  let voxel_size = max_size / self.settings.max_resolution as f32;
  // Calculate the size of the expanded boundary based on the voxel size.
  let padding = [
    self.settings.padding[0] * voxel_size,
    self.settings.padding[1] * voxel_size,
    self.settings.padding[2] * voxel_size,
  ];

  // Finally, obtain the center and size of the entire area to be baked.
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

The calculation method for actual_size is as follows:
```rust
fn snap_box_to_bounds(&mut self) {
  // Calculate the longest edge length of the area to be baked.
  let max_size = self.settings.desired_size.iter().fold(0.0, |a: f32, b| a.max(*b));
  // Determine the reference axis based on the longest edge, and the number of voxels along this axis will be the set maximum resolution value.
  let ref_axis = if max_size == self.settings.desired_size[0] {
    Axis::X
  } else if max_size == self.settings.desired_size[1] {
    Axis::Y
  } else {
    Axis::Z
  };

  // Depending on the reference axis, first calculate the size of a single voxel, and then calculate the size of the area to be baked as a multiple of the voxel size.
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

Next, prepare a global UBO to store some parameters needed throughout the baking process, as explained in the comments in the code below.
```rust
pub struct GlobalUniform {
  pub dimensions: [u32; 3],   // The size of the three dimensions calculated based on the BoundingBox information of the Mesh to be baked and the maximum resolution of the voxels.
  pub num_of_voxels: u32,     // The total number of voxels, which is dimensions[0] * dimensions[1] * dimensions[2].
  pub num_of_triangles: u32,  // The total number of triangles in the Mesh to be baked.
  pub initial_distance: f32,  // The initial value of the UDF. It is 1.01 times the length of the diagonal of the normalized BoundingBox of the baking area (no value in the entire UDF can be greater than this value).
  pub max_size: f32,          // The length of the longest edge of the entire baking area.
  pub max_dimension: u32,     // The number of voxels along the longest edge of the entire voxel space.
  pub center: [f32; 3],       // The center coordinates of the BoundingBox of the baking area.
  pub extents: [f32; 3],      // The half-length of the BoundingBox of the baking area.
}
```

Based on the number of voxels along the three axes of the voxel space calculated above, create an Image resource. Here, the Usage is set to Storage for writing in the Shader, and set to Sampled for reading.
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

### Step 2: Fill Initial Values

This step is the simplest. The only thing to note is that the initial distance written here is not in float form but in uint. This will be explained in detail in the next Shader.
```hlsl
_distance_texture_rw[int3(id.x, id.y, id.z)] = float_flip(_initial_distance);
```

Next, traverse all the triangles in the Mesh, where id.x is the index of the triangle being traversed.
```hlsl
Triangle tri_uvw;
tri_uvw.a = (get_vertex_pos(id.x, 0) - _center + _extents) / _max_size;
tri_uvw.b = (get_vertex_pos(id.x, 1) - _center + _extents) / _max_size;
tri_uvw.c = (get_vertex_pos(id.x, 2) - _center + _extents) / _max_size;
```
First, use the get_vertex_pos function to read the vertex position information from the Mesh's index buffer and vertex buffer.
Then, translate the vertices to the first quadrant in 3D space using the passed center and extents.
Finally, normalize to the uvw space in the range [0, 1] based on the value of max_size.

| Stage | Description |
|-------|-------------|
|![Image Bound 0](images/bound_0.png)| *Original Area* |
|![Image Bound 1](images/bound_1.png)| *Translated to First Quadrant* |
|![Image Bound 2](images/bound_2.png)| *Normalized to UVW Space* |

Next, calculate the AABB of the area covered by the triangle, transform it to voxel space using _max_dimension, and expand it by one voxel.
```hlsl
const float3 aabb_min = min(tri_uvw.a, min(tri_uvw.b, tri_uvw.c));
const float3 aabb_max = max(tri_uvw.a, max(tri_uvw.b, tri_uvw.c));
int3 voxel_min = int3(aabb_min * _max_dimension) - GRID_MARGIN;
int3 voxel_max = int3(aabb_max * _max_dimension) + GRID_MARGIN;
voxel_min = max(0, min(voxel_min, int3(_dimensions) - 1));
voxel_max = max(0, min(voxel_max, int3(_dimensions) - 1));
```

Finally, loop through all the voxels covered by the AABB, calculate the distance from the voxel center to the triangle, and write it into the Distance Texture.
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
Note that InterlockedMin atomic comparison is used here to write the minimum value because multiple GPU threads may be updating the same voxel simultaneously.
Additionally, float_flip is used to convert the float type distance to uint because InterlockedMin needs to operate on uint type data (not all hardware supports float InterlockedMin).
Let's take a closer look at the implementation of the float_flip function.
```hlsl
inline uint float_flip(float fl) {
  uint f = asuint(fl);
  return (f << 1) | (f >> 31);
}
```
This function moves the first bit of the float value, which is the sign bit, to the last position, so that when comparing with InterlockedMin, the absolute minimum value can be obtained, which conforms to the definition of UDF.

![Image IEEE 754](images/ieee_754.png)

As can be seen from the definition of the float type, as long as the sign bit is moved to the last bit, it can be compared like a uint.

After processing all the triangles, use the float_unflip function to move the sign bit back to its original position.

```hlsl
const int3 uvw = int3(id.x, id.y, id.z);
const uint distance = _distance_texture_rw[uvw];
_distance_texture_rw[uvw] = float_unflip(distance);
```

At this point, the Distance Texture records the distance to the nearest Mesh surface (unsigned) for the voxels covered by the triangles. However, the areas not covered by the triangles still have the initial value, which will be handled next.

### Step 3: Jump Flooding

Jump Flooding is an efficient algorithm used for computing Distance Transform and Voronoi Diagram, commonly applied in image processing and computational geometry. Unlike traditional pixel-by-pixel propagation methods, the Jump Flooding algorithm significantly speeds up the computation by "jumping" with exponentially increasing step sizes instead of propagating pixel by pixel.

#### Working Principle

The core idea of the Jump Flooding algorithm is to propagate distance information through a series of decreasing "jump" steps. Specifically, the algorithm starts from the initial seed points and updates multiple distance values simultaneously with larger step sizes, then gradually reduces the step size for finer updates. During each jump, the algorithm checks the neighbors of the current pixel and updates their distance values to ensure the propagation of the optimal solution.

First, the flooding algorithm requires two buffers to be used alternately. Here, the usage is set to TRANSFER_SRC to allow subsequent transfer from GPU to CPU, and then save it as a file.
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

It is worth mentioning that since the two buffers are used alternately, two DescriptorSets are pre-created and bound to the buffers in different orders for subsequent use.
```rust
// During odd-numbered jumps, read data from jump_buffer and write to jump_buffer_bis.
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

// During even-numbered jumps, read data from jump_buffer_bis and write to jump_buffer.
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

Next, initialize the jump flooding, with the initial seed considered as the optimal solution.
```hlsl
  const float distance = _distance_texture[int3(id.x, id.y, id.z)];
  const uint voxel_index = id3(id.x, id.y, id.z);
  _jump_buffer_rw[voxel_index] = voxel_index;
```

Calculate the total number of jumps needed by taking log2 of the maximum resolution. The offset for each step is halved from the previous step.
```rust
let num_of_steps = self.settings.max_resolution.ilog2();
for i in 1..=num_of_steps {
  let offset = (1 << (num_of_steps - i)) as u32;
  // Iterate through each step, flooding data from one buffer to another.
  ...
}
```

Jump sample from the current voxel to 26 surrounding directions, record the shortest distance to the Mesh surface (optimal solution), and update the jump buffer.
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
*Note: There is no check for x == 0 && y == 0 && z == 0 because if the current voxel is already the shortest distance, subsequent updates will not affect it.*

The specific jump sampling code is as follows:
```hlsl
void jump_sample(int3 center_coord, int3 offset, inout float best_distance, inout int best_index) {
  // Add offset to the current coordinate to get the sample coordinate.
  int3 sample_coord = center_coord + offset;
  // If the sample coordinate is out of the voxel range, do nothing.
  if (
    sample_coord.x < 0 || sample_coord.y < 0 || sample_coord.z < 0 ||
    sample_coord.x >= _dimensions.x || sample_coord.y >= _dimensions.y || sample_coord.z >= _dimensions.z
  ) {
    return;
  }
  // Get the seed index at the sample coordinate.
  uint voxel_sample_index = _jump_buffer[id3(sample_coord)];
  // Convert the index to x, y, z coordinates.
  int3 voxel_sample_coord = unpack_id3(voxel_sample_index);
  // Get the shortest distance from this coordinate to the Mesh surface.
  float voxel_sample_distance = _distance_texture[voxel_sample_coord];
  // The total distance is the distance from the current coordinate to the sample coordinate plus the shortest distance from the sample coordinate to the Mesh surface.
  // Note: Dividing by max_dimension is to convert to UVW space for calculation, as the Distance Texture stores distances in UVW space.
  float distance = length(float3(center_coord) / _max_dimension - float3(voxel_sample_coord) / _max_dimension) + voxel_sample_distance;
  // If the calculated jump distance is smaller than the previous ones, update the optimal solution.
  if (distance < best_distance) {
    best_distance = distance;
    best_index = voxel_sample_index;
  }
}
```

After repeating this algorithm for num_of_steps times, the optimal solution propagation for each voxel grid is completed. Here is an example in one-dimensional space: assuming the maximum resolution is 8, log2(8) = 3, three jumps are needed, with jump distances of 4, 2, and 1 respectively.

    First step:
    Voxel 0 calculates if 0->4 has an optimal solution
    Voxel 1 calculates if 1->5 has an optimal solution
    Voxel 2 calculates if 2->6 has an optimal solution
    Voxel 3 calculates if 3->7 has an optimal solution
    Voxel 4 calculates if 4->0 has an optimal solution
    Voxel 5 calculates if 5->1 has an optimal solution
    Voxel 6 calculates if 6->2 has an optimal solution
    Voxel 7 calculates if 7->3 has an optimal solution
    Second step:
    Voxel 0 calculates if 0->2 has an optimal solution
    Voxel 1 calculates if 1->3 has an optimal solution
    Voxel 2 calculates if 2->4, 2->0 has an optimal solution
    Voxel 3 calculates if 3->5, 3->1 has an optimal solution
    Voxel 4 calculates if 4->6, 4->2 has an optimal solution
    Voxel 5 calculates if 5->7, 5->3 has an optimal solution
    Voxel 6 calculates if 6->4 has an optimal solution
    Voxel 7 calculates if 7->5 has an optimal solution
    Third step:
    Voxel 0 calculates if 0->1 has an optimal solution
    Voxel 1 calculates if 1->2, 1->0 has an optimal solution
    Voxel 2 calculates if 2->3, 2->1 has an optimal solution
    Voxel 3 calculates if 3->4, 3->2 has an optimal solution
    Voxel 4 calculates if 4->5, 4->3 has an optimal solution
    Voxel 5 calculates if 5->6, 5->4 has an optimal solution
    Voxel 6 calculates if 6->7, 6->5 has an optimal solution
    Voxel 7 calculates if 7->6 has an optimal solution

Assuming voxel 4 is not covered by the triangle, the entire calculation process calculates 4->0, 4->2, 4->3, 4->5, 4->6. If voxel 1 is covered by the triangle, will voxel 4 not be calculated?
In the first step, 5->1 is calculated for the optimal solution, so the index of voxel 5 is updated to 1. In the third step, when calculating 4->5, it actually calculates if 4->1 has an optimal solution.

After completing the above steps, the Distance Texture needs to be updated.
```hlsl
// Current voxel coordinates.
const uint voxel_index = id3(id.x, id.y, id.z);

// Get the optimal voxel index from the Jump Buffer.
const uint cloest_voxel_index = _jump_buffer[voxel_index];
// Convert the index to coordinates.
const int3 cloest_voxel_coord = unpack_id3(cloest_voxel_index);
// Get the shortest distance to the Mesh stored in the optimal voxel coordinates.
const float cloest_voxel_distance = _distance_texture_rw[cloest_voxel_coord];

// Distance from the current voxel to the optimal voxel (UVW space, for the same reason as above).
const float distance_to_cloest_voxel = length(float3(id) / _max_dimension - float3(cloest_voxel_coord) / _max_dimension);

// The final distance is the distance from the current voxel to the optimal voxel plus the distance from the optimal voxel to the Mesh, plus the offset specified in the baking settings.
_distance_texture_rw[int3(id.x, id.y, id.z)] = cloest_voxel_distance + distance_to_cloest_voxel + g_push_constants.offset;
```
*Note: The Jump Flooding algorithm is a fast approximation method and does not guarantee that each voxel is updated to the shortest distance.*

At this point, the Distance Texture has saved the computed UDF data. Visualization can now be performed.

![Image UDF](images/udf.png)

From the image, it can be seen that the closer to the Mesh surface, the darker the color (smaller value, closer distance), and the farther away, the brighter the color (larger value, farther distance).

It is also possible to reconstruct the Mesh through isosurface.

![Image UDF Mesh](images/udf_mesh.png)


## SDF Baking

Compared to UDF, SDF baking is much more complex. The implementation here is referenced from Unity's [Visual Effect Graph](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@14.0/manual/sdf-in-vfx-graph.html).

### Step 1: Initialization

Add baking configuration items:
```rust
pub sign_passes_count: i32, // Number of iterations for the sign pass (determining whether the sign is positive or negative).
pub in_out_threshold: f32,  // Threshold for determining whether it is inside or outside the Mesh.
```

Next, prepare a global UBO to store some parameters needed throughout the baking process, as detailed in the comments in the following code.
```rust
pub struct GlobalUniform {
  pub dimensions: [u32; 3],   // Calculate the size of the three dimensions based on the BoundingBox information of the Mesh to be baked and the maximum resolution of the baking voxel.
  pub upper_bound_count: u32, // Upper bound of the Buffer containing triangles in each voxel.
  pub num_of_triangles: u32,  // Total number of triangles of the Mesh to be baked.
  pub max_size: f32,          // Length of the longest side of the entire baking area.
  pub max_dimension: u32,     // Number of voxels along the longest side of the voxel space.
  pub center: [f32; 3],       // Center coordinates of the BoundingBox of the baking area.
  pub extents: [f32; 3],      // Half-length of the BoundingBox of the baking area.
}
```
The calculation of other values is the same as UDF. Regarding upper_bound_count, since it is impossible to determine how many triangles each voxel contains, we can only estimate a maximum value here.
```rust
// First, assume that half of the voxels contain triangles.
let num_of_voxels_has_triangles = dimensions[0] as f64 * dimensions[1] as f64 * dimensions[2] as f64 / 2.0f64;
// Assume that a triangle is shared by 8 neighboring voxels. Assume that each voxel has a number of triangles equal to the square root of the total number of triangles.
// Take the maximum value of the above two assumptions.
let avg_triangles_per_voxel = (num_of_triangles as f64 / num_of_voxels_has_triangles * 8.0f64).max((num_of_triangles as f64).sqrt());
// Total number of triangles to be stored.
let upper_bound_count64 = (num_of_voxels_has_triangles * avg_triangles_per_voxel) as u64;
// Limit the maximum value to 1536 * 2^18.
let upper_bound_count = (1536 * (1 << 18)).min(upper_bound_count64) as u32;
// Limit the minimum value to 1024.
let upper_bound_count = upper_bound_count.max(1024);
```
*Note: This is just a conservative estimate, and the actual required number may be much smaller. The conservative estimate is made to cover more boundary cases.*

A large number of temporary Buffers are needed throughout the SDF baking process. To save space, they are not introduced here. Please refer to the source code files for details.

### Step 2: Construct Geometry

First, as with UDF, read the triangle information from the Mesh's Vertex Buffer and Index Buffer, transform it to the normalized UVW space, and save it to the Triangle UVW Buffer.
```hlsl
Triangle tri_uvw;
tri_uvw.a = (get_vertex_pos(id.x, 0) - _center + _extents) / _max_size;
tri_uvw.b = (get_vertex_pos(id.x, 1) - _center + _extents) / _max_size;
tri_uvw.c = (get_vertex_pos(id.x, 2) - _center + _extents) / _max_size;

_triangles_uvw_rw[id.x] = tri_uvw;
```

Next, calculate the "direction" of each triangle. The "direction" here indicates which axis the triangle is generally facing, i.e., which plane it is closer to among XY, ZX, and YZ. The result is saved to the Coord Flip Buffer.
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
The reason why it is the ZX plane instead of the XZ plane is that calculations need to be performed in three directions respectively. The ZX plane indicates that when calculating in the Y-axis direction, the local X-axis is actually Z, and the local Y-axis is actually X.

Since the direction has been assigned to each triangle, the next step is to perform conservative rasterization of the triangles in each direction.
Before that, calculate the orthogonal and projection matrices in the three directions.
```rust
// Construct the View matrix and Proj matrix based on the viewpoint position, rotation axis, width, height, near plane distance, and far plane distance.
let calculate_world_to_clip_matrix = |eye, rot, width: f32, height: f32, near: f32, far: f32| {
  let proj = glam::Mat4::orthographic_rh(-width / 2.0, width / 2.0, -height / 2.0, height / 2.0, near, far);
  let view = glam::Mat4::from_scale_rotation_translation(glam::Vec3::ONE, rot, eye).inverse();
  proj * view
};
```

The XY plane in the Z direction is shown below, with the local X-axis being the world's X-axis and the local Y-axis being the world's Y-axis.

![Image XY Plane](images/xy_plane.png)

```rust
let xy_plane_mtx = {
  // The viewpoint is looking down from a position 1 unit in the positive Z direction.
  let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(0.0, 0.0, bounds.extents[2] + 1.0);
  // The View space is looking down by default, no need to rotate.
  let rot = glam::Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0);
  // The near plane is at 1, which is why the viewpoint position is incremented by 1, leaving space for the near plane.
  let near = 1.0f32;
  // The far plane extends from the near plane to the length of the Z direction of the entire bounding box.
  let far = near + bounds.extents[2] * 2.0;
  calculate_world_to_clip_matrix(pos, rot, bounds.extents[0] * 2.0, bounds.extents[1] * 2.0, near, far)
};
```

The ZX plane in the Y direction is shown below, with the local X-axis being the world's Z-axis and the local Y-axis being the world's X-axis.

![Image ZX Plane](images/zx_plane.png)

```rust
let zx_plane_mtx = {
  // The viewpoint is looking outward from a position 1 unit in the positive Y direction (looking from the positive Y axis to the negative direction).
  let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(0.0, bounds.extents[1] + 1.0, 0.0);
  // First rotate -90 degrees along the Y-axis, then rotate -90 degrees along the X-axis. Align the local X-axis with the world's Z-axis, and the local Y-axis with the world's X-axis.
  let rot = glam::Quat::from_euler(glam::EulerRot::YXZ, -std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2, 0.0);
  let near = 1.0f32;
  let far = near + bounds.extents[1] * 2.0;
  calculate_world_to_clip_matrix(pos, rot, bounds.extents[2] * 2.0, bounds.extents[0] * 2.0, near, far)
};
```

The YZ plane in the X direction is shown below, with the local X-axis being the world's Y-axis and the local Y-axis being the world's Z-axis.

![Image YZ Plane](images/yz_plane.png)

```rust
let yz_plane_mtx = {
  // The viewpoint is looking left from a position 1 unit in the positive X direction.
  let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(bounds.extents[0] + 1.0, 0.0, 0.0);
  // First rotate 90 degrees along the X-axis, then rotate 90 degrees along the Y-axis. Align the local X-axis with the world's Y-axis, and the local Y-axis with the world's Z-axis.
  let rot = glam::Quat::from_euler(glam::EulerRot::XYZ, std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2, 0.0);
  let near = 1.0f32;
  let far = near + bounds.extents[0] * 2.0;
  calculate_world_to_clip_matrix(pos, rot, bounds.extents[1] * 2.0, bounds.extents[2] * 2.0, near, far)
};
```

Next, perform conservative rasterization of the triangles in the corresponding direction for each of the above directions.
First, calculate the 2D AABB of the triangle's coverage range and save it to a float4, where xy stores min and zw stores max.
```hlsl
// Get the three vertices of the triangle and transform them to clip space.
[unroll(3)]
for (i = 0; i < 3; i++) {
  vertex_in_clip[i] = mul(_world_to_clip[current_axis], float4(get_vertex_pos(id.x, i), 1.0));
}

// Calculate the size of the AABB.
float4 aabb = float4(1.0, 1.0, -1.0, -1.0);
aabb.xy = min(aabb.xy, min(vertex_in_clip[0].xy, min(vertex_in_clip[1].xy, vertex_in_clip[2].xy)));
aabb.zw = max(aabb.xy, max(vertex_in_clip[0].xy, max(vertex_in_clip[1].xy, vertex_in_clip[2].xy)));
float2 conservative_pixel_size;
// Calculate the actual Offset pixel size based on the current rasterization direction and the set Conservative Offset parameter.
if (current_axis == 0) {
  conservative_pixel_size = float2(_conservative_offset / _dimensions.x, _conservative_offset / _dimensions.y);
} else if (current_axis == 1) {
  conservative_pixel_size = float2(_conservative_offset / _dimensions.z, _conservative_offset / _dimensions.x);
} else {
  conservative_pixel_size = float2(_conservative_offset / _dimensions.y, _conservative_offset / _dimensions.z);
}

// Expand the size of the AABB.
_aabb_buffer_rw[id.x] = aabb + float4(-conservative_pixel_size.x, -conservative_pixel_size.y, conservative_pixel_size.x, conservative_pixel_size.y);
```

Then rasterize the triangle and expand the set Offset. The reason for the conservative rasterization expansion is to prevent "gaps" caused by float calculation errors.
```hlsl
// Construct the plane where the triangle is located and store it in float4, where xyz is the normal direction of the plane and w is the distance from the plane to the origin.
const float3 normal = normalize(cross(vertex_in_clip[1].xyz - vertex_in_clip[0].xyz, vertex_in_clip[2].xyz - vertex_in_clip[0].xyz));
const float4 triangle_plane = float4(normal, -dot(vertex_in_clip[0].xyz, normal));

// Calculate whether the normal direction is positive Z direction (1) or negative direction (-1).
const float direction = sign(dot(normal, float3(0, 0, 1)));
float3 edge_plane[3];
[unroll(3)]
for (i = 0; i < 3; i++) {
  // Calculate the 2D edge plane. W is the homogeneous coordinate.
  edge_plane[i] = cross(vertex_in_clip[i].xyw, vertex_in_clip[(i + 2) % 3].xyw);
  // Push the edge plane "outward" by a distance based on the previously determined direction and offset pixel value.
  // This is hard to understand, but you can look at the picture later.
  edge_plane[i].z -= direction * dot(conservative_pixel_size, abs(edge_plane[i].xy));
}

float4 conservative_vertex[3];
bool is_degenerate = false;
[unroll(3)]
for (i = 0; i < 3; i++) {
  _vertices_buffer_rw[3 * id.x + i] = float4(0, 0, 0, 1);

  // Intersect the edge planes of the three edges to get the new vertex positions.
  conservative_vertex[i].xyw = cross(edge_plane[i], edge_plane[(i + 1) % 3]);

  // Determine whether the triangle is degenerate based on the W value.
  if (abs(conservative_vertex[i].w) < CONSERVATIVE_RASTER_EPS) {
    is_degenerate |= true;
  } else {
    is_degenerate |= false;
    conservative_vertex[i] /= conservative_vertex[i].w; // after this, w is 1.
  }
}
if (is_degenerate)
  return;

// Calculate the Z values of the three vertices based on the points on the triangle that satisfy the plane equation.
// Plane equation: ax + by + cz + d = 0.
// Calculate Z: z = -(ax + by + d) / c.
// Finally, write the newly obtained three vertices into the Vertices Buffer.
[unroll(3)]
for (i = 0; i < 3; i++) {
  conservative_vertex[i].z = -(triangle_plane.x * conservative_vertex[i].x + triangle_plane.y * conservative_vertex[i].y + triangle_plane.w) / triangle_plane.z;
  _vertices_buffer_rw[3 * id.x + i] = conservative_vertex[i];
}
```
In computer graphics, a plane can be represented by a four-dimensional vector: float4(plane) = (a, b, c, d), where the plane equation is ax + by + cz + d = 0. The concept of an "edge plane" is based on the idea that when dealing with geometry in 2D projections (such as triangles), the boundaries of the triangle can be represented by planes that divide the space.

    edge_plane[i] = cross(vertex_in_clip[i].xyw, vertex_in_clip[(i + 2) % 3].xyw);

In this code, the specific method of constructing the edge plane is obtained by the cross product of the homogeneous coordinates of two vertices. Here, vertex_in_clip is the homogeneous coordinate of the vertex. vertex_in_clip[i].xyw extracts the x, y, w components of the vertex and treats it as a 3D vector. The cross function calculates the cross product of two 3D vectors, generating a vector perpendicular to the plane containing these two vectors. This generated vector edge_plane[i] represents the boundary plane from vertex_in_clip[i] to vertex_in_clip[(i + 2) % 3] (note that it is the 2D plane represented in homogeneous coordinates).

Here, the conservatively rasterized triangle is restored to model space. The red wireframe is the enlarged triangle, and the white wireframe is the original triangle. You can see that each triangle is enlarged along its plane.

![Image Conservative Offset](images/conservative_offset.png)

### Step 3: Triangle Coverage Voxel Count Statistics

Next, we will temporarily leave the Compute Shader and use the Vertex Shader and Fragment Shader to count the number of triangle coverages in three directions. Let's first look at the Vertex Shader.

```hlsl
struct VertexInput {
  // Pass in Vertex Id through Draw(num_of_triangles * 3).
  uint vertex_id: SV_VertexID;
};

ToFragment main(VertexInput input) {
  ToFragment output = (ToFragment)0;

  // Read the vertex data in Clip space directly from the Vertices Buffer of the previous rasterization result based on Vertex Id.
  const float4 pos = _vertices_buffer[input.vertex_id];
  // Divide the Vertex Id by 3 to get the triangle ID.
  output.triangle_id = input.vertex_id / 3;
  // If the current triangle is different from the current drawing direction, pass (-1, -1, -1, -1) to skip the Fragment Shader.
  if (_coord_flip_buffer[output.triangle_id] != g_push_constants.current_axis) {
    output.position = float4(-1, -1, -1, -1);
  } else {
    output.position = pos;
  }

  return output;
}
```

Now let's take a look at the overall process of the Fragment Shader.

```hlsl
struct ToFragment {
  float4 position: SV_Position;
  uint triangle_id: TEXCOORD0;
};

FragmentOutput main(ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

  // Calculate the voxel coordinates voxel_coord of the current processing pixel based on the position and triangle ID passed from the Vertex Shader.
  // Also determine whether to extend processing inward (backward) and outward (forward) in the depth direction.
  int3 depth_step, voxel_coord;
  bool can_step_backward, can_step_forward;
  get_voxel_coordinates(input.position, input.triangle_id, voxel_coord, depth_step, can_step_backward, can_step_forward);

  // Convert the voxel center coordinates to normalized UVW space and store them in the Voxels Buffer.
  float3 voxel_uvw = (float3(voxel_coord) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension;
  _voxels_buffer_rw[id3(voxel_coord)] = float4(voxel_uvw, 1.0f);
  // Accumulate in the Counter Buffer at the current voxel coordinates, marking this voxel as covered by a triangle once.
  InterlockedAdd(_counter_buffer_rw[id3(voxel_coord)], 1u);
  // If it can extend outward, perform the same operation on the outward voxel.
  if (can_step_forward) {
    _voxels_buffer_rw[id3(voxel_coord + depth_step)] = float4(voxel_uvw, 1.0f);
    InterlockedAdd(_counter_buffer_rw[id3(voxel_coord + depth_step)], 1u);
  }
  // If it can extend inward, perform the same operation on the inward voxel.
  if (can_step_backward) {
    _voxels_buffer_rw[id3(voxel_coord - depth_step)] = float4(voxel_uvw, 1.0f);
    InterlockedAdd(_counter_buffer_rw[id3(voxel_coord - depth_step)], 1u);
  }

  // The output of the RT here is not involved in the baking process, only used for debugging.
  output.color = float4(voxel_uvw, 1);
  return output;
}
```

The overall process is to use VS and FS to accumulate the Counter Buffer in the triangle coverage area. Now let's take a closer look at the implementation of `get_voxel_coordinates`.

```hlsl
void get_voxel_coordinates(
  float4 screen_position,
  uint triangle_id,
  out int3 voxel_coord,
  out int3 depth_step,
  out bool can_step_backward,
  out bool can_step_forward
) {
  // Get the current screen resolution, which is the voxel width and height in the current direction.
  // For example, if the current voxel space is [2, 3, 4], then when processing in the XY plane in the Z direction, it returns 2 x 3.
  const float2 screen_params = get_custom_screen_params();
  // Convert the Position passed from the Vertex Shader to UVW space.
  screen_to_uvw(screen_position, screen_params);
  // Get the AABB of the triangle coverage area calculated previously based on the triangle ID, and if it is not within the AABB range, discard the subsequent execution of the current Fragment Shader.
  cull_with_aabb(screen_position, triangle_id);
  // Calculate the voxel coordinates and determine whether to extend forward and backward.
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

Now let's take a closer look at the implementation of `compute_coord_and_depth_step`.

```hlsl
void compute_coord_and_depth_step(
  float2 screen_params,
  float4 screen_position,
  out int3 voxel_coord,
  out int3 depth_step,
  out bool can_step_backward,
  out bool can_step_forward
) {
  // Here we conservatively assume that the triangle will be shared by adjacent front and back voxels, which can avoid some display issues later.
  can_step_forward = true;
  can_step_backward = true;

  if (g_push_constants.current_axis == 1) {
    // Calculate the voxel coordinates through the Position in UVW space.
    voxel_coord = (screen_position.xyz * float3(screen_params, _dimensions[1]));
    voxel_coord.xyz = voxel_coord.yzx;

    // Determine if it is a boundary, if not, it can extend inward and outward.
    depth_step = int3(0, 1, 0);
    if (voxel_coord.y <= 0) {
      can_step_backward = false;
    }
    if (voxel_coord.y >= _dimensions[1] - 1) {
      can_step_forward = false;
    }
  } else if (g_push_constants.current_axis == 2) {
    // Same as above, but the specific axis direction is different.
  } else {
    // Same as above, but the specific axis direction is different.
  }
}
```

Since depth writing and depth testing are turned off, at this point, the voxels covered by triangles in the three directions have been counted in the Counter Buffer through InterlockedAdd. At the same time, the UVW coordinates of these covered voxels are also stored in the Voxels Buffer.

Next, we use the Prefix Sum algorithm to accumulate the Counter Buffer, and the final result is stored in the Accum Counter Buffer. The basic idea is to preprocess the array so that the sum of all elements before each position is stored, allowing subsequent query operations to be completed in constant time. Since the Prefix Sum algorithm is not directly related to baking itself, only the relevant algorithm introduction links are provided:
* [Wikipedia](https://en.wikipedia.org/wiki/Prefix_sum),
* [GPU Gems 3 - Chapter 39. Parallel Prefix Sum (Scan) with CUDA](https://developer.nvidia.com/gpugems/gpugems3/part-vi-gpu-computing/chapter-39-parallel-prefix-sum-scan-cuda)

At this point, the Accum Counter Buffer has saved the number of triangles covered by all previous voxels for the current voxel. For example, voxels 0, 1, 2, 3, 4 are covered by 4, 2, 5, 0, and 3 triangles, respectively. The values in the count Buffer at this time are:

    0 (no other voxels before the current voxel)
    4 (the previous voxel 0 has 4 triangles)
    6 (the previous voxels 0 and 1 have a total of 6 triangles)
    11 (same as above)
    11 (same as above)

Next, we store these triangles in the Triangle Id Buffer and can traverse the list of triangles contained in each voxel through the Accum Counter Buffer. Here we also use the Vertex Shader and Fragment Shader.

The Vertex Shader is the same as before, so it will not be repeated here. Let's look at the differences in the Fragment Shader.

```hlsl
// Here, the index for writing to the Triangle Ids Buffer is returned by adding 1 to the Counter Buffer through the calculated voxel coordinates.
uint index = 0u;
InterlockedAdd(_counter_buffer_rw[id3(voxel_coord)], 1u, index);
// To prevent out-of-bounds, the upper limit of the per-voxel triangle Buffer calculated at the beginning is used here.
if (index < _upper_bound_count)
_triangle_ids_buffer_rw[index] = input.triangle_id;
// Similarly, extend to the outward and inward voxels.
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

This is easy to understand. The number of triangles covered by all previous voxels for voxel i has been stored in the Counter Buffer, so `_counter_buffer_rw[id3(voxel_coord)]` takes the starting position where the current voxel i can write the triangle index.

### Step 4: Calculate Ray Map

After completing all the calculations above, you can traverse the list of triangles for a specified voxel using the following code.
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

Before starting to calculate the Ray Map, let's introduce a few helper functions.
```hlsl
// Calculate the intersection of a line segment with a triangle. Returns 0 if no intersection, 0.5 or -0.5 if intersecting the triangle edge, and 1.0 or -1.0 if intersecting the triangle interior.
// The sign indicates whether the intersection is with the front or back of the triangle. t_value returns the intersection parameter.
float intersect_segment_to_triangle_with_face_check(float3 segment_start, float3 segment_end, Triangle tri, out float t_value) {
  /*
   * Triangle plane equation: n * (P - A) = 0
   * Line segment equation: P(t) = Q + t(S - Q)
   * n dot ((Q + t(S - Q)) - A) = 0
   * n dot (Q - A + t(S - Q)) = 0
   * n dot (Q - A) + t(n dot (S - Q)) = 0
   * v = Q - A, d = S - Q
   * t = - (n dot v) / (n dot d)
   *
   * Where:
   * n - Normal vector of the triangle plane
   * P - Any point on the triangle plane
   * A - A vertex of the triangle
   * Q, S - Two endpoints of the line segment
   * t - Intersection parameter, describing the intersection point of the line segment and triangle
   * v - Vector Q - A
   * d - Vector S - Q
   */

  // Calculate two edges of the triangle.
  const float3 edge1 = tri.b - tri.a;
  const float3 edge2 = tri.c - tri.a;
  // Actually calculating -d = Q - S here.
  const float3 end_to_start = segment_start - segment_end;

  // Calculate the normal vector of the triangle plane using the cross product.
  const float3 normal = cross(edge1, edge2);
  // Calculate the dot product of the line segment direction and the triangle normal vector.
  const float dot_product = dot(end_to_start, normal);
  // The sign of this dot product result indicates whether the intersection is with the front or back of the triangle.
  const float side = sign(dot_product);
  // Take the reciprocal.
  const float inverse_dot_product = 1.0f / dot_product;

  // v = Q - A
  const float3 vertex0_to_start = segment_start - tri.a;
  // Calculate the intersection parameter t using the formula.
  // t = - (n dot v) / (n dot d)
  //   = (n dot v) / (n dot -d)
  float t = dot(vertex0_to_start, normal) * inverse_dot_product;

  // If t is less than 0 or greater than 1, it means the line segment and triangle plane do not intersect.
  if (t < -INTERSECT_EPS || t > 1 + INTERSECT_EPS) {
    t_value = 1e10f;
    return 0;
  } else {
    // Calculate the barycentric coordinates to check if the intersection point is inside the triangle.
    const float3 cross_product = cross(end_to_start, vertex0_to_start);
    const float u = dot(edge2, cross_product) * inverse_dot_product;
    const float v = -dot(edge1, cross_product) * inverse_dot_product;
    float edge_coefficient = 1.0f;

    // If the barycentric coordinates are not within the specified range, the intersection point is outside the triangle.
    if (u < -BARY_EPS || u > 1 + BARY_EPS || v < -BARY_EPS || u + v > 1 + BARY_EPS) {
      t_value = 1e10f;
      return 0;
    } else {
      const float w = 1.0f - u - v;
      // If the barycentric coordinates are on the triangle edge, adjust the coefficient to 0.5.
      if (abs(u) < BARY_EPS || abs(v) < BARY_EPS || abs(w) < BARY_EPS) {
        edge_coefficient = 0.5f;
      }

      // Return the t value and intersection result.
      t_value = t;
      return side * edge_coefficient;
    }
  }
}

// Calculate the intersection points of a triangle within a specified voxel from three directions: front, back, left, right, up, and down.
// Return the difference between the number of front-facing and back-facing triangle intersections in the positive (+x +y +z) and negative (-x -y -z) directions.
void calculate_triangle_intersection_with_3_rays(
  in Triangle tri,
  in int3 voxel_id,
  out float3 intersect_forward,
  out float3 intersect_backward
) {
  // Initialize all counts to 0.
  intersect_forward = float3(0.0f, 0.0f, 0.0f);
  intersect_backward = float3(0.0f, 0.0f, 0.0f);

  // Intersection parameter t.
  float t = 1e10f;
  // Normalize the start and end points of the line segment in UVW space.
  float3 p, q;
  // Variable to accumulate the intersection direction count.
  float intersect = 0;

  // In UVW space, for the X direction, generate the start and end points of the line segment at the center of the voxel.
  p = (float3(voxel_id) + float3(0.0f, 0.5f, 0.5f)) / _max_dimension;
  q = (float3(voxel_id) + float3(1.0f, 0.5f, 0.5f)) / _max_dimension;
  // The line segment goes from left to right. If the triangle faces right, it means the left side is inside (-) and the right side is outside (+).
  // But since the line segment intersects the back of the triangle, the result is negated here.
  intersect = -intersect_segment_to_triangle_with_face_check(p, q, tri, t);
  if (t < 0.5f) {
    // If t is less than 0.5, the intersection point is near the left side, so accumulate the sign count for the backward direction.
    intersect_backward.x += float(intersect);
  } else {
    // Conversely, if the intersection point is near the right side, accumulate the sign count for the forward direction.
    intersect_forward.x += float(intersect);
  }

  // The Y direction is similar to the X direction, just with a different axis.
  ...

  // The Z direction is similar to the X direction, just with a different axis.
  ...
}
```

With the above two helper functions, you can calculate whether the adjacent voxels of all voxels contain more front-facing or back-facing triangles in units of 2x2 voxels.
This is done in 8 passes, starting from positions (0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 0), (1, 0, 1), (0, 1, 1), and (1, 1, 1) to traverse all voxels.
```hlsl
for (uint i = start_triangle_id; i < end_triangle_id && (i < _upper_bound_count - 1); i++) {
  // Using the helper functions above, calculate the difference between the number of front-facing and back-facing intersections for the triangles contained in voxel [x, y, z] in the "forward" and "backward" directions.
  calculate_triangle_intersection_with_3_rays(tri, int3(id.xyz), intersect_forward, intersect_backward);

  // Accumulate the results for the "forward" direction in the voxel [x, y, z] in the Ray Map.
  _ray_map_rw[id.xyz] += float4(intersect_forward, 1.0f);

  // If not out of bounds, accumulate the results for the "backward" direction in the adjacent voxel in the Ray Map.
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
*Note: If the line segment and triangle do not intersect, the return value of `intersect_segment_to_triangle_with_face_check` is 0. Even if accumulated, it has no effect, so no additional checks are performed here.*

Next, sum these values from three directions. Only the calculation for the X direction is shown here.
```hlsl
// Accumulate from the positive direction to the negative direction.
for (int t = _dimensions.x - 2; t >= 0; t--) {
  float count = _ray_map_rw[int3(t + 1, id.y, id.z)].x;
  _ray_map_rw[int3(t, id.y, id.z)] += float4(count, 0, 0, count != 0 ? 1 : 0);
}
```

After a series of calculations, you can now use the Ray Map to know the difference between the number of front-facing and back-facing triangles for any voxel from right to left, top to bottom, and back to front.
This prepares the data for the subsequent sign determination.

As shown in the visualization of the Ray Map below, you can already distinguish the internal and external regions of the model.

![Image Ray Map](images/ray_map.png)

### Step 5: Calculating Signs

First, initialize the Sign Map.
```hlsl
const float4 self_ray_map = _ray_map[id.xyz];
// The Ray Map corresponding to the current voxel stores the difference between positive and negative intersections from right to left, back to front, and top to bottom to the current voxel.
const float right_side_intersection = self_ray_map.x;
const float back_side_intersection = self_ray_map.y;
const float top_side_intersection = self_ray_map.z;
// The first plane voxel in each direction stores the difference between positive and negative intersections through the entire voxel from right to left, back to front, and top to bottom.
// Subtracting the value of the current voxel gives the remaining difference in positive and negative intersections on its left, front, and bottom sides.
const float left_side_intersection = _ray_map[int3(0, id.y, id.z)].x - self_ray_map.x;
const float front_side_intersection = _ray_map[int3(id.x, 0, id.z)].y - self_ray_map.y;
const float bottom_side_intersection = _ray_map[int3(id.x, id.y, 0)].z - self_ray_map.z;
// Adding them all together roughly indicates whether there are more positive or negative intersections on the current voxel, implying whether it is "inside" or "outside."
_sign_map_rw[id.xyz] =
  right_side_intersection - left_side_intersection +
  back_side_intersection - front_side_intersection +
  top_side_intersection - bottom_side_intersection;
```

At this point, only the intersection effects along the axis directions around each voxel are considered, which is not accurate enough. Therefore, perform n iterations, each time randomly selecting 8 neighboring voxels and calculating the sign accumulation using 6 different paths to improve accuracy.
Here, the initial value of normalize_factor is 6 because it will be accumulated through 6 different paths in the following steps.
It keeps doubling because each iteration accumulates on the previous one, and normalization is only performed in the last iteration.
```rust
let num_of_neighbors = 8u32;
let mut normalize_factor = 6.0f32;
for i in 1..=self.settings.sign_passes_count {
  // Dispatch Compute Shader.
  ...
  normalize_factor += num_of_neighbors as f32 * 6.0 * normalize_factor;
}
```
The Compute Shader is as follows:
```hlsl
const float4 self_ray_map = _ray_map[id.xyz];
// Loop to take 8 neighboring voxels.
for (uint i = 0; i < g_push_constants.num_of_neighbors; i++) {
  int3 neighbors_offset = generate_random_neighbor_offset((i * g_push_constants.num_of_neighbors) + g_push_constants.pass_id, _max_dimension * 0.05f);
  int3 neighbors_index;
  neighbors_index.x = min((int)(_dimensions.x - 1), max(0, (int)id.x + neighbors_offset.x));
  neighbors_index.y = min((int)(_dimensions.y - 1), max(0, (int)id.y + neighbors_offset.y));
  neighbors_index.z = min((int)(_dimensions.z - 1), max(0, (int)id.z + neighbors_offset.z));

  // Calculate the sign accumulation value for 6 different paths to the neighboring voxel.
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

// Normalize at the end of the last iteration.
if (g_push_constants.need_normalize) {
  const float normalize_factor_final = g_push_constants.normalize_factor + g_push_constants.num_of_neighbors * 6 * g_push_constants.normalize_factor;
  _sign_map_rw[id.xyz] /= normalize_factor_final;
}
```

At this point, if the Sign Map is visualized, the internal and external regions of the model can be clearly distinguished.

![Image Sign Map](images/sign_map.png)

### Step 6: Closing the Surface

First, because the mesh may not be completely closed, find the boundary of internal and external "holes," i.e., the places in the Sign Map where positive and negative values are adjacent near the specified threshold.
```hlsl
// Calculate the score based on the set threshold.
const float self_sign_score = _sign_map[id.xyz] - g_push_constants.threshold;
// If the score is less than 10% of the threshold.
if (abs(self_sign_score / g_push_constants.threshold) < 0.1f) {
  // If the score of the current voxel and the score of the voxel on the right are opposite, it means the boundary is found.
  if (self_sign_score * (_sign_map[id.xyz + uint3(1, 0, 0)] - g_push_constants.threshold) < 0) {
    // Determine whether to write to itself or the voxel on the right based on the score of the current voxel.
    const uint3 write_coord = id.xyz + (self_sign_score < 0 ? uint3(1, 0, 0) : uint3(0, 0, 0));
    // Write the normalized value of the voxel coordinates to the UVW space.
    _voxels_texture_rw[write_coord.xyz] = float4((float3(write_coord) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension, 1.0f);
  }
  // Similarly handle the Y-axis.
  ...
  // Similarly handle the Z-axis.
  ...
}
```

Next, write the voxel information (covered by triangles) from the previously processed Voxels Buffer into the Voxels Texture.
```hlsl
const float4 voxel = _voxels_buffer[id3(id.xyz)];
if (voxel.w != 0.0f)
  _voxels_texture_rw[id.xyz] = voxel;
```

At this point, all boundary voxels and voxels covered by triangles are stored in the Voxels Texture, ready for jump flooding.
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
The jump flooding propagation here is similar to UDF, so it will not be explained in detail.
The only difference is that an initialization jump with an offset of 1 is performed before the formal jump, which can improve some details near the surface in advance, as jump flooding is an approximate algorithm.
At this point, the entire Voxels Texture has saved the coordinates of the nearest distance from each voxel to the mesh surface.

### Step 7: Calculating the Final Signed Distance Field

```hlsl
// Get the seed coordinates saved in the Voxels Texture and the normalized UVW coordinates of the current voxel.
const float3 seed_coord = _voxels_texture[int3(id.x, id.y, id.z)].xyz;
const float3 voxel_coord = (float3(id.xyz) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension;

// Determine the sign of the current voxel based on the specified threshold.
float sign_d = _sign_map[id.xyz] > g_push_constants.threshold ? -1 : 1;

// Calculate the index coordinates of the voxel based on the seed coordinates.
const int3 id_seed = int3(seed_coord * _max_dimension);

// Get the Start and End positions of the triangle list of the seed voxel.
uint start_triangle_id = 0;
[branch]
if(id3(id_seed) > 0) {
  start_triangle_id = _accum_counters_buffer[id3(id_seed) - 1];
}
uint end_triangle_id = _accum_counters_buffer[id3(id_seed)];

// Traverse all triangles to get the shortest distance from the current voxel to the triangles covered by the seed voxel.
float distance = 1e6f;
for (uint i = start_triangle_id; (i < end_triangle_id) && (i < _upper_bound_count - 1); i++) {
  const uint triangle_index = _triangles_in_voxels[i];
  Triangle tri = _triangles_uvw[triangle_index];
  distance = min(distance, point_distance_to_triangle(voxel_coord, tri));
}
// Special case, if the seed voxel has no triangles, the distance is directly the UVW coordinate distance from the current voxel to the seed voxel.
if (1e6f - distance < COMMON_EPS) {
  distance = length(seed_coord - voxel_coord);
}
// Apply the sign and offset to get the signed distance.
distance = sign_d * distance - g_push_constants.offset;

// Save the signed distance to the Image and Buffer. The Image can be used for rendering, and the Buffer can be used for export.
_voxels_buffer_rw[id3(id)] = float4(distance, distance, distance, distance);
_distance_texture_rw[id] = distance;
```

At this point, all baking calculations are completed, and the SDF data is obtained. Below are the visualization effects of SDF using Ray Marching with normal direction as color.

![Image SDF Normal 0](images/sdf_normal_0.png)

![Image SDF Normal 1](images/sdf_normal_1.png)
