# hala-sdf-baker
[![License](https://img.shields.io/badge/License-GPL3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)
[![MSRV](https://img.shields.io/badge/rustc-1.70.0+-ab6000.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

[English(TODO)](README_EN.md) | [中文](README.md) | [日本語(TODO)](README_JP.md) | [한국어(TODO)](README_KO.md)

## 引言

在现代计算机图形学和游戏开发中，有一个技术被广泛认为是不可或缺的，那就是使用有向距离场（Signed Distance Fields, SDF）和无向距离场（Unsigned Distance Fields, UDF）。SDF和UDF提供了一种高效而强大的手段来表达和操作复杂的几何形状。它们在渲染、碰撞检测、模型生成等多个领域中扮演着重要角色。

SDF是一种典型的表示方法，它为每个点在空间中分配一个实数值，表示该点到最近表面的有向距离。这种结构不但可以用来高效地进行形状建模，还可以用于执行几何操作如平滑、膨胀或缩小形状等。与之相对的，UDF记录的是距离表面的绝对最短距离，这在处理具有不规则或复杂拓扑的模型时特别有用。

SDF和UDF不仅仅是数据结构，它们更是在多维空间中表示形状的一种方法。在视频游戏开发中，利用SDF进行实时阴影计算和环境光遮蔽已成为一种流行的技术。这是因为SDF可以迅速确定光线与几何表面的接触点，从而有效地生成软阴影和其他视觉效果。此外，在实时图形中，采用SDF可以进行高效的几何建模和修改，如角色动态变形，或是开发中常见的破坏效果等。在工业视觉和科学可视化领域，UDF常被用于形状重建和数据拟合，尤其是在处理来自扫描设备或其他测量设备的数据时。通过构建一个准确的UDF，研究者可以从一组离散的数据点中推断出一个连继的三维表面，这对于重建复杂的生物形态或其他科学结构尤为关键。本项目，将通过Rust和Vulkan实现SDF和UDF的烘焙。

![Image Intro](images/intro.png)

图一：来自https://arxiv.org/abs/2011.02570。上半为UDF，只记录了距离表面的绝对最短距离。下半为SDF，除了记录最短距离，正负号还表示了是在“内”还是“外”。

## UDF烘焙

算法实现上UDF相对简单，这里先从UDF烘焙讲起。

### 第一步：初始化

To be continue...