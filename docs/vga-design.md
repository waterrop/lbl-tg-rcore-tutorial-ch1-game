# lbl-tg-rcore-tutorial-vga 设计文档

## 1. 设计目标

`lbl-tg-rcore-tutorial-vga` 是一个面向 `RISC-V 64 + QEMU` 场景的 Rust 图形显示组件，目标是在裸机或内核环境中提供最小但可复用的 framebuffer 显示能力。

组件需要满足以下目标：

- 提供统一的显示初始化接口，屏蔽 QEMU 图形设备初始化细节；
- 提供逐像素写入接口，支持上层内核自行构建图形、字符或简单窗口系统；
- 提供清屏接口，支持快速切换场景或重绘整个画面；
- 以组件化方式封装，方便其他内核以依赖库方式复用；
- 保持 `no_std` 友好，不依赖操作系统运行时；
- 优先支持 QEMU `ramfb + fw_cfg` 方案，并兼容 `simple-framebuffer` 信息接入。

## 2. 使用场景与边界

### 2.1 适用场景

- 教学型操作系统内核；
- 基于 QEMU 的裸机图形实验；
- 需要最小图形输出能力的 `no_std` RISC-V 项目；
- 需要复用显示初始化与像素写入能力的内核组件体系。

### 2.2 非目标范围

- 不负责 2D/3D 硬件加速；
- 不负责窗口管理、字体排版、输入事件；
- 不负责复杂 GPU 驱动协商；
- 不以 Linux DRM/KMS 完整驱动为目标。

## 3. 总体设计思路

组件采用“显示配置发现 + framebuffer 抽象 + 最小绘图接口”三层设计。

1. 底层负责获取 framebuffer 所需的物理地址、分辨率、步长和像素格式；
2. 中间层将这些信息封装为 `VgaDevice`，对上提供统一写像素与清屏能力；
3. 上层内核只需要持有设备句柄，即可独立实现图标、字符、图元或 UI 绘制逻辑。

这样设计的核心价值在于：上层调用者不需要理解 QEMU `ramfb`、`fw_cfg`、设备树等底层细节，只需要使用稳定的 Rust API。

## 4. framebuffer 获取方案

### 4.1 首选方案：ramfb + fw_cfg

对于可复用组件，首选 `ramfb + fw_cfg` 方式。

原因如下：

- framebuffer 物理地址可由内核或组件主动选择，控制权更强；
- 不强依赖设备树中是否存在 `simple-framebuffer` 节点；
- 更适合封装成独立组件发布，便于不同内核集成；
- 与 QEMU `virt` 机器模型兼容性较好。

该方案的工作流程为：

1. 组件内部预留一段连续物理内存作为 framebuffer；
2. 通过 `fw_cfg` 查询 `etc/ramfb` 配置入口；
3. 组织 `RamfbCfg` 结构体，写入 framebuffer 基址、宽高、stride、像素格式；
4. 通过 DMA 或等效提交方式通知 QEMU；
5. QEMU 将这块内存映射为图形显示缓冲区；
6. 后续所有写像素操作都直接写入该内存区域。

### 4.2 兼容方案：simple-framebuffer

为增强组件适配性，可保留 `simple-framebuffer` 接入能力。

该方案适用于：

- 平台已经在设备树中给出 framebuffer；
- 上层内核已有 DTB 解析能力；
- 不希望组件自行分配 framebuffer 内存。

兼容路径为：

- 由调用方传入已解析好的 framebuffer 参数；
- 或由组件在启用对应特性时解析设备树节点；
- 最终统一转换成内部 `FramebufferInfo` 结构。

### 4.3 策略抽象

组件内部建议定义显示发现策略：

- `Ramfb`
- `SimpleFramebuffer`
- `Custom`

其中：

- `Ramfb` 由组件完成初始化与地址注册；
- `SimpleFramebuffer` 使用外部已知地址；
- `Custom` 允许其他平台自行提供 framebuffer 元信息。

这样可保证该组件虽然当前服务于 QEMU RISC-V 64，但未来仍可扩展到其他虚拟平台或板级环境。

## 5. 组件架构

建议按如下逻辑模块组织：

### 5.1 `config` 模块

职责：

- 定义初始化配置；
- 定义分辨率、步长、像素格式、发现策略；
- 提供默认值与参数校验。

核心结构：

- `VgaInitConfig`
- `Resolution`
- `PixelFormat`
- `DiscoveryMethod`

