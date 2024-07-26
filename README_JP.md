# hala-sdf-baker
[![License](https://img.shields.io/badge/License-GPL3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)
[![MSRV](https://img.shields.io/badge/rustc-1.70.0+-ab6000.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

[English](README_EN.md) | [中文](README.md) | [日本語](README_JP.md) | [한국어](README_KO.md)

## 引言

現代のコンピュータグラフィックスとゲーム開発において、広く不可欠とされる技術があります。それは、有向距離場（Signed Distance Fields, SDF）および無向距離場（Unsigned Distance Fields, UDF）の使用です。SDFとUDFは、複雑な幾何形状を効率的かつ強力に表現し操作する手段を提供します。これらは、レンダリング、衝突検出、モデル生成などの多くの分野で重要な役割を果たしています。

SDFは典型的な表現方法であり、空間内の各点に実数値を割り当て、その点から最も近い表面までの有向距離を示します。この構造は、形状モデリングを効率的に行うだけでなく、平滑化、膨張、縮小などの幾何操作を実行するためにも使用できます。対照的に、UDFは表面までの絶対最短距離を記録し、不規則または複雑なトポロジーを持つモデルを処理する際に特に有用です。

SDFとUDFは単なるデータ構造ではなく、多次元空間で形状を表現する方法でもあります。ビデオゲーム開発において、SDFを利用したリアルタイムシャドウ計算や環境光遮蔽は一般的な技術となっています。これは、SDFが光線と幾何表面の接触点を迅速に特定し、ソフトシャドウや他の視覚効果を効率的に生成できるためです。また、リアルタイムグラフィックスにおいて、SDFを使用することで効率的な幾何モデリングや修正が可能となり、キャラクターの動的変形や破壊効果などの開発に役立ちます。産業ビジョンや科学的可視化の分野では、UDFは形状再構築やデータフィッティングに頻繁に使用されます。特に、スキャンデバイスや他の計測デバイスからのデータを処理する際に有用です。正確なUDFを構築することで、研究者は離散データポイントの集合から連続的な三次元表面を推定でき、複雑な生物形態や他の科学構造の再構築において重要です。本プロジェクトでは、RustとVulkanを使用して三次元メッシュデータをSDFおよびUDFにベイクすることを実現します。

![Image Intro](images/intro.png)

図1：https://arxiv.org/abs/2011.02570 より。上半分はUDFで、表面までの絶対最短距離のみを記録しています。下半分はSDFで、最短距離に加え正負の符号で「内側」か「外側」かを示しています。

## 開発環境のセットアップ

現在、開発環境はWindowsプラットフォーム上でRTX 4090およびRadeon 780Mを使用してテストされています（私のデバイスが限られているため、他の互換性についてはテストできていません）。`hala-gfx`、`hala-renderer`、および`hala-imgui`に基づいて開発されています。

* `hala-gfx`はVulkanの呼び出しとラッピングを担当します。
* `hala-renderer`はglTFファイルからメッシュ情報を読み取り、GPUにアップロードします。
* `hala-imgui`はimGUIのRustブリッジで、ユーザーインターフェースの表示とインタラクションを担当します。

Rust 1.70+をインストールします。すでにインストールされている場合は、`rustup update`で最新バージョンに更新します。`git clone --recursive`を使用してリポジトリおよびサブモジュールをクローンします。`cargo build`でデバッグ版をビルドするか、`cargo build -r`でリリース版をビルドします。

ビルドが完了したら、以下のコマンドで直接実行できます。

    ./target/（debugまたはrelease）/hala-sdf-baker -c conf/config.yaml -o ./out/output.txt

「Bake」ボタンをクリックしてベイクを実行し、「Save」ボタンをクリックしてベイク結果を"./out/output.txt"に保存できます。

出力ファイルのフォーマットは以下の通りです：

    X軸解像度 Y軸解像度 Z軸解像度
    1番目のボクセルの値
    2番目のボクセルの値
    。。。
    n-1番目のボクセルの値
    n番目のボクセルの値

## UDFベイキング

アルゴリズムの実装において、UDFは比較的簡単です。ここではまずUDFベイキングについて説明します。

### 第一步：初期化

ベイキングを開始する前に、リソースを割り当てる必要があります。UDFはボクセルストレージであり、Imageを3D形式で保存するか、Bufferを線形形式で保存するかを選択できます。ここでは、後続の可視化デバッグを容易にするために、3D形式で保存します。

ベイキング前にいくつかのベイキングパラメータを設定する必要があります。その具体的な役割は以下のコードのコメントに示されています。
```rust
pub selected_mesh_index: i32, // glTFには複数のMeshデータが保存されている可能性があり、このフィールドはどのMeshをベイキングするかを決定します。
pub max_resolution: i32,      // ベイキング出力のボクセルの最長軸の解像度。例えばサイズが(1, 2, 4)のMesh範囲の場合、このフィールドが64であれば、最終的なボクセルの解像度は[16, 32, 64]になります。
pub surface_offset: f32,      // このオフセット値は最終的にベイキングされたデータに加算されます。
pub center: [f32; 3],         // ベイキング対象データのBoundingBoxの中心位置。
pub desired_size: [f32; 3],   // MeshのBoundingBoxのサイズ、max_resolution、および指定された余白サイズpaddingに基づいて計算された計画ベイキングスペースのサイズ。
pub actual_size: [f32; 3],    // desired_sizeをボクセルサイズの整数倍に調整したサイズで、最終的にデータを保存するサイズでもあります。
pub padding: [f32; 3],        // MeshのBoundingBox外にどれだけのボクセルを拡張するかを示す境界。
```

centerとdesired_sizeの計算方法は以下の通りです：
```rust
fn fit_box_to_bounds(&mut self) {
  // ベイキング対象MeshのBoundingBoxを取得します。
  let bounds = self.get_selected_mesh_bounds().unwrap();

  // 最長辺の長さを計算します。
  let max_size = bounds.get_size().iter().fold(0.0, |a: f32, b| a.max(*b));
  // 指定された最大解像度に基づいて、単一ボクセルのサイズを計算します。
  let voxel_size = max_size / self.settings.max_resolution as f32;
  // ボクセルサイズに基づいて拡張境界のサイズを計算します。
  let padding = [
    self.settings.padding[0] * voxel_size,
    self.settings.padding[1] * voxel_size,
    self.settings.padding[2] * voxel_size,
  ];

  // 最終的にベイキング領域全体の中心とサイズを取得します。
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

actual_sizeの計算方法は以下の通りです：
```rust
fn snap_box_to_bounds(&mut self) {
  // ベイキング領域の最長辺の長さを計算します。
  let max_size = self.settings.desired_size.iter().fold(0.0, |a: f32, b| a.max(*b));
  // 最長辺の軸を基準軸として確定し、この軸のボクセル数は設定された最大解像度値になります。
  let ref_axis = if max_size == self.settings.desired_size[0] {
    Axis::X
  } else if max_size == self.settings.desired_size[1] {
    Axis::Y
  } else {
    Axis::Z
  };

  // 基準軸に応じて、まず単一ボクセルのサイズを計算し、次にボクセルサイズの整数倍のベイキング領域のサイズを計算します。
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

次に、ベイキング全体で使用するいくつかのパラメータを保存するためのグローバルUBOを準備します。具体的には以下のコードのコメントに示されています。
```rust
pub struct GlobalUniform {
  pub dimensions: [u32; 3],   // ベイキング対象MeshのBoundingBox情報とベイキングボクセルの最大解像度に基づいて計算された3つの次元のサイズ。
  pub num_of_voxels: u32,     // 総ボクセル数。値はdimensions[0] * dimensions[1] * dimensions[2]です。
  pub num_of_triangles: u32,  // ベイキング対象Meshの総三角形数。
  pub initial_distance: f32,  // UDFの初期値。ベイキング領域の最長辺の長さに基づき、正規化されたベイキングBoundingBoxの対角線の長さの1.01倍（UDF全体でこの値を超えることはありません）。
  pub max_size: f32,          // ベイキング領域の最長辺の長さに基づきます。
  pub max_dimension: u32,     // ボクセル空間の最長辺のボクセル数。
  pub center: [f32; 3],       // ベイキング領域BoundingBoxの中心座標。
  pub extents: [f32; 3],      // ベイキング領域BoundingBoxの半長。
}
```

上記の計算に基づいて、ボクセル空間の3つの軸方向のボクセル数を設定し、Imageリソースを作成します。ここでは、Shaderでの書き込みのためにUsageをStorageに設定し、読み取りのためにSampledに設定します。
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

### 第二步：初期値の入力

このステップは最も簡単です。唯一注意すべき点は、ここで書き込むのは初期距離のfloat形式ではなく、uint形式であることです。これは次のShaderで詳しく説明します。
```hlsl
_distance_texture_rw[int3(id.x, id.y, id.z)] = float_flip(_initial_distance);
```

次に、Mesh内のすべての三角形を遍歴します。id.xは現在遍歴している三角形のインデックス番号です。
```hlsl
Triangle tri_uvw;
tri_uvw.a = (get_vertex_pos(id.x, 0) - _center + _extents) / _max_size;
tri_uvw.b = (get_vertex_pos(id.x, 1) - _center + _extents) / _max_size;
tri_uvw.c = (get_vertex_pos(id.x, 2) - _center + _extents) / _max_size;
```
まず、get_vertex_pos関数を使用してMeshのindex bufferとvertex bufferから頂点の位置情報を読み取ります。
次に、渡されたcenterとextentsを使用して頂点を三次元空間の第一象限に平行移動させます。
最後に、max_sizeの値に基づいて[0, 1]範囲のuvw空間に正規化します。

| ステージ | 説明 |
|------|------|
|![Image Bound 0](images/bound_0.png)| *原始領域* |
|![Image Bound 1](images/bound_1.png)| *第一象限に平行移動* |
|![Image Bound 2](images/bound_2.png)| *UVW空間に正規化* |

次に、三角形がカバーする領域のAABBを計算し、_max_dimensionを使用してボクセル空間に変換し、一周拡張します。
```hlsl
const float3 aabb_min = min(tri_uvw.a, min(tri_uvw.b, tri_uvw.c));
const float3 aabb_max = max(tri_uvw.a, max(tri_uvw.b, tri_uvw.c));
int3 voxel_min = int3(aabb_min * _max_dimension) - GRID_MARGIN;
int3 voxel_max = int3(aabb_max * _max_dimension) + GRID_MARGIN;
voxel_min = max(0, min(voxel_min, int3(_dimensions) - 1));
voxel_max = max(0, min(voxel_max, int3(_dimensions) - 1));
```

最後に、AABBがカバーするすべてのボクセルをループして遍歴し、ボクセル中心から三角形までの距離を計算し、Distance Textureに書き込みます。
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
注意点として、ここではInterlockedMin原子比較書き込み最小値関数を使用しています。これは、複数のGPUスレッドが同時に同じボクセルを更新する可能性があるためです。
また、float_flipを使用してfloat型の距離をuintに変換しています。これは、InterlockedMinがuint型データを操作する必要があるためです（すべてのハードウェアがfloatのInterlockedMinをサポートしているわけではありません）。
ここで、float_flip関数の実装を詳しく見てみましょう。
```hlsl
inline uint float_flip(float fl) {
  uint f = asuint(fl);
  return (f << 1) | (f >> 31);
}
```
この関数は、float数値の最初のビット、つまり符号ビットを最後に移動させます。これにより、InterlockedMinで比較するときに絶対値が最小の値を取得でき、UDFの定義に適合します。

![Image IEEE 754](images/ieee_754.png)

float型の定義からわかるように、符号ビットを最後のビットに移動させるだけで、uintと同じように比較できます。

すべての三角形の処理が完了した後、float_unflip関数を使用して符号ビットを元の位置に戻します。

```hlsl
const int3 uvw = int3(id.x, id.y, id.z);
const uint distance = _distance_texture_rw[uvw];
_distance_texture_rw[uvw] = float_unflip(distance);
```

これでDistance Texture中の三角形がカバーするボクセルは、Mesh表面までの最短距離（無符号）を記録しています。しかし、三角形がカバーしていない領域はまだ初期値のままです。次にこれらの領域を処理します。

### ステップ3：ジャンプフラッディング

ジャンプフラッディング（Jump Flooding）は、距離変換（Distance Transform）やボロノイ図（Voronoi Diagram）を計算するための効率的なアルゴリズムであり、画像処理や計算幾何学の分野でよく使用されます。従来のピクセルごとの伝播方法とは異なり、ジャンプフラッディングアルゴリズムは指数関数的に増加するステップサイズで「ジャンプ」することで、計算速度を大幅に向上させます。

#### 動作原理

ジャンプフラッディングアルゴリズムの核心となるアイデアは、一連の減少する「ジャンプ」ステップを通じて距離情報を伝播させることです。具体的には、アルゴリズムは初期の種点から始まり、大きなステップサイズで複数の距離値を同時に更新し、その後ステップサイズを徐々に小さくしてより詳細な更新を行います。各ジャンプの過程で、アルゴリズムは現在のピクセルの近隣をチェックし、その距離値を更新して最適解の伝播を確保します。

まず、フラッディングアルゴリズムには2つのバッファを交互に使用する必要があります。ここでは、UsageをTRANSFER_SRCに設定することで、後でGPUからCPUにデータを転送し、ファイルとして保存できるようにします。
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

注目すべき点は、2つのバッファを交互に使用するため、事前に2つのDescriptorSetを作成し、異なる順序でバッファをバインドして後で使用しやすくすることです。
```rust
// 奇数ステップのジャンプ時に、jump_bufferからデータを読み取り、jump_buffer_bisに書き込みます。
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

// 偶数ステップのジャンプ時に、jump_buffer_bisからデータを読み取り、jump_bufferに書き込みます。
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

次に、ジャンプフラッディングの初期化を行います。初期の種は自分自身が最適解であると見なします。
```hlsl
  const float distance = _distance_texture[int3(id.x, id.y, id.z)];
  const uint voxel_index = id3(id.x, id.y, id.z);
  _jump_buffer_rw[voxel_index] = voxel_index;
```

最大解像度に対してlog2を求め、必要なジャンプのステップ数を計算します。各ステップの開始オフセットは前のステップの半分になります。
```rust
let num_of_steps = self.settings.max_resolution.ilog2();
for i in 1..=num_of_steps {
  let offset = (1 << (num_of_steps - i)) as u32;
  // ループを繰り返し、各ステップで一方のバッファからデータをもう一方のバッファにフラッディングします。
  ...
}
```

現在のボクセルから周囲の26方向にジャンプしてサンプリングし、メッシュ表面までの最短距離（最適解）を記録してジャンプバッファを更新します。
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
*ここではx == 0 && y == 0 && z == 0の判断をしていないことに注意してください。現在のボクセルがすでに最短距離であれば、後続の更新には影響しません。*

具体的なジャンプサンプリングコードは以下の通りです：
```hlsl
void jump_sample(int3 center_coord, int3 offset, inout float best_distance, inout int best_index) {
  // 現在の座標にオフセットを加えてサンプリング座標を取得します。
  int3 sample_coord = center_coord + offset;
  // サンプリング座標がボクセルの範囲を超えている場合は何もしません。
  if (
    sample_coord.x < 0 || sample_coord.y < 0 || sample_coord.z < 0 ||
    sample_coord.x >= _dimensions.x || sample_coord.y >= _dimensions.y || sample_coord.z >= _dimensions.z
  ) {
    return;
  }
  // サンプリング座標の種インデックスを取得します。
  uint voxel_sample_index = _jump_buffer[id3(sample_coord)];
  // インデックスをx, y, zの座標形式に変換します。
  int3 voxel_sample_coord = unpack_id3(voxel_sample_index);
  // この座標からメッシュ表面までの最短距離を取得します。
  float voxel_sample_distance = _distance_texture[voxel_sample_coord];
  // 総距離は現在の座標からサンプリング座標までの距離に、サンプリング座標からメッシュ表面までの最短距離を加えたものです。
  // 注：ここでmax_dimensionで割るのは、UVW空間で計算するためです。Distance TextureにはUVW空間の距離が保存されています。
  float distance = length(float3(center_coord) / _max_dimension - float3(voxel_sample_coord) / _max_dimension) + voxel_sample_distance;
  // 以上の計算で得られたジャンプ距離が以前のものより小さい場合、最適解を更新します。
  if (distance < best_distance) {
    best_distance = distance;
    best_index = voxel_sample_index;
  }
}
```

このアルゴリズムをnum_of_steps回繰り返すと、各ボクセルグリッドが最適解の伝播を完了します。ここでは1次元空間を例にとり、最大解像度が8であると仮定します。log2(8)=3であり、3ステップのジャンプが必要です。各ジャンプの距離はそれぞれ4, 2, 1です。

    第1ステップ：
    ボクセル0 0->4の最適解を計算
    ボクセル1 1->5の最適解を計算
    ボクセル2 2->6の最適解を計算
    ボクセル3 3->7の最適解を計算
    ボクセル4 4->0の最適解を計算
    ボクセル5 5->1の最適解を計算
    ボクセル6 6->2の最適解を計算
    ボクセル7 7->3の最適解を計算
    第2ステップ：
    ボクセル0 0->2の最適解を計算
    ボクセル1 1->3の最適解を計算
    ボクセル2 2->4, 2->0の最適解を計算
    ボクセル3 3->5, 3->1の最適解を計算
    ボクセル4 4->6, 4->2の最適解を計算
    ボクセル5 5->7, 5->3の最適解を計算
    ボクセル6 6->4の最適解を計算
    ボクセル7 7->5の最適解を計算
    第3ステップ：
    ボクセル0 0->1の最適解を計算
    ボクセル1 1->2, 1->0の最適解を計算
    ボクセル2 2->3, 2->1の最適解を計算
    ボクセル3 3->4, 3->2の最適解を計算
    ボクセル4 4->5, 4->3の最適解を計算
    ボクセル5 5->6, 5->4の最適解を計算
    ボクセル6 6->7, 6->5の最適解を計算
    ボクセル7 7->6の最適解を計算

ここでは、4が三角形で覆われていないボクセルであると仮定します。計算プロセス全体で4->0, 4->2, 4->3, 4->5, 4->6が計算されました。では、1が三角形で覆われているボクセルであると仮定した場合、4は計算されないのでしょうか？
第1ステップで5->1の最適解が計算されているため、この時点で5のインデックスは1に更新されています。第3ステップで4->5が計算されると、実際には4->1の最適解が計算されていることになります。

以上のステップを完了した後、最終的にDistance Textureを更新する必要があります。
```hlsl
// 現在のボクセル座標。
const uint voxel_index = id3(id.x, id.y, id.z);

// ジャンプバッファから最適なボクセルインデックスを取得します。
const uint cloest_voxel_index = _jump_buffer[voxel_index];
// インデックスを座標に変換します。
const int3 cloest_voxel_coord = unpack_id3(cloest_voxel_index);
// この最適なボクセル座標に保存されているメッシュまでの最短距離を取得します。
const float cloest_voxel_distance = _distance_texture_rw[cloest_voxel_coord];

// 現在のボクセルから最適なボクセルまでの距離（UVW空間、理由は前述の通り）。
const float distance_to_cloest_voxel = length(float3(id) / _max_dimension - float3(cloest_voxel_coord) / _max_dimension);

// 最終的な距離は、現在のボクセルから最適なボクセルまでの距離に、最適なボクセルからメッシュまでの距離を加え、さらにベイク設定で指定されたオフセットを加えたものです。
_distance_texture_rw[int3(id.x, id.y, id.z)] = cloest_voxel_distance + distance_to_cloest_voxel + g_push_constants.offset;
```
*注意：ジャンプフラッディングアルゴリズムは高速な近似方法であり、各ボクセルが必ずしも最短距離に更新されるわけではありません。*

これでDistance Textureには計算が完了したUDFデータが保存されました。可視化を行うことができます。

![Image UDF](images/udf.png)

図からわかるように、メッシュ表面に近い場所ほど色が濃く（数値が小さく距離が近い）、遠い場所ほど明るくなっています（数値が大きく距離が遠い）。

また、等値面を通じてメッシュを再構築することもできます。

![Image UDF Mesh](images/udf_mesh.png)


## SDFベイク

UDFと比較して、SDFのベイクははるかに複雑です。ここでの実装はUnityの[Visual Effect Graph](https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@14.0/manual/sdf-in-vfx-graph.html)の方法を参考にしています。

### 第一步：初期化

ベイク設定項目を追加します：
```rust
pub sign_passes_count: i32, // 符号パス（符号が正か負かを探す）の反復回数。
pub in_out_threshold: f32,  // メッシュの内側か外側かを判断する閾値。
```

次に、ベイク全体で使用するいくつかのパラメータを保存するためのグローバルUBOを準備します。具体的な内容は以下のコードのコメントに示されています。
```rust
pub struct GlobalUniform {
  pub dimensions: [u32; 3],   // ベイクするメッシュのBoundingBox情報とベイクボクセルの最大解像度に基づいて3つの次元のサイズを計算します。
  pub upper_bound_count: u32, // 各ボクセルに含まれる三角形のバッファの上限を保存します。
  pub num_of_triangles: u32,  // ベイク対象のメッシュの総三角形数。
  pub max_size: f32,          // ベイク領域の最長辺の長さに基づきます。
  pub max_dimension: u32,     // ボクセル空間の最長辺のボクセル数。
  pub center: [f32; 3],       // ベイク領域のBoundingBoxの中心座標。
  pub extents: [f32; 3],      // ベイク領域のBoundingBoxの半長。
}
```
他の値の計算はUDFと同じです。upper_bound_countについては、各ボクセルに含まれる三角形の数を正確に特定できないため、ここでは最大値を推定するしかありません。
```rust
// まず、ボクセルの半分に三角形が含まれていると仮定します。
let num_of_voxels_has_triangles = dimensions[0] as f64 * dimensions[1] as f64 * dimensions[2] as f64 / 2.0f64;
// 一つの三角形が隣接する8つのボクセルに共有されると仮定します。各ボクセルが総三角形数の平方根の数の三角形を持つと仮定します。
// 上記の二つの仮定の最大値を取ります。
let avg_triangles_per_voxel = (num_of_triangles as f64 / num_of_voxels_has_triangles * 8.0f64).max((num_of_triangles as f64).sqrt());
// 総計で必要な三角形数を保存します。
let upper_bound_count64 = (num_of_voxels_has_triangles * avg_triangles_per_voxel) as u64;
// 最大値を1536 * 2^18に制限します。
let upper_bound_count = (1536 * (1 << 18)).min(upper_bound_count64) as u32;
// 最小値を1024に制限します。
let upper_bound_count = upper_bound_count.max(1024);
```
*注意：これはあくまで保守的な推測であり、実際に必要な数はこれよりはるかに少ないかもしれません。保守的な推測を行うのは、より多くの境界ケースをカバーするためです。*

SDFのベイク全体で多くの一時バッファが必要です。ここではスペースの都合上、詳細な説明は省略します。具体的にはソースコードファイルを参照してください。

### 第二步：ジオメトリの構築

まず、UDFと同様にメッシュのVertex BufferとIndex Bufferから三角形情報を読み取り、正規化されたUVW空間に変換し、Triangle UVW Bufferに保存します。
```hlsl
Triangle tri_uvw;
tri_uvw.a = (get_vertex_pos(id.x, 0) - _center + _extents) / _max_size;
tri_uvw.b = (get_vertex_pos(id.x, 1) - _center + _extents) / _max_size;
tri_uvw.c = (get_vertex_pos(id.x, 2) - _center + _extents) / _max_size;

_triangles_uvw_rw[id.x] = tri_uvw;
```

次に、各三角形の「方向」を計算します。ここでの「方向」とは、三角形がどの軸に大まかに向いているかを示し、XY、ZX、YZのどの平面に最も近いかを表します。結果はCoord Flip Bufferに保存されます。
```hlsl
const float3 a = get_vertex_pos(id.x, 0);
const float3 b = get_vertex_pos(id.x, 1);
const float3 c = get_vertex_pos(id.x, 2);
const float3 edge0 = b - a;
const float3 edge1 = c - b;
const float3 n = abs(cross(edge0, edge1));
if (n.x > max(n.y, n.z) + 1e-6f) {  // 比較をより安定させるためにイプシロンを追加。
  // 三角形がYZ平面にほぼ平行
  _coord_flip_buffer_rw[id.x] = 2;
} else if (n.y > max(n.x, n.z) + 1e-6f) {
  // 三角形がZX平面にほぼ平行
  _coord_flip_buffer_rw[id.x] = 1;
} else {
  // 三角形がXY平面にほぼ平行
  _coord_flip_buffer_rw[id.x] = 0;
}
```
ここでなぜZX平面なのかというと、後続で3つの方向で計算を行う必要があるためです。ZX平面はY軸方向で計算する際に、ローカルのX軸が実際にはZ、ローカルのY軸が実際にはXを表します。

各三角形に方向を割り当てた後は、各方向で三角形を保守的にラスタライズします。
その前に、3つの方向の正交および投影行列を計算します。
```rust
// 視点位置、回転軸、幅、高さ、近接面距離、遠方面距離に基づいてView行列とProj行列を構築します。
let calculate_world_to_clip_matrix = |eye, rot, width: f32, height: f32, near: f32, far: f32| {
  let proj = glam::Mat4::orthographic_rh(-width / 2.0, width / 2.0, -height / 2.0, height / 2.0, near, far);
  let view = glam::Mat4::from_scale_rotation_translation(glam::Vec3::ONE, rot, eye).inverse();
  proj * view
};
```

Z方向のXY平面は以下の図のようになります。ローカルX軸は世界のX軸、ローカルY軸は世界のY軸です。

![Image XY Plane](images/xy_plane.png)

```rust
let xy_plane_mtx = {
  // 視点が正Z方向に1追加された位置から下を見る。
  let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(0.0, 0.0, bounds.extents[2] + 1.0);
  // View空間はデフォルトで下を向いているため、回転は不要。
  let rot = glam::Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0);
  // 近接面は1、これが視点位置が1追加される理由です。近接面にスペースを確保します。
  let near = 1.0f32;
  // 遠方面は近接面からベイク領域のZ方向の長さ全体に延びます。
  let far = near + bounds.extents[2] * 2.0;
  calculate_world_to_clip_matrix(pos, rot, bounds.extents[0] * 2.0, bounds.extents[1] * 2.0, near, far)
};
```

Y方向のZX平面は以下の図のようになります。ローカルX軸は世界のZ軸、ローカルY軸は世界のX軸です。

![Image ZX Plane](images/zx_plane.png)

```rust
let zx_plane_mtx = {
  // 視点が正Y方向に1追加された位置から外を見る（Y軸の正方向から負方向を見る）。
  let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(0.0, bounds.extents[1] + 1.0, 0.0);
  // まずY軸に沿って-90度回転し、次にX軸に沿って-90度回転します。ローカルX軸を世界のZ軸に、ローカルY軸を世界のX軸に合わせます。
  let rot = glam::Quat::from_euler(glam::EulerRot::YXZ, -std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2, 0.0);
  let near = 1.0f32;
  let far = near + bounds.extents[1] * 2.0;
  calculate_world_to_clip_matrix(pos, rot, bounds.extents[2] * 2.0, bounds.extents[0] * 2.0, near, far)
};
```

X方向のYZ平面は以下の図のようになります。ローカルX軸は世界のY軸、ローカルY軸は世界のZ軸です。

![Image YZ Plane](images/yz_plane.png)

```rust
let yz_plane_mtx = {
  // 視点が正X方向に1追加された位置から左を見る。
  let pos = glam::Vec3::from_array(bounds.center) + glam::Vec3::new(bounds.extents[0] + 1.0, 0.0, 0.0);
  // まずX軸に沿って90度回転し、次にY軸に沿って90度回転します。ローカルX軸を世界のY軸に、ローカルY軸を世界のZ軸に合わせます。
  let rot = glam::Quat::from_euler(glam::EulerRot::XYZ, std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2, 0.0);
  let near = 1.0f32;
  let far = near + bounds.extents[0] * 2.0;
  calculate_world_to_clip_matrix(pos, rot, bounds.extents[1] * 2.0, bounds.extents[2] * 2.0, near, far)
};
```

次に、上記の3つの方向で、対応する方向の三角形を保守的にラスタライズ処理します。
まず、三角形のカバー範囲の2D AABBを計算し、float4に保存します。xyはmin、zwはmaxを保存します。
```hlsl
// 三角形の3つの頂点を取得し、clip空間に変換します。
[unroll(3)]
for (i = 0; i < 3; i++) {
  vertex_in_clip[i] = mul(_world_to_clip[current_axis], float4(get_vertex_pos(id.x, i), 1.0));
}

// AABBのサイズを計算します。
float4 aabb = float4(1.0, 1.0, -1.0, -1.0);
aabb.xy = min(aabb.xy, min(vertex_in_clip[0].xy, min(vertex_in_clip[1].xy, vertex_in_clip[2].xy)));
aabb.zw = max(aabb.xy, max(vertex_in_clip[0].xy, max(vertex_in_clip[1].xy, vertex_in_clip[2].xy)));
float2 conservative_pixel_size;
// 現在のラスタライズ方向に基づき、設定されたConservative Offsetパラメータに基づいて実際に必要なOffsetピクセルサイズを計算します。
if (current_axis == 0) {
  conservative_pixel_size = float2(_conservative_offset / _dimensions.x, _conservative_offset / _dimensions.y);
} else if (current_axis == 1) {
  conservative_pixel_size = float2(_conservative_offset / _dimensions.z, _conservative_offset / _dimensions.x);
} else {
  conservative_pixel_size = float2(_conservative_offset / _dimensions.y, _conservative_offset / _dimensions.z);
}

// AABBサイズを拡大します。
_aabb_buffer_rw[id.x] = aabb + float4(-conservative_pixel_size.x, -conservative_pixel_size.y, conservative_pixel_size.x, conservative_pixel_size.y);
```

次に三角形をラスタライズし、設定されたOffsetを拡大します。ここで保守的にラスタライズを拡大するのは、float計算時の誤差による「隙間」を防ぐためです。
```hlsl
// 三角形が存在する平面をfloat4に保存し、xyzは平面法線方向、wは平面が原点からの距離を示します。
const float3 normal = normalize(cross(vertex_in_clip[1].xyz - vertex_in_clip[0].xyz, vertex_in_clip[2].xyz - vertex_in_clip[0].xyz));
const float4 triangle_plane = float4(normal, -dot(vertex_in_clip[0].xyz, normal));

// 法線方向がZ正方向（1）か負方向（-1）かを計算します。
const float direction = sign(dot(normal, float3(0, 0, 1)));
float3 edge_plane[3];
[unroll(3)]
for (i = 0; i < 3; i++) {
  // 2Dエッジ平面を計算します。Wは同次座標です。
  edge_plane[i] = cross(vertex_in_clip[i].xyw, vertex_in_clip[(i + 2) % 3].xyw);
  // 以前に決定された方向とオフセットピクセル値に基づいてエッジ平面を「外側」に押し出します。
  // ここは理解しにくい場合は後で図を参照してください。
  edge_plane[i].z -= direction * dot(conservative_pixel_size, abs(edge_plane[i].xy));
}

float4 conservative_vertex[3];
bool is_degenerate = false;
[unroll(3)]
for (i = 0; i < 3; i++) {
  _vertices_buffer_rw[3 * id.x + i] = float4(0, 0, 0, 1);

  // 三つのエッジ平面に基づいて、新しい頂点位置を計算します。
  conservative_vertex[i].xyw = cross(edge_plane[i], edge_plane[(i + 1) % 3]);

  // W値に基づいて三角形が退化しているかどうかを判断します。
  if (abs(conservative_vertex[i].w) < CONSERVATIVE_RASTER_EPS) {
    is_degenerate |= true;
  } else {
    is_degenerate |= false;
    conservative_vertex[i] /= conservative_vertex[i].w; // この後、wは1になります。
  }
}
if (is_degenerate)
  return;

// 三角形上の点を通じて、平面方程式を満たすように3つの頂点のZ値を計算します。
// 平面方程式：ax + by + cz + d = 0。
// Zを計算：z = -(ax + by + d) / c。
// 最後に新しく得られた3つの頂点をVertices Bufferに書き込みます。
[unroll(3)]
for (i = 0; i < 3; i++) {
  conservative_vertex[i].z = -(triangle_plane.x * conservative_vertex[i].x + triangle_plane.y * conservative_vertex[i].y + triangle_plane.w) / triangle_plane.z;
  _vertices_buffer_rw[3 * id.x + i] = conservative_vertex[i];
}
```
計算機グラフィックスでは、平面は四次元ベクトルで表すことができます：float4(plane) = (a, b, c, d)、ここで平面の方程式は ax + by + cz + d = 0です。「エッジ平面」の概念は、2D投影上のジオメトリ（例えば三角形）を処理する際に、三角形の境界を表す平面を使用するという考えに基づいています。

    edge_plane[i] = cross(vertex_in_clip[i].xyw, vertex_in_clip[(i + 2) % 3].xyw);

このコードの中で、具体的にエッジ平面を構築する方法は、2つの頂点の同次座標のクロス積を取得することによって行われます。ここで、vertex_in_clip は頂点の同次座標です。vertex_in_clip[i].xyw は頂点の x, y, w 成分を抽出し、それを3次元ベクトルとして扱います。cross 関数は2つの3次元ベクトルのクロス積を計算し、それらのベクトルが存在する平面に垂直なベクトルを生成します。この生成されたベクトル edge_plane[i] は、vertex_in_clip[i] から vertex_in_clip[(i + 2) % 3] までのエッジ平面を表します（同次座標下での2D平面の表現に注意）。

ここでは、保守的ラスタライズ後の三角形をモデル空間に戻し、赤いワイヤーフレームは拡大された三角形を、白いワイヤーフレームは元の三角形を示しています。各三角形がその所在平面に沿って一回り拡大されていることがわかります。

![Image Conservative Offset](images/conservative_offset.png)

### ステップ3：三角形被覆ボクセル計数統計

次に、Compute Shaderから一時的に離れ、Vertex ShaderとFragment Shaderを使用して3つの方向における三角形の被覆回数を統計します。
まず、Vertex Shaderを見てみましょう。
```hlsl
struct VertexInput {
  // Draw(num_of_triangles * 3)を通じて、Vertex Idを渡します。
  uint vertex_id: SV_VertexID;
};

ToFragment main(VertexInput input) {
  ToFragment output = (ToFragment)0;

  // Vertex Idに基づいて、前のステップのラスタライズ結果のVertices BufferからClip空間の頂点データを直接読み取ります。
  const float4 pos = _vertices_buffer[input.vertex_id];
  // Vertex Idを単純に3で割って三角形IDを取得します。
  output.triangle_id = input.vertex_id / 3;
  // 現在の三角形が現在の描画方向と異なる場合、(-1, -1, -1, -1)を渡してFragment Shaderをスキップさせます。
  if (_coord_flip_buffer[output.triangle_id] != g_push_constants.current_axis) {
    output.position = float4(-1, -1, -1, -1);
  } else {
    output.position = pos;
  }

  return output;
}
```

次に、Fragment Shaderの全体的な流れを見てみましょう。
```hlsl
struct ToFragment {
  float4 position: SV_Position;
  uint triangle_id: TEXCOORD0;
};

FragmentOutput main(ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

  // Vertex Shaderから渡されたpositionと三角形IDに基づいて、現在処理しているピクセルのボクセル座標voxel_coordを計算します。
  // 同時に、深度方向に内側にbackwardおよび外側にforwardに拡張処理が可能かどうかを判断します。
  int3 depth_step, voxel_coord;
  bool can_step_backward, can_step_forward;
  get_voxel_coordinates(input.position, input.triangle_id, voxel_coord, depth_step, can_step_backward, can_step_forward);

  // ボクセル中心座標を正規化されたUVW空間に変換し、Voxels Bufferに格納します。
  float3 voxel_uvw = (float3(voxel_coord) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension;
  _voxels_buffer_rw[id3(voxel_coord)] = float4(voxel_uvw, 1.0f);
  // 現在のボクセル座標のCounter Bufferで累加し、このボクセルが三角形に一度被覆されたことをマークします。
  InterlockedAdd(_counter_buffer_rw[id3(voxel_coord)], 1u);
  // 外側に拡張可能な場合、外側のボクセルに対して同じ操作を行います。
  if (can_step_forward) {
    _voxels_buffer_rw[id3(voxel_coord + depth_step)] = float4(voxel_uvw, 1.0f);
    InterlockedAdd(_counter_buffer_rw[id3(voxel_coord + depth_step)], 1u);
  }
  // 内側に拡張可能な場合、内側のボクセルに対して同じ操作を行います。
  if (can_step_backward) {
    _voxels_buffer_rw[id3(voxel_coord - depth_step)] = float4(voxel_uvw, 1.0f);
    InterlockedAdd(_counter_buffer_rw[id3(voxel_coord - depth_step)], 1u);
  }

  // ここでのRTの出力はベイクプロセスには関与せず、デバッグ用として使用されます。
  output.color = float4(voxel_uvw, 1);
  return output;
}
```
全体の流れとしては、VSとFSを利用して、三角形の被覆領域でCounter Bufferに対して累加操作を行います。
次に、`get_voxel_coordinates`の実装を詳しく見てみましょう。
```hlsl
void get_voxel_coordinates(
  float4 screen_position,
  uint triangle_id,
  out int3 voxel_coord,
  out int3 depth_step,
  out bool can_step_backward,
  out bool can_step_forward
) {
  // 現在のスクリーン解像度、すなわち現在の方向のボクセルの幅と高さを取得します。
  // 例えば、現在のボクセル空間が[2, 3, 4]の場合、Z方向のXY平面で処理する際には2 x 3を返します。
  const float2 screen_params = get_custom_screen_params();
  // Vertex Shaderから渡されたPositionをUVW空間に変換します。
  screen_to_uvw(screen_position, screen_params);
  // 三角形IDに基づいて、以前計算された三角形被覆領域のAABBを取得し、AABB範囲外の場合は現在のFragment Shaderの後続処理を破棄します。
  cull_with_aabb(screen_position, triangle_id);
  // ボクセル座標を計算し、前方および後方に拡張可能かどうかを決定します。
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
次に、`compute_coord_and_depth_step`の実装を詳しく見てみましょう。
```hlsl
void compute_coord_and_depth_step(
  float2 screen_params,
  float4 screen_position,
  out int3 voxel_coord,
  out int3 depth_step,
  out bool can_step_backward,
  out bool can_step_forward
) {
  // ここでは保守的に三角形が隣接する前後のボクセルと共有されると仮定します。これにより、後続の表示上の問題を回避できます。
  can_step_forward = true;
  can_step_backward = true;

  if (g_push_constants.current_axis == 1) {
    // UVW空間のPositionを使用してボクセル座標を計算します。
    voxel_coord = (screen_position.xyz * float3(screen_params, _dimensions[1]));
    voxel_coord.xyz = voxel_coord.yzx;

    // 境界かどうかを判断し、そうでない場合は内側および外側に拡張可能です。
    depth_step = int3(0, 1, 0);
    if (voxel_coord.y <= 0) {
      can_step_backward = false;
    }
    if (voxel_coord.y >= _dimensions[1] - 1) {
      can_step_forward = false;
    }
  } else if (g_push_constants.current_axis == 2) {
    // 上記と基本的に同じですが、特定の軸の方向が異なります。
  } else {
    // 上記と基本的に同じですが、特定の軸の方向が異なります。
  }
}
```
深度書き込みと深度テストが無効になっているため、これで3つの方向において、三角形に被覆されたボクセルはすべてInterlockedAddを使用してCounter Bufferに計数されました。
同時に、これらの被覆されたボクセルのUVW座標もVoxels Bufferに格納されました。

次に、Prefix Sumアルゴリズムを使用してCounter Bufferを累加し、最終結果をAccum Counter Bufferに格納します。その基本的な考え方は、配列の各位置にその前のすべての要素の和を事前に計算して格納することで、後続のクエリ操作を定数時間で完了できるようにすることです。
Prefix Sumアルゴリズムとベイク自体には直接の関係がないため、関連するアルゴリズムの紹介リンクのみを示します：
* [ウィキペディア](https://en.wikipedia.org/wiki/Prefix_sum)
* [GPU Gems 3 - Chapter 39. Parallel Prefix Sum (Scan) with CUDA](https://developer.nvidia.com/gpugems/gpugems3/part-vi-gpu-computing/chapter-39-parallel-prefix-sum-scan-cuda)

この時点で、Accum Counter Bufferには現在のボクセルの前のすべてのボクセルが含む（被覆された）三角形の数が保存されています。
例を挙げると、ボクセル0、1、2、3、4がそれぞれ4、2、5、0、3個の三角形に被覆されている場合、計数Bufferの値は次のようになります：

    0（現在のボクセルの前には他のボクセルがない）
    4（現在のボクセルの前には0番があり、0番には4個の三角形がある）
    6（現在のボクセルの前には0番と1番があり、0番には4個の三角形、1番には2個の三角形がある、合計6個）
    11（同様のアルゴリズム）
    11（同様のアルゴリズム）

次に、これらの三角形をTriangle Id Bufferに格納し、Accum Counter Bufferを使用して各ボクセルに含まれる三角形のリストを遍歴できます。ここでもVertex ShaderとFragment Shaderを使用します。

Vertex Shaderは以前と同じなので、ここでは繰り返しません。Fragmentの違いだけを見てみましょう。
```hlsl
// ここで計算されたボクセル座標を使用してCounter Bufferを1加算し、その後の値をTriangle Ids Bufferのインデックスとして使用します。
uint index = 0u;
InterlockedAdd(_counter_buffer_rw[id3(voxel_coord)], 1u, index);
// ここでは越境を防ぐため、最初に計算した各ボクセルの三角形Bufferの上限を使用します。
if (index < _upper_bound_count)
_triangle_ids_buffer_rw[index] = input.triangle_id;
// 同様に、外側および内側のボクセルに対して拡張します。
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
ここは非常に理解しやすいです。ボクセルiの前のボクセルに含まれる三角形の総数はすでにCounter Bufferに保存されているため、`_counter_buffer_rw[id3(voxel_coord)]`で取得されるのは、現在のボクセルiに三角形インデックスを書き込む開始位置です。

### 第四ステップ：Ray Mapの計算

上記のすべての計算が完了した後、以下のコードを使用して指定されたボクセルの三角形リストを遍歴することができます。
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

Ray Mapの計算を開始する前に、いくつかの補助関数を導入します。
```hlsl
// 線分と三角形の交点を計算します。交点がない場合は0を返し、三角形の辺上に交点がある場合は0.5または-0.5を返し、三角形の内部に交点がある場合は1.0または-1.0を返します。
// 符号は三角形の正面または裏面との交点を示します。tは交点パラメータを返します。
float intersect_segment_to_triangle_with_face_check(float3 segment_start, float3 segment_end, Triangle tri, out float t_value) {
  /*
   * 三角形平面方程式：n * (P - A) = 0
   * 線分方程式：P(t) = Q + t(S - Q)
   * n dot ((Q + t(S - Q)) - A) = 0
   * n dot (Q - A + t(S - Q)) = 0
   * n dot (Q - A) + t(n dot (S - Q)) = 0
   * 𝑣 = 𝑄 - 𝐴, 𝑑 = 𝑆 − 𝑄
   * t = - (n dot 𝑣) / (n dot d)
   *
   * ここで：
   * n - 三角形平面の法線ベクトル
   * P - 三角形平面上の任意の点
   * A - 三角形の頂点の一つ
   * Q, S - 線分の両端点
   * t - 交点パラメータ、線分と三角形の交点を記述するために使用
   * 𝑣 - ベクトル Q - A
   * 𝑑 - ベクトル S - Q
   */

  // 三角形の2つの辺を計算します。
  const float3 edge1 = tri.b - tri.a;
  const float3 edge2 = tri.c - tri.a;
  // ここで実際に計算しているのは -d = Q - Sです。
  const float3 end_to_start = segment_start - segment_end;

  // クロス積を使用して三角形平面の法線ベクトルを計算します。
  const float3 normal = cross(edge1, edge2);
  // 線分の方向と三角形の法線ベクトルのドット積を計算します。
  const float dot_product = dot(end_to_start, normal);
  // このドット積の符号は、線分が三角形の正面または裏面と交差しているかを示します。
  const float side = sign(dot_product);
  // 逆数を取ります。
  const float inverse_dot_product = 1.0f / dot_product;

  // v = Q - A
  const float3 vertex0_to_start = segment_start - tri.a;
  // 式に基づいて、交点のt値を計算します。
  // t = - (n dot v) / (n dot d)
  //   = (n dot v) / (n dot -d)
  float t = dot(vertex0_to_start, normal) * inverse_dot_product;

  // t値が0未満または1を超える場合、線分と三角形平面に交点がないことを意味します。
  if (t < -INTERSECT_EPS || t > 1 + INTERSECT_EPS) {
    t_value = 1e10f;
    return 0;
  } else {
    // 重心座標を計算して交点が三角形の内部にあるかどうかを確認します。
    const float3 cross_product = cross(end_to_start, vertex0_to_start);
    const float u = dot(edge2, cross_product) * inverse_dot_product;
    const float v = -dot(edge1, cross_product) * inverse_dot_product;
    float edge_coefficient = 1.0f;

    // 重心座標が指定された範囲内にない場合、交点は三角形の外部にあります。
    if (u < -BARY_EPS || u > 1 + BARY_EPS || v < -BARY_EPS || u + v > 1 + BARY_EPS) {
      t_value = 1e10f;
      return 0;
    } else {
      const float w = 1.0f - u - v;
      // 重心座標が三角形の辺上にある場合、係数を0.5に調整します。
      if (abs(u) < BARY_EPS || abs(v) < BARY_EPS || abs(w) < BARY_EPS) {
        edge_coefficient = 0.5f;
      }

      // t値と交差結果を返します。
      t_value = t;
      return side * edge_coefficient;
    }
  }
}

// 指定されたボクセル内で、前後左右上下の3方向から三角形との交点を計算します。
// 正方向（+x +y +z）と負方向（-x -y -z）で正面と裏面の交差する三角形の数の差を返します。
void calculate_triangle_intersection_with_3_rays(
  in Triangle tri,
  in int3 voxel_id,
  out float3 intersect_forward,
  out float3 intersect_backward
) {
  // 初期カウントはすべて0です。
  intersect_forward = float3(0.0f, 0.0f, 0.0f);
  intersect_backward = float3(0.0f, 0.0f, 0.0f);

  // 交点パラメータt。
  float t = 1e10f;
  // 正規化されたUVW空間での線分の開始点と終了点。
  float3 p, q;
  // 交差方向のカウント変数。
  float intersect = 0;

  // UVW空間で、X方向において、ボクセルの中心から線分の両端点を生成します。
  p = (float3(voxel_id) + float3(0.0f, 0.5f, 0.5f)) / _max_dimension;
  q = (float3(voxel_id) + float3(1.0f, 0.5f, 0.5f)) / _max_dimension;
  // 線分が左から右に向かう場合、三角形が右を向いていると、左側が内側（-）、右側が外側（+）を意味します。
  // しかし、この場合、線分が三角形の裏面を通過するため、負の値を返します。したがって、結果を反転します。
  intersect = -intersect_segment_to_triangle_with_face_check(p, q, tri, t);
  if (t < 0.5f) {
    // tが0.5未満の場合、交点が左側に近いことを意味するため、Backwardの符号カウントを累積します。
    intersect_backward.x += float(intersect);
  } else {
    // 逆に、交点が右側に近いことを意味するため、Forwardの符号カウントを累積します。
    intersect_forward.x += float(intersect);
  }

  // Y方向はX方向と同様で、軸が異なるだけです。
  ...

  // Z方向もX方向と同様で、軸が異なるだけです。
  ...
}
```

上記の2つの補助関数を使用すると、2x2のボクセル単位で、すべてのボクセルについて、その隣接するボクセルが三角形の正面が多いか裏面が多いかを計算できます。
8回に分けて、(0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 0), (1, 0, 1), (0, 1, 1), (1, 1, 1)の位置からすべてのボクセルを遍歴します。
```hlsl
for (uint i = start_triangle_id; i < end_triangle_id && (i < _upper_bound_count - 1); i++) {
  // 上記の補助関数を使用して、ボクセル[x, y, z]に含まれる三角形が「前面」で正面と裏面で交差する差、および「背面」で正面と裏面で交差する差を計算します。
  calculate_triangle_intersection_with_3_rays(tri, int3(id.xyz), intersect_forward, intersect_backward);

  // 「前面」の場合の結果をRay Mapのボクセル[x, y, z]に累積します。
  _ray_map_rw[id.xyz] += float4(intersect_forward, 1.0f);

  // 境界を越えない場合、「背面」の結果をRay Mapの隣接ボクセルに累積します。
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
*注意：線分と三角形が交差しない場合、`intersect_segment_to_triangle_with_face_check`の戻り値は0です。累積しても影響がないため、ここでは特に判断を行っていません。*

次に、これらの値を3つの方向からそれぞれ加算します。ここではX方向の計算のみを示します。
```hlsl
// 正方向から負方向に向かって累積します。
for (int t = _dimensions.x - 2; t >= 0; t--) {
  float count = _ray_map_rw[int3(t + 1, id.y, id.z)].x;
  _ray_map_rw[int3(t, id.y, id.z)] += float4(count, 0, 0, count != 0 ? 1 : 0);
}
```

これで、一連の計算を経て、Ray Mapを使用して任意のボクセルが右から左、上から下、後ろから前に向かって、いくつの三角形の正面といくつの三角形の裏面を通過したかの差を知ることができるようになりました。
次の符号判定のためのデータが準備されました。

以下のRay Mapの可視化図から、モデルの内部領域と外部領域を基本的に区別できることがわかります。

![Image Ray Map](images/ray_map.png)

### 第五ステップ：シンボルの計算

まず最初にSign Mapを初期化します。
```hlsl
const float4 self_ray_map = _ray_map[id.xyz];
// 現在のボクセルに対応するRay Mapには、右から左、後ろから前、上から下への正の交差と負の交差の差分が格納されています。
const float right_side_intersection = self_ray_map.x;
const float back_side_intersection = self_ray_map.y;
const float top_side_intersection = self_ray_map.z;
// 各方向の最初の平面内のボクセルには、右から左、後ろから前、上から下への正の交差と負の交差の差分が格納されています。
// 現在のボクセルの値を引くと、左側、前側、下側の正の交差と負の交差の差分が残ります。
const float left_side_intersection = _ray_map[int3(0, id.y, id.z)].x - self_ray_map.x;
const float front_side_intersection = _ray_map[int3(id.x, 0, id.z)].y - self_ray_map.y;
const float bottom_side_intersection = _ray_map[int3(id.x, id.y, 0)].z - self_ray_map.z;
// これらをすべて加算すると、現在のボクセルが正の交差が多いか負の交差が多いかを大まかに示し、それが「内側」か「外側」かを意味します。
_sign_map_rw[id.xyz] =
  right_side_intersection - left_side_intersection +
  back_side_intersection - front_side_intersection +
  top_side_intersection - bottom_side_intersection;
```

この時点では各ボクセル周囲の軸方向の交差の影響のみを考慮しており、まだ正確ではありません。したがって、n回繰り返し、毎回ランダムに8つの隣接ボクセルを取り、6つの経路に沿ってサンプリング計算を行い、精度を向上させます。
ここでnormalize_factorの初期値は6です。これは次に6つの異なる経路を通じて加算されるためです。
繰り返し倍増し、各反復は前の結果に累積され、最後の反復でのみ正規化が行われます。
```rust
let num_of_neighnors = 8u32;
let mut normalize_factor = 6.0f32;
for i in 1..=self.settings.sign_passes_count {
  // コンピュートシェーダーをディスパッチ。
  ...
  normalize_factor += num_of_neighnors as f32 * 6.0 * normalize_factor;
}
```
コンピュートシェーダーは以下の通りです：
```hlsl
const float4 self_ray_map = _ray_map[id.xyz];
// 8つの隣接ボクセルをループで取得。
for (uint i = 0; i < g_push_constants.num_of_neighbors; i++) {
  int3 neighbors_offset = generate_random_neighbor_offset((i * g_push_constants.num_of_neighbors) + g_push_constants.pass_id, _max_dimension * 0.05f);
  int3 neighbors_index;
  neighbors_index.x = min((int)(_dimensions.x - 1), max(0, (int)id.x + neighbors_offset.x));
  neighbors_index.y = min((int)(_dimensions.y - 1), max(0, (int)id.y + neighbors_offset.y));
  neighbors_index.z = min((int)(_dimensions.z - 1), max(0, (int)id.z + neighbors_offset.z));

  // 6つの異なる経路で隣接ボクセルに到達するシンボルの累積値を計算。
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

// 最後の反復が終了したら正規化を行います。
if (g_push_constants.need_normalize) {
  const float normalize_factor_final = g_push_constants.normalize_factor + g_push_constants.num_of_neighbors * 6 * g_push_constants.normalize_factor;
  _sign_map_rw[id.xyz] /= normalize_factor_final;
}
```

この時点でSign Mapを可視化すると、モデルの内部領域と外部領域を明確に区別できるようになります。

![Image Sign Map](images/sign_map.png)

### 第六ステップ：閉じた表面

まず、メッシュが完全に閉じているわけではないため、内外の「穴」の境界を見つけます。つまり、指定されたしきい値付近のSign Mapの正負の値が隣接している場所です。
```hlsl
// 設定されたしきい値に基づいてスコアを計算。
const float self_sign_score = _sign_map[id.xyz] - g_push_constants.threshold;
// スコアがしきい値の10%未満の場合。
if (abs(self_sign_score / g_push_constants.threshold) < 0.1f) {
  // 現在のボクセルのスコアと右側のボクセルのスコアが反対の場合、境界が見つかります。
  if (self_sign_score * (_sign_map[id.xyz + uint3(1, 0, 0)] - g_push_constants.threshold) < 0) {
    // 現在のボクセルのスコアに基づいて、自身または右側のボクセルに書き込みます。
    const uint3 write_coord = id.xyz + (self_sign_score < 0 ? uint3(1, 0, 0) : uint3(0, 0, 0));
    // 書き込む内容は、ボクセル座標をUVW空間に正規化した値です。
    _voxels_texture_rw[write_coord.xyz] = float4((float3(write_coord) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension, 1.0f);
  }
  // 同様にY軸を処理。
  ...
  // 同様にZ軸を処理。
  ...
}
```

次に、以前に処理したVoxels Bufferのボクセル（三角形で覆われたもの）の情報をVoxelsに書き込みます。
```hlsl
const float4 voxel = _voxels_buffer[id3(id.xyz)];
if (voxel.w != 0.0f)
  _voxels_texture_rw[id.xyz] = voxel;
```

この時点で、すべての境界のボクセル、三角形で覆われたボクセルがVoxels Textureに保存され、ジャンプフラッドの準備が整いました。
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
ここでのジャンプフラッドの伝播はUDFとほぼ同じなので、詳しい説明は省略します。
唯一の違いは、正式なジャンプの前にオフセット1の初期ジャンプを行い、これにより表面に近いディテールを事前に補完できる点です。ジャンプフラッドは近似アルゴリズムであるためです。
この時点で、Voxels Texture全体に各ボクセルがメッシュ表面に最も近い距離の座標が保存されました。

### 第七ステップ：最終的なシンボル距離場の計算

```hlsl
// Voxels Textureに保存された種子座標と現在のボクセルの正規化されたUVW座標を取得。
const float3 seed_coord = _voxels_texture[int3(id.x, id.y, id.z)].xyz;
const float3 voxel_coord = (float3(id.xyz) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension;

// 指定されたしきい値に基づいて現在のボクセルのシンボルを決定。
float sign_d = _sign_map[id.xyz] > g_push_constants.threshold ? -1 : 1;

// 種子座標に基づいてボクセルのインデックス座標を計算。
const int3 id_seed = int3(seed_coord * _max_dimension);

// 種子ボクセルの三角形リストの開始位置と終了位置を取得。
uint start_triangle_id = 0;
[branch]
if(id3(id_seed) > 0) {
  start_triangle_id = _accum_counters_buffer[id3(id_seed) - 1];
}
uint end_triangle_id = _accum_counters_buffer[id3(id_seed)];

// すべての三角形を巡回し、現在のボクセルから種子ボクセルが覆う三角形までの最短距離を取得。
float distance = 1e6f;
for (uint i = start_triangle_id; (i < end_triangle_id) && (i < _upper_bound_count - 1); i++) {
  const uint triangle_index = _triangles_in_voxels[i];
  Triangle tri = _triangles_uvw[triangle_index];
  distance = min(distance, point_distance_to_triangle(voxel_coord, tri));
}
// 特殊な場合、種子ボクセルに三角形がない場合、距離は現在のボクセルから種子ボクセルまでのUVW座標距離を使用。
if (1e6f - distance < COMMON_EPS) {
  distance = length(seed_coord - voxel_coord);
}
// シンボルとオフセットを適用し、シンボル距離を取得。
distance = sign_d * distance - g_push_constants.offset;

// シンボル距離をImageとBufferに保存。Imageはレンダリングに使用でき、Bufferはエクスポートに使用できます。
_voxels_buffer_rw[id3(id)] = float4(distance, distance, distance, distance);
_distance_texture_rw[id] = distance;
```

この時点で、すべてのベイク計算が完了し、SDFデータが取得されました。以下は、SDFを使用してRay Marchingで法線方向を色としてレンダリングした可視化結果です。

![Image SDF Normal 0](images/sdf_normal_0.png)

![Image SDF Normal 1](images/sdf_normal_1.png)
