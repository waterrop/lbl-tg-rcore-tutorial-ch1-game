# QEMU 图形窗口未显示七巧板图案的 bug 处理记录

## 1. bug 现象

执行 `cargo run` 后，QEMU 图形窗口能够弹出，但窗口内容显示：

`guest has not initialized the display (yet)`

这说明 QEMU 已经创建了图形设备，但来宾系统并没有成功完成显示初始化，因此 framebuffer 没有真正生效，也就无法显示七巧板 `O` 和 `S` 图案。

## 2. bug 本质

这个 bug 的本质不是“绘图逻辑错误”，而是“显示设备初始化失败”。

项目使用的是 QEMU 的 `ramfb` 设备。该设备要求来宾系统通过 `fw_cfg` 的 `etc/ramfb` 项，向 QEMU 写入一份 `RamfbCfg` 配置结构，告诉 QEMU：

- framebuffer 在哪块物理内存；
- 分辨率是多少；
- 像素格式是什么；
- 每行步长是多少。

只有这一步完成后，QEMU 才会把该内存区域当作图形 framebuffer 来显示。

之前的程序虽然已经实现了图形绘制代码，但 `ramfb` 初始化链路有问题，导致 QEMU 没收到有效配置，所以图形窗口一直停留在“guest 尚未初始化显示”的状态。

## 3. 排查流程

### 3.1 第一阶段：确认不是简单退出问题

最开始的问题表现为 `cargo run` 直接退出，没有图形窗口。排查后先确认：

- QEMU runner 已经去掉 `-nographic`；
- 已经添加 `-device ramfb`；
- 程序没有立即 panic 或直接正常关机。

修正后，QEMU 窗口能出现，但内容仍不正确，进入下一阶段排查。

### 3.2 第二阶段：确认不是 DTB framebuffer 路径问题

程序最初尝试从 DTB 中解析 `simple-framebuffer` 节点。

排查发现：

- 当前 `-device ramfb` 场景下，并不能依赖 DTB 一定提供 `simple-framebuffer`；
- 因此需要改为通过 `fw_cfg` 初始化 `ramfb`。

这一步确认了问题方向：核心不在图形数据，而在 `fw_cfg + ramfb` 初始化。

### 3.3 第三阶段：验证 `fw_cfg` 文件目录是否可读

继续排查 `fw_cfg` 访问流程，重点验证是否能找到 `etc/ramfb` 这一项。

处理方式包括：

- 通过 `fw_cfg` 的文件目录项读取逻辑查找 `etc/ramfb`；
- 在 QEMU 监视器中检查内存变化；
- 将读取到的目录项信息写入内存进行旁路验证。

最终确认：

- `fw_cfg` 文件目录可以正常读取；
- `etc/ramfb` 确实存在；
- selector 也能正确拿到。

这说明“找不到 `etc/ramfb`”不是根因。

### 3.4 第四阶段：定位到 DMA 提交流程错误

接下来重点检查向 `etc/ramfb` 写入配置时的 DMA 过程。

排查时先后验证过：

- selector 的字节序是否正确；
- `RamfbCfg` 是否需要 `packed`；
- fourcc 与 stride 的组合是否合理；
- 是否需要回退到不同的 selector 写法；
- DMA doorbell 的写入方式是否正确。

最终确认真正问题在于：

- 之前对 `FW_CFG_DMA` 的触发写法不正确；
- QEMU 没有收到一份有效的 DMA 请求；
- 因此 `ramfb` 配置虽然在 guest 侧准备好了，但没有真正提交给 QEMU。

也就是说，问题不是“配置内容完全错误”，而是“配置提交链路没有按 QEMU 期望方式完成”。

### 3.5 第五阶段：用截图结果验证是否真正显示成功

修复后，并没有只依赖“窗口不报错”来判断成功，而是进一步做了验证：

- 让 QEMU 在无窗口模式下运行；
- 通过 QEMU monitor 执行 `screendump`；
- 解析生成的 ppm 图像内容；
- 检查像素颜色种类和分布。

验证结果显示：

- 图像中存在背景色；
- 图像中存在 7 种七巧板颜色；
- 像素分布不是全黑、全白或单色；
- 说明 framebuffer 已经被正确初始化并被程序实际绘制。

因此可以确认图形窗口中已经能够显示七巧板 `O` 和 `S`。

## 4. 最终修复内容

本次 bug 修复的关键点有：

1. 保留 framebuffer 绘图逻辑，但把问题聚焦到 `ramfb` 初始化；
2. 使用 `fw_cfg` 文件目录查找 `etc/ramfb` 对应 selector；
3. 使用符合 QEMU 规范的 DMA 方式提交 `RamfbCfg`；
4. 调整 `RamfbCfg` 结构布局与像素格式配置；
5. 在 `rust_main` 中优先走可用的 `ramfb` 初始化路径；
6. 通过 QEMU monitor 的截图分析，而不是只靠肉眼猜测结果。

## 5. 这个 bug 的结论

这个 bug 的结论可以概括为一句话：

> 七巧板图案没有显示，不是因为图形绘制代码本身出错，而是因为 QEMU 的 `ramfb` 没有被 guest 正确初始化。

进一步展开就是：

- QEMU 窗口出现，说明设备被创建了；
- 窗口提示 `guest has not initialized the display (yet)`，说明设备没有被正确配置；
- 根因在 `fw_cfg -> etc/ramfb -> DMA 提交` 这条链路；
- 修复 DMA 提交流程后，QEMU 才真正把内存映射为可显示的 framebuffer；
- 随后七巧板 `O` 和 `S` 的绘图结果才能正确显示出来。
