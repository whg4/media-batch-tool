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
│   └── tauri.conf.json
├── scripts/                # fetch-ffmpeg / build-update-json / 基准
├── docs/                   # 设计文档与实施计划
└── website/                # 官网/下载页（静态）
```

## 开发

前置：Node ≥ 20 + pnpm、Rust stable、ffmpeg（本机 PATH 即可，开发期使用系统 ffmpeg）。

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
```

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
- **自动更新**：`tauri-plugin-updater`。发布时生成签名 `latest.json`（见 `scripts/build-update-json.sh`），私钥存 CI Secret `TAURI_SIGNING_PRIVATE_KEY`（本地示例：`scripts/updater.private.key`，已被 gitignore）。
- **CI**：`.github/workflows/ci.yml` 双平台构建 + 测试；`release.yml` 打 tag 触发发布。
- **ffmpeg 授权**：使用 LGPL 版静态构建可避免 GPL 传染；最终产品随附开源声明。

## 设计文档

- 设计：`docs/superpowers/specs/2026-08-17-media-batch-tool-design.md`
- 实施计划：`docs/plans/2026-08-17-media-batch-tool-implementation-plan.md`

## 已知限制

- HEIC 在 macOS 通过系统 `sips` 解码，Windows 端暂无 HEIC 解码
- WebP 损失压缩使用 `webp` crate（libwebp），AVIF 需要 ffmpeg 可用
- 文字水印需要系统字体（macOS PingFang / Windows 微软雅黑），可用 `MBT_WATERMARK_FONT` 覆盖
- 生产分发需自行配置静态 ffmpeg 与签名证书
