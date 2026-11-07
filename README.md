# WaveRelay

WaveRelay 将 Android 手机的系统音频通过 USB 或局域网转发到电脑，在电脑的默认音频设备上播放。当前版本面向 Windows 和 Android 11 及以上版本。手机通过 ADB 临时运行音频采集服务，无需 Root 或安装常驻应用。

- **一个可执行文件**　双击启动桌面界面，也可以传入命令在终端使用。
- **USB 和 Wi-Fi**　界面支持刷新设备列表、选择设备和完成无线调试配对。
- **可调播放设置**　桌面版支持 PCM 无损、Opus 192 kbps 和 Opus 96 kbps。音量可在转发中调节，音质和缓冲在开始转发前设置。

## 快速开始

### 1. 准备依赖

1. 将 `waverelay.exe` 放到一个文件夹。也可以在项目根目录运行下文的构建命令。
2. 安装 Android SDK Platform-Tools，将 `platform-tools` 加到 `PATH`。在 PowerShell 中运行 `adb version` 确认能调用 ADB。[Android SDK Platform-Tools](https://developer.android.com/tools/releases/platform-tools)
3. 在 EXE 所在文件夹或项目根目录运行以下命令，下载匹配的 Android 服务。WaveRelay 使用固定版本的服务协议。[scrcpy v3.3.4](https://github.com/Genymobile/scrcpy/releases/tag/v3.3.4)

```powershell
New-Item -ItemType Directory -Force '.local' | Out-Null
Invoke-WebRequest 'https://github.com/Genymobile/scrcpy/releases/download/v3.3.4/scrcpy-server-v3.3.4' -OutFile '.local\scrcpy-server-v3.3.4'
Get-FileHash '.local\scrcpy-server-v3.3.4' -Algorithm SHA256
```

服务文件的 SHA-256 为 `8588238c9a5a00aa542906b6ec7e6d5541d9ffb9b5d0f6e1bc0e365e2303079e`。Windows 桌面界面使用 Microsoft Edge WebView2 Runtime。[Tauri 系统要求](https://v2.tauri.app/start/prerequisites/)

### 2. 连接手机

- **USB**　开启手机的 USB 调试，插入数据线并在手机上确认授权。
- **Wi-Fi**　让手机和电脑接入同一局域网，开启手机的无线调试。在 WaveRelay 的「网络连接」中，用配对弹窗显示的地址和六位码完成配对，再用无线调试主页显示的地址连接。两个端口通常不同。[Android 无线调试说明](https://developer.android.com/tools/adb#connect-to-a-device-over-wi-fi)

Android 11 启动转发时请先解锁屏幕。[scrcpy 音频转发说明](https://github.com/Genymobile/scrcpy/blob/v3.3.4/doc/audio.md)

### 3. 开始转发

双击 `waverelay.exe` 打开界面。选择已授权的 Android 设备，按需调整音质和缓冲，再点击「开始转发」。缓冲值表示目标播放缓冲，实际延迟还受手机采集、传输和电脑音频系统影响。

## 命令行

在包含 EXE 和 `.local` 的文件夹中执行以下命令。若从项目根目录运行，请将 `.\waverelay.exe` 替换为 `.\target\release\waverelay.exe`。

```powershell
.\waverelay.exe --help
.\waverelay.exe gui
.\waverelay.exe devices
.\waverelay.exe play
.\waverelay.exe play --serial '192.168.1.50:39111' --volume 75 --buffer-ms 100
```

`play` 使用 PCM。多台设备在线时，用 `--serial` 指定 `devices` 列出的序列号。按 Ctrl+C 停止转发。ADB 路径可用 `--adb` 或 `WAVERELAY_ADB` 指定。服务文件路径可用 `play --server` 指定，GUI 使用 `WAVERELAY_SERVER`。

## 从源码构建

安装 Rust stable、Visual Studio C++ 构建工具、Windows SDK 和 WebView2 Runtime，然后在项目根目录执行以下命令。[Rust 安装](https://rust-lang.org/tools/install/) · [Tauri Windows 构建要求](https://v2.tauri.app/start/prerequisites/#windows)

```powershell
cargo build --release --locked
.\target\release\waverelay.exe
```

Cargo 会生成 `target\release\waverelay.exe`。桌面界面、Rust 后端和命令行入口都包含在这个文件中。

```powershell
cargo fmt --check
cargo test --locked
```

## 项目结构

| 路径 | 用途 |
| --- | --- |
| `src/main.rs` | GUI 与命令行的统一入口 |
| `src/desktop.rs`、`src/ui/` | Tauri 窗口与桌面界面 |
| `src/adb.rs`、`src/relay.rs` | Android 连接、服务启动与转发会话 |
| `src/protocol.rs`、`src/audio.rs` | 音频数据解析、解码、缓冲与播放 |

## 验证范围

已在 Windows 上验证桌面启动、命令行入口，以及使用模拟设备进行无线配对、PCM 和 Opus 播放。Android 真机的 USB 和 Wi-Fi 转发仍待验证。

## 第三方组件

Android 音频采集由 Genymobile 的 scrcpy 服务提供，适用 scrcpy 的 Apache-2.0 许可证。服务文件由使用者从上游下载。[scrcpy 许可证](https://github.com/Genymobile/scrcpy/blob/v3.3.4/LICENSE)
