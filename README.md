# ⚡ 媒体批处理工具（Media Batch Tool）

多合一媒体批处理桌面应用：**智能瘦身、格式转换、社交媒体适配**，面向普通消费者，完全免费、纯本地处理。

- 🖥️ 平台：macOS + Windows（Tauri 2）
- 🧩 前端：Vue 3 + TypeScript + Vite + TailwindCSS + shadcn-vue 风格组件 + Pinia
- 🦀 后端：Rust（图像处理 / ffmpeg 视频转码 / 并行批处理队列）
- 🔒 隐私：所有文件只在本机处理，不联网、不上传

## 功能

| 模式 | 说明 |
|---|---|
| ✨ 智能瘦身 | 自动压缩图片与视频，肉眼几乎无差别，体积大幅减小；压缩后更大则自动跳过 |
| 🔄 格式转换 | JPG / PNG / WebP / AVIF / MP4 互转（AVIF 走 ffmpeg SVT-AV1） |
| 📱 社交媒体适配 | 微信 / 朋友圈 / 抖音 / 小红书 / Instagram 等平台模板一键适配 |

- 场景模板机制：用户不面对参数，只面对场景；可自定义模板
- 自定义水印：文字 / 图片，位置、透明度可调
- 批量队列：图片 rayon 并行（`min(核数, 8)`），视频固定并发 2
- 实时进度：单文件 + 批次双粒度事件流；可随时停止
- 单文件失败隔离：坏文件不影响整批；失败可重试
- i18n：中文 / English

## 使用说明（用户手册）

### 三步完成批量处理

1. **选择场景**：首页选「智能瘦身」「格式转换」或「社交媒体适配」
2. **拖入文件**：把图片/视频（或整个文件夹）拖进窗口，也可以点「选择文件 / 文件夹」
3. **选择方案 → 开始处理**：按需选择方案（如"发抖音"），点「开始处理」

### 三种模式

| 模式 | 适用场景 |
|---|---|
| ✨ 智能瘦身 | 图片/视频太大，想发给别人或节省空间 |
| 🔄 格式转换 | 打不开的格式（HEIC→JPG）、网页用图（WebP/AVIF）、统一视频为 MP4 |
| 📱 社交媒体适配 | 发微信/朋友圈/抖音/小红书/Instagram 前一键调整到平台规格 |

### 常用操作

- **随时停止**：处理中点「停止」，已完成的文件保留
- **失败重试**：处理完成后在完成页点「重试」只重跑失败的文件
- **导出**：点「导出到…」选择文件夹；勾选「替换原文件」会直接覆盖同名文件（默认自动改名保留）
- **自定义水印**：新建模板时可添加文字水印（需系统字体）或图片水印（推荐透明 PNG），支持 5 种位置与透明度
- **语言**：首页左下角可切换 中文 / English
- **深色模式**：跟随系统外观

### 隐私

所有处理都在本机完成，不上传任何数据，不需要账号。

## 项目结构

```
media-batch-tool/
├── src/                    # 前端（Vue 3）
│   ├── views/              # 5 屏主流程
│   ├── components/         # 文件卡片 / 进度条 / 模板卡片 / UI 组件
│   ├── stores/             # Pinia（文件、任务、模板、模式）
│   ├── lib/                # Tauri IPC 封装、格式化工具
│   └── locales/            # zh / en 文案
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── commands.rs     # IPC 命令层
│   │   ├── image_proc.rs   # 图像处理管道（压缩/转换/水印/AVIF）
│   │   ├── video_proc.rs   # ffprobe/ffmpeg 转码 + 进度解析
│   │   ├── queue.rs        # 批处理队列（rayon + tokio + 取消）
│   │   ├── templates.rs    # 内置/自定义模板
│   │   ├── thumbnails.rs   # 缩略图生成
│   │   └── ffmpeg.rs       # ffmpeg/ffprobe 解析（env → sidecar → PATH）
│   ├── binaries/           # ffmpeg/ffprobe sidecar（gitignored）
│   ├── capabilities/       # 生产权限（不含 WDIO）
│   ├── capabilities-e2e/   # 仅 E2E 构建（--features wdio）追加的 WDIO 权限
│   └── tauri.conf.json
├── e2e/                    # Playwright（浏览器 mock）+ real-app.spec.ts（真实应用）
├── wdio.conf.ts            # WebdriverIO 真实应用 E2E 配置
├── scripts/                # fetch-ffmpeg / build-update-json / 基准
├── docs/                   # 设计文档与实施计划
└── website/                # 官网/下载页（静态）
```

## 开发

