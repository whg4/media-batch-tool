# Third-party notices

本产品（媒体批处理工具）的**自有代码**以 MIT 协议开源（见 `LICENSE`）。

## FFmpeg / FFprobe

发布安装包内附带的 `ffmpeg` / `ffprobe` 静态构建使用 **GPL** 许可，
随附二进制仅作为独立 sidecar 进程调用，不链接进应用代码。

- 版本：ffmpeg 9.0（aarch64-apple-darwin，osxexperts.net 静态构建）
- 来源：
  - macOS arm64：https://www.osxexperts.net/（ffmpeg9arm / ffprobe9arm）
  - macOS x86_64：https://evermeet.cx/ffmpeg/
  - Windows：https://www.gyan.dev/ffmpeg/builds/（release-essentials）
- FFmpeg 源码：https://ffmpeg.org/download.html
- 获取/重打包脚本：`scripts/fetch-ffmpeg.sh`

FFmpeg 遵循 LGPL/GPL 选择条款。若需规避 GPL 传染，可替换为 LGPL 静态构建
（见 `scripts/fetch-ffmpeg.sh` 注释）后重新打包发布。

## 其他前端依赖

见 `package.json` / `src-tauri/Cargo.toml` 中各自的开源许可。
