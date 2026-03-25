# 七巧板 O/S framebuffer 显示设计方案

## 1. 目标

在 `tg-rcore-tutorial-ch1` 的最小裸机框架上，完成以下目标：

- 通过 QEMU 图形设备对应的 framebuffer 直接绘制像素；
- 使用“数组定义的七巧板几何数据”绘制两组图案；
- 左侧绘制由七巧板拼接的 `O`，右侧绘制由七巧板拼接的 `S`；
- 绘制完成后保持显示状态，便于肉眼验收。

## 2. 技术路径

### 2.1 启动与参数传递

- 沿用当前 `_start` 裸入口与手动设栈流程；
- 通过寄存器将 QEMU 启动参数中的 `hartid` / `dtb_pa` 传递给 `rust_main`；
- 在 `rust_main` 内基于 `dtb_pa` 解析设备树中的 framebuffer 描述。

### 2.2 framebuffer 获取

- 使用 no_std 下的最小 FDT 解析逻辑；
- 扫描 `compatible = "simple-framebuffer"` 节点；
- 读取 `reg`、`width`、`height`、`stride`、`format` 属性，构建 framebuffer 视图；
- 若未发现可用 framebuffer，则回退串口输出错误并异常关机。

### 2.3 绘制模型

- 图元采用数组表达，定义点坐标、颜色和片段类型；
- 每个字母都由 7 个“七巧板片段”组成：
  - 2 个大三角形
  - 1 个中三角形
  - 2 个小三角形
  - 1 个正方形
  - 1 个平行四边形
- 通过统一光栅化函数将三角形与四边形转成像素填充。

### 2.4 画布布局

- 全屏清空为深色背景；
- 按分栏布局：左半屏放 `O`，右半屏放 `S`；
- 采用统一缩放因子适配不同分辨率，保证在常见分辨率下都能完整显示；
- 每个片段采用高对比度颜色，便于观察片段边界。

### 2.5 QEMU 运行配置

- 调整 `.cargo/config.toml` 的 runner：
  - 去除 `-nographic`
  - 增加 `-device ramfb`
- 使运行 `cargo run` 时可在图形窗口中看到 framebuffer 结果。

## 3. 关键实现点

- `main.rs` 新增：
  - FDT 头与结构块解析；
  - framebuffer 描述结构；
  - 像素写入函数；
  - 三角形/四边形填充；
  - 七巧板数组常量与字母组合绘制；
- 保持 no_std、no_main、panic 收口、SBI 关机流程一致。

## 4. 测试方案

### 4.1 编译验证

- 执行 `cargo build`，确认交叉编译通过。

### 4.2 运行与显示验证

- 执行 `cargo run`；
- 验证出现 QEMU 图形窗口；
- 验证窗口中左侧是七巧板 `O`，右侧是七巧板 `S`；
- 验证无 panic 与异常退出。

### 4.3 代码质量验证

- 执行 `cargo clippy --target riscv64gc-unknown-none-elf -- -D warnings`；
- 执行 `cargo check --target riscv64gc-unknown-none-elf`。

## 5. 风险与对策

- 风险：部分环境下 FDT 不含 `simple-framebuffer` 节点。  
  对策：在 runner 中显式启用 `ramfb`，并保留串口错误提示与失败收口路径。

- 风险：framebuffer 格式不一致。  
  对策：优先支持 `x8r8g8b8`/`a8r8g8b8`，其它格式按近似路径输出或直接拒绝。

- 风险：分辨率差异导致图案拉伸。  
  对策：以较小边为基准计算缩放，保证图案始终完整落在画布内。