前置：Node ≥ 22（jsdom 30 需要 ^22.22.2）+ pnpm、Rust stable、ffmpeg（本机 PATH 即可，开发期使用系统 ffmpeg）。

```bash
pnpm install
pnpm tauri dev        # 开发模式（HMR）
```

## 测试

```bash
pnpm build            # 前端类型检查 + 构建
pnpm vitest run       # 前端单元测试
cd src-tauri
cargo test            # Rust 单元 + 集成测试（含真实媒体 fixture）
cargo test --release -- --ignored bench_compression   # 压缩基准

# 浏览器模式主流程 E2E（需先启动 dev server：pnpm dev）
pnpm exec playwright test

# 真实应用 E2E（WebdriverIO + 嵌入式 WebDriver，macOS 原生）
pnpm e2e:real:build   # ① 构建 E2E 二进制：dev 前端 + release + --features "wdio tauri/custom-protocol"
pnpm test:e2e:real    # ② 启动真实 app 跑完整 智能瘦身 流程
```

### 真实应用 E2E 说明

- 测试二进制是 **release** 构建（release 才嵌入前端资源）并启用 `wdio` feature（注册嵌入式 WebDriver 与 `browser.tauri.execute`）；前端用 `vite build --mode development`（`NODE_ENV=development`）构建，以便打包进 `@wdio/tauri-plugin` 与测试钩子。
- `wdio` 是可选 feature：生产构建（`cargo build --release --features tauri/custom-protocol` 或 `tauri build`）不包含任何 WDIO 代码；`build.rs` 按 feature 切换 capabilities（`capabilities-e2e/` 只在 `--features wdio` 时生效）。
- 原生文件对话框无法自动化，测试通过 dev-only 钩子 `window.__MBT_ADD_PATHS__`（仅在 DEV 前端构建中存在，生产构建会被 tree-shake）向应用注入 `e2e/fixtures/` 里的真实媒体文件。

## 打包发布

```bash
# 1. 下载静态 ffmpeg sidecar（mac 或 Windows 各跑一次）
bash scripts/fetch-ffmpeg.sh

# 2. 打包（生产 ffmpeg 用静态构建，见 scripts/fetch-ffmpeg.sh 注释）
bash scripts/build-release.sh
# 等价于：
#   pnpm tauri build --bundles app
#   bash src-tauri/target/release/bundle/dmg/bundle_dmg.sh --skip-jenkins <dmg> <app>
```

> 备注：macOS .dmg 的 Finder 美化步骤需要交互式 GUI 会话（AppleScript），
> 无头/CI 环境会超时，`build-release.sh` 用 `--skip-jenkins` 跳过该步骤。

- **签名 / 公证**：macOS 需 Developer ID + notarization（CI 中配置证书）；Windows 建议 EV 证书。
- **自动更新**：`tauri-plugin-updater`。发布时生成签名 `latest.json`（见 `scripts/build-update-json.sh`），私钥 `scripts/updater.key`（gitignore）+ 公钥 `scripts/updater.key.pub`（已提交）；CI 用 Secret `TAURI_SIGNING_PRIVATE_KEY`。
- **CI**：`.github/workflows/ci.yml` 双平台构建 + 测试；`release.yml` 打 tag 触发发布。
- **授权**：自有代码 MIT（见 `LICENSE`）；随附 ffmpeg/ffprobe 为 GPL 静态构建，详见 `THIRD_PARTY_NOTICES.md`。

## 设计文档

- 设计：`docs/superpowers/specs/2026-08-17-media-batch-tool-design.md`
- 实施计划：`docs/plans/2026-08-17-media-batch-tool-implementation-plan.md`

## 已知限制

> Windows 验证：
> - ✅ 本地已通过 `cargo check --target x86_64-pc-windows-gnu`（含 `--tests`）——
>   安装 mingw-w64 后整个项目（tauri/ring/image/webp 等全部依赖 + 我们的代码）
>   在 Windows 目标下编译通过；sidecar 用 `scripts/fetch-ffmpeg.sh` 获取
> - ⏳ MSVC 目标构建/链接与运行行为由 CI 的 `windows-latest` runner 权威验证
>   （`.github/workflows/ci.yml`）

- HEIC 在 macOS 通过系统 `sips` 解码，Windows 端暂无 HEIC 解码
- WebP 损失压缩使用 `webp` crate（libwebp），AVIF 需要 ffmpeg 可用
- 文字水印需要系统字体（macOS PingFang / Windows 微软雅黑），可用 `MBT_WATERMARK_FONT` 覆盖
- 生产分发需自行配置静态 ffmpeg 与签名证书