### 5.2 `device` 模块

职责：

- 封装显示设备状态；
- 提供对外公开 API；
- 维护 framebuffer 基地址、宽高、stride、格式等运行时信息。

核心结构：

- `VgaDevice`
- `FramebufferInfo`

### 5.3 `ramfb` 模块

职责：

- 封装 QEMU `fw_cfg` 寄存器访问；
- 查找 `etc/ramfb`；
- 组装并提交 `RamfbCfg`；
- 返回可用的 framebuffer 描述。

### 5.4 `fb` 模块

职责：

- 实现像素写入；
- 实现清屏；
- 执行颜色打包和边界检查；
- 提供基础矩形填充等可选内部能力。

### 5.5 `error` 模块

职责：

- 统一定义初始化失败、参数非法、格式不支持、地址不可用等错误；
- 为上层内核提供明确的失败原因。

## 6. 对外接口设计

组件对外暴露最小接口集合，满足题目要求并兼顾可扩展性。

### 6.1 初始化接口

建议接口：

```rust
pub fn init(config: VgaInitConfig) -> Result<VgaDevice, VgaError>
```

职责：

- 根据配置选择发现策略；
- 初始化 QEMU 显示链路；
- 构建并返回设备句柄；
- 在失败时返回显式错误，而不是静默退出。

### 6.2 写入像素接口

建议接口：

```rust
impl VgaDevice {
    pub fn write_pixel(&mut self, x: usize, y: usize, color: Rgb888) -> Result<(), VgaError>;
}
```

职责：

- 将逻辑颜色转换为目标像素格式；
- 计算 framebuffer 偏移地址；
- 执行越界检查；
- 将像素值写入显示缓冲区。

### 6.3 清屏接口

建议接口：

```rust
impl VgaDevice {
    pub fn clear(&mut self, color: Rgb888);
}
```

职责：

- 按当前分辨率遍历 framebuffer；
- 将所有像素设置为指定颜色；
- 作为最基本的整屏刷新能力。

### 6.4 可选辅助接口

为提升组件复用性，建议预留但不强制首版实现的接口：

```rust
impl VgaDevice {
    pub fn width(&self) -> usize;
    pub fn height(&self) -> usize;
    pub fn stride(&self) -> usize;
    pub fn framebuffer_paddr(&self) -> usize;
}
```

这些接口可以帮助上层内核进行布局计算或调试验证。

## 7. 核心数据结构

建议定义如下数据结构：

```rust
pub struct VgaInitConfig {
    pub method: DiscoveryMethod,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: PixelFormat,
    pub framebuffer_paddr: Option<usize>,
}

pub struct FramebufferInfo {
    pub paddr: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: PixelFormat,
}

pub struct VgaDevice {
    info: FramebufferInfo,
}
```

设计原则：

- `VgaInitConfig` 表达初始化输入；
- `FramebufferInfo` 表达已确认的硬件视图；
- `VgaDevice` 负责封装运行期设备状态；
- 颜色类型建议单独定义，避免直接暴露裸 `u32` 降低可读性。

## 8. 初始化时序

以 `ramfb + fw_cfg` 为例，初始化时序如下：

1. 调用方构造 `VgaInitConfig`；
2. `init()` 检查宽高、stride、对齐和地址有效性；
3. 组件准备 framebuffer 内存区域；
4. 访问 `fw_cfg` 文件目录，定位 `etc/ramfb`；
5. 构造 `RamfbCfg` 并写入配置；
6. 生成 `FramebufferInfo`；
7. 返回 `VgaDevice`；
8. 调用方后续通过 `write_pixel()` 和 `clear()` 完成显示。

若采用 `simple-framebuffer` 路径，则跳过 `fw_cfg` 提交，直接构造 `FramebufferInfo`。

## 9. 像素写入设计

`write_pixel()` 的内部流程建议如下：

1. 检查 `(x, y)` 是否越界；
2. 根据 `offset = y * stride + x * bytes_per_pixel` 计算偏移；
3. 将 `Rgb888` 转为目标格式，例如 `X8R8G8B8` 或 `A8R8G8B8`；
4. 使用易失写入方式把像素写入 framebuffer；
5. 返回成功或错误结果。

关键约束：

- 必须避免写出 framebuffer 边界；
- 必须保证 `stride` 与像素字节数一致；
- 对于不支持的格式应立即报错；
- 应保持实现简单，不在首版引入复杂缓冲同步机制。

