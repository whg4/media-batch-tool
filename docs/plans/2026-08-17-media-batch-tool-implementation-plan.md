# 媒体批处理工具 · 实施计划

- 日期：2026-08-17
- 设计文档：`docs/superpowers/specs/2026-08-17-media-batch-tool-design.md`
- 技术栈：Tauri 2 + Rust · Vue 3 + TS + Vite + TailwindCSS + shadcn-vue + Pinia
- 目标平台：macOS + Windows

## 前置条件

- Rust 工具链（rustup，stable）
- Node.js ≥ 20 + pnpm
- Tauri 2 CLI（`cargo install tauri-cli` 或 npm 依赖）
- macOS：Xcode Command Line Tools；Windows：VS Build Tools + WebView2（系统自带）
- ffmpeg/ffprobe 二进制（开发期本机安装，发布期用 sidecar 打包）

---

## 阶段 0：M0 技术验证（1-2 周）

**目标：验证"能跑"——双平台下 Tauri + ffmpeg + 图像压缩 + 进度流全链路跑通。**

| # | 任务 | 验收标准 |
|---|---|---|
| 0.1 | 用 `create-tauri-app`（vue-ts 模板）初始化项目骨架 | 双平台 `pnpm tauri dev` 能启动窗口 |
| 0.2 | 接入 TailwindCSS + shadcn-vue + Pinia + Vue Router | 页面可渲染基础组件，`dark:` 主题切换生效 |
| 0.3 | 图像压缩 PoC：`image` + webp/avif 编码器，实现 `compress_image` 命令 | CLI/命令可把测试图压缩到目标质量，输出可解码 |
| 0.4 | ffmpeg sidecar PoC：打包 ffmpeg/ffprobe 为 sidecar，`ffprobe` 读元数据 + `ffmpeg` 转码 | 视频转码成功；解析 `-progress` 输出得到实时百分比 |
| 0.5 | IPC 事件流 PoC：`start_batch` 命令 + `file_progress` 事件推送到前端 | 前端实时收到逐文件进度事件，无轮询 |
| 0.6 | 双平台冒烟：mac + Windows 各跑一遍 0.3-0.5 | 两侧行为一致 |

**M0 退出标准**：一个最小 demo（拖 3 张图 + 1 个视频 → 压缩/转码 → 前端实时进度 → 输出结果）在 mac 和 Windows 上都跑通。

## 阶段 1：M1 MVP（4-6 周）

**目标：5 屏主流程 + 三个模式全功能可用（全免费）。**

| # | 任务 | 验收标准 |
|---|---|---|
| 1.1 | 前端骨架：5 屏路由（模式选择/拖拽区/模板选择/处理中/完成）+ 空状态 | 页面导航完整，空状态有引导文案 |
| 1.2 | 拖拽区：拖文件/文件夹、文件卡片列表（虚拟滚动）、去重标记、不支持格式提示 | 拖入 1000 文件列表不卡 |
| 1.3 | `analyze_files` + 缩略图生成（Rust）→ 前端 `convertFileSrc` 展示 | 元数据（尺寸/大小/时长）与缩略图正确显示 |
| 1.4 | 模板系统：模板 schema + 校验 + 模板选择器 UI（卡片式） | 模板定义可解析校验；社交模式展示平台模板卡片 |
| 1.5 | 图片处理引擎：图片管道 + `tokio` 队列 + `rayon` 并发（`min(核,8)`） | 批量压缩正确；体积收益校验（更大则跳过并提示） |
| 1.6 | 视频处理引擎：ffprobe → ffmpeg 转码 → 进度解析 → 输出校验 | 视频并发=2；失败自动清理临时文件 |
| 1.7 | 处理中屏：总进度/单文件状态/取消（CancellationToken + kill ffmpeg）/失败内联重试 | 取消后无残留临时文件；坏文件不拖垮批次 |
| 1.8 | 完成页：体积对比 + 节省汇总 + 导出（保留/替换原文件） | 导出正确；数字统计与文件实际大小一致 |

**M1 退出标准**：真实使用场景（手机照片 200 张 + 视频 5 个）完整跑通三个模式，全程 UI 不卡，错误场景（损坏文件、磁盘满）有合理提示。

## 阶段 2：M2 打磨发布（+2-4 周）

| # | 任务 | 验收标准 |
|---|---|---|
| 2.1 | 全量社交模板（微信/朋友圈/抖音/小红书/Instagram）+ 自定义模板编辑 | 模板符合各平台规格，可新增/修改/删除 |
| 2.2 | i18n：文案抽离，中文默认，英文可切 | 语言切换后主要界面完整 |
| 2.3 | 错误处理强化 + 媒体基准测试 | 每周基准监控压缩质量/体积收益不退化 |
| 2.4 | 签名/公证（mac Developer ID + notarization，Windows 证书） | 安装包通过公证，无警告或可解释 |
| 2.5 | `tauri-plugin-updater` 自动更新 + CI 双平台流水线 | 发布新版本后可自动更新；CI 双平台绿 |
| 2.6 | 官网/下载页分发（.dmg + NSIS/.exe） | 双平台安装包可下载安装，附开源声明 |

**M2 退出标准**：双平台发布安装包可用、自动更新链路通、用户手册级 README 完成。

---

## 横切任务（贯穿全程）

- **测试**：Rust 单测（golden 测试）+ 集成测试（真实 fixture 混批/取消）；前端 Vitest + Playwright E2E 主流程
- **CI**：GitHub Actions（macos-latest + windows-latest）：build → test → package
- **版本管理**：semver；M0/M1/M2 各打 tag

## 风险与应对

| 风险 | 应对 |
|---|---|
| WKWebView/WebView2 渲染差异 | 尽早双平台联测，E2E 冒烟进 CI |
| ffmpeg sidecar 平台打包复杂 | 版本锁定 + 构建脚本统一管理，M0 就验证 |
| WebView 预览大图卡顿 | 缩略图优先策略（M1 任务 1.3 落实） |
| 图像/视频质量不达预期 | 每周基准测试，M0 先用代表性样本验证 |

## 开始顺序建议

先做 **0.1 + 0.3**（骨架 + 图片压缩），这两步最快建立信心；随后 0.4（ffmpeg）是最大不确定项，尽早验证。
