# 七巧板 O/S framebuffer 实现工作过程

## 1. 代码改造过程

1. 阅读现有 `src/main.rs`、`.cargo/config.toml`、`tg-sbi` 源码，确认当前模板仅提供串口与关机能力。
2. 在 `src/main.rs` 中新增 FDT 解析逻辑，从 `dtb_pa` 查找 `simple-framebuffer` 节点并提取 `reg/width/height/stride/format`。
3. 新增 framebuffer 像素写入与清屏逻辑，建立最小绘制后端。
4. 新增三角形光栅化、四边形拆分填充，以及“数组驱动”的七巧板片段定义。
5. 新增 `O_PIECES` 与 `S_PIECES` 两组 7 片图元数据，按左右分栏布局绘制。
6. 修改 `.cargo/config.toml` 的 QEMU runner，启用 `-device ramfb` 以提供图形 framebuffer。

## 2. 测试执行记录

### 2.1 编译与静态检查

- `cargo check`：通过
- `cargo build`：通过
- `cargo check --target riscv64gc-unknown-none-elf`：通过
- `cargo clippy --target riscv64gc-unknown-none-elf -- -D warnings`：通过

### 2.2 运行验证

- `timeout 8 cargo run`：QEMU 启动成功，进入图形运行路径；
- `timeout 5 qemu-system-riscv64 -machine virt -device ramfb -display none -serial stdio -bios none -kernel target/riscv64gc-unknown-none-elf/debug/lbl-tg-rcore-tutorial-ch1-game`：
  - 命令在超时后返回（说明内核保持在绘制后的驻留循环）；
  - 串口未输出 `framebuffer not found`，说明成功命中 framebuffer 节点并进入图形分支。

## 3. 结果说明

- 程序已从“串口 Hello World”改为“framebuffer 图形绘制”。
- 图形数据以数组形式描述七巧板片段，并组合渲染成 `O` 与 `S`。
- 在图形可见环境中执行 `cargo run` 可直接观察最终图案。