## 10. 清屏设计

`clear()` 本质是对 framebuffer 的批量填充。

建议实现方式：

- 预先将颜色转换成目标像素值；
- 双层循环遍历 `height * width`；
- 按行优先顺序连续写入；
- 若后续需要性能优化，可改为按 `u32` 或更宽字长批量写入。

首版优先保证正确性，其次再考虑性能。

## 11. 错误处理设计

建议定义统一错误类型：

```rust
pub enum VgaError {
    InvalidConfig,
    UnsupportedFormat,
    FramebufferNotFound,
    FwCfgUnavailable,
    RamfbConfigureFailed,
    OutOfBounds,
}
```

错误处理原则：

- 初始化错误通过 `Result` 返回；
- 运行期越界写像素返回 `OutOfBounds`；
- 不在组件内部直接关机或 panic，由上层内核决定失败策略；
- 对外暴露可诊断错误，便于教学与调试。

## 12. 兼容性与可复用性设计

为了适合作为可发布组件，需要控制对外依赖和平台耦合。

### 12.1 Rust 侧约束

- 使用 `no_std`；
- 避免必须依赖分配器；
- 不强依赖特定内核框架；
- 公共 API 保持简单稳定。

### 12.2 平台侧约束

- 首版面向 `riscv64`；
- 首版默认适配 QEMU `virt`；
- 平台寄存器地址、DMA 提交细节放入内部模块，不泄露到公共 API。

### 12.3 复用方式

其他内核集成该组件时，仅需：

1. 提供一块可访问的 framebuffer 物理内存，或允许组件分配固定区域；
2. 在 QEMU 启动参数中启用 `ramfb`；
3. 调用 `init()` 获得 `VgaDevice`；
4. 使用 `write_pixel()` 和 `clear()` 构建自身绘制逻辑。

## 13. 建议目录结构

建议将组件实现为独立 crate，目录结构如下：

```text
lbl-tg-rcore-tutorial-vga/
├── Cargo.toml
└── src
    ├── lib.rs
    ├── config.rs
    ├── device.rs
    ├── fb.rs
    ├── ramfb.rs
    └── error.rs
```

其中：

- `lib.rs` 负责导出公共 API；
- `ramfb.rs` 保留平台相关实现；
- `device.rs` 负责设备对象与接口组织；
- `fb.rs` 聚焦像素访问逻辑。

## 14. 测试方案

### 14.1 编译验证

- 执行 `cargo check --target riscv64gc-unknown-none-elf`；
- 执行 `cargo build --target riscv64gc-unknown-none-elf`。

### 14.2 接口验证

- 初始化后读取 `width()`、`height()` 等信息确认设备状态正确；
- 调用 `clear()` 后观察屏幕是否显示纯色背景；
- 调用 `write_pixel()` 在若干关键坐标写入不同颜色，验证颜色与位置是否正确。

### 14.3 集成验证

- 在 QEMU 中启用 `-device ramfb`；
- 运行一个最小示例内核；
- 先清屏为背景色，再绘制简单几何图形；
- 验证图形窗口不再显示 `guest has not initialized the display (yet)`。

### 14.4 异常验证

- 传入非法分辨率，验证初始化失败；
- 模拟 `fw_cfg` 不可用，验证返回 `FwCfgUnavailable`；
- 向越界坐标写像素，验证返回 `OutOfBounds`。

## 15. 风险与应对

- 风险：QEMU 未启用 `ramfb`。  
  应对：初始化时检测配置提交结果，失败时返回明确错误。

- 风险：不同环境像素格式不一致。  
  应对：首版仅支持少量 32 位格式，并在接口层显式约束。

- 风险：framebuffer 地址或大小配置错误。  
  应对：在初始化时进行地址、stride、容量一致性校验。

- 风险：组件过度绑定当前内核实现。  
  应对：把平台相关逻辑封装在内部模块，对外仅暴露通用设备接口。

## 16. 结论

`lbl-tg-rcore-tutorial-vga` 的首版设计应围绕“最小可用、no_std、可复用”展开，以 `ramfb + fw_cfg` 作为主要显示初始化路径，以 `write_pixel()` 和 `clear()` 作为核心能力出口。

该设计既能满足当前在 QEMU RISC-V 64 中显示图形的需求，也为后续在其他内核中复用、扩展图元绘制能力或支持更多 framebuffer 来源保留了清晰演进空间。
