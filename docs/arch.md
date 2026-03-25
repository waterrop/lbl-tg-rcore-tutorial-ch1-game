# 项目架构设计（七巧板 framebuffer 版本）

## 1. 总体结构

本项目仍是单文件内核入口架构，但逻辑上分为四层：

1. 启动层：`_start` 裸函数设栈并跳转 `rust_main`。
2. 设备发现层：从 QEMU 传入的 DTB 中解析 framebuffer 参数。
3. 渲染层：像素写入、清屏、三角形/四边形填充。
4. 场景层：七巧板数组数据与 `O`/`S` 布局组合。

## 2. 启动与控制流

- QEMU `-bios none` 启动后，先进入 `tg-sbi` 的 M 态入口；
- `tg-sbi` 完成最小环境配置并切换到 S 态 `_start`；
- `_start` 初始化 S 态栈后跳转 `rust_main(hartid, dtb_pa)`；
- `rust_main` 执行 framebuffer 发现、绘制和驻留循环；
- 若关键步骤失败，走串口错误输出并 `shutdown(true)`。

## 3. 核心模块职责

### 3.1 FDT 解析模块

- 解析 FDT header 和 structure block token；
- 识别 `compatible = simple-framebuffer` 节点；
- 提取 `reg/width/height/stride/format` 构成 `Framebuffer`。

### 3.2 Framebuffer 模块

- `put_pixel`：按 `(x, y)` 写入 32 位像素；
- `pack`：按格式封装颜色值；
- `clear`：清空画布背景色。

### 3.3 栅格化模块

- `fill_triangle`：边函数判定三角形内部像素；
- `draw_piece`：三角形直接绘制、四边形拆成两三角形；
- `draw_letter`：批量渲染字母对应的 7 片图元。

### 3.4 图案数据模块

- `O_PIECES` 和 `S_PIECES` 是固定数组常量；
- 每个片段包含颜色、顶点数组、顶点数；
- 通过统一坐标变换适配不同分辨率。

## 4. 运行时数据流

1. `dtb_pa` 输入到 FDT 解析逻辑；
2. 得到 framebuffer 基址与分辨率；
3. 场景层下发片段数组；
4. 栅格化层逐像素写入 framebuffer；
5. GPU/显示设备读取 framebuffer，输出 `O` 与 `S` 图像。

## 5. 关键设计取舍

- 采用 no_std 自研最小 FDT 解析，避免新增依赖；
- 采用数组常量表达图形，保证数据可审计和可复用；
- 采用软件光栅化，保持实现简单并适配教学场景；
- 保持故障可诊断路径：framebuffer 不可用时串口输出错误并退出。
