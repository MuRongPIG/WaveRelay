<p align="center">
  <img src="src/icons/icon.png" alt="" width="64" height="64">
</p>

<h1 align="center">WaveRelay</h1>

<p align="center">通过 ADB 将 Android 系统音频转发到电脑。</p>

<p align="center">
  <a href="README.md">English</a> · <strong>简体中文</strong>
</p>

<p align="center">
  <a href="#功能">功能</a> ·
  <a href="#快速开始">快速开始</a> ·
  <a href="#命令行">命令行</a> ·
  <a href="#源码构建">源码构建</a>
</p>

WaveRelay 将 Android 手机的音频转发到电脑的默认音频设备。Android 调试桥（ADB）会在手机上临时启动音频采集服务，无需 Root 或安装常驻应用。

## 功能

- **USB 和 Wi-Fi**　查找已授权的设备，并在桌面界面中完成无线配对。
- **可调播放设置**　在 GUI 中选择 PCM 无损传输或 96、192 kbps Opus，启动前设置目标缓冲，转发时调节音量。
- **紧凑的桌面界面**　窗口高度随内容调整，支持浅色和深色主题。
- **单个可执行文件**　双击 `waverelay.exe` 打开桌面界面，也可以传入子命令在终端使用。

## 快速开始

### 系统要求

- Windows 电脑。桌面界面使用 Microsoft Edge WebView2 Runtime。 [Tauri 系统要求](https://v2.tauri.app/start/prerequisites/#windows)
- 运行 Android 11 或以上版本的手机，已开启 USB 调试或无线调试。
- Android SDK Platform-Tools 中的 ADB。将 `platform-tools` 加入 `PATH`，并确认 `adb version` 可以运行。 [Android SDK Platform-Tools](https://developer.android.com/tools/releases/platform-tools)

### 1. 准备 WaveRelay

将 `waverelay.exe` 放到一个文件夹，或按下文[从源码构建](#源码构建)。在 EXE 所在目录或源码仓库根目录下载对应的 Android 服务。以下命令会校验 scrcpy v3.3.4 服务文件的 SHA-256。 [scrcpy v3.3.4 发布页](https://github.com/Genymobile/scrcpy/releases/tag/v3.3.4)

```powershell
$server = '.local\scrcpy-server-v3.3.4'
$expected = '8588238c9a5a00aa542906b6ec7e6d5541d9ffb9b5d0f6e1bc0e365e2303079e'
New-Item -ItemType Directory -Force '.local' | Out-Null
Invoke-WebRequest -Uri 'https://github.com/Genymobile/scrcpy/releases/download/v3.3.4/scrcpy-server-v3.3.4' -OutFile $server
if ((Get-FileHash -LiteralPath $server -Algorithm SHA256).Hash -ne $expected) {
    throw 'scrcpy server checksum mismatch.'
}
```

### 2. 连接并播放

- **USB**　连接手机，在 Android 上确认调试授权，然后刷新 USB 设备列表。
- **Wi-Fi**　让手机和电脑接入同一局域网，在 WaveRelay 中打开「网络连接」。首次配对时使用 Android 配对弹窗中的地址和六位码。连接时使用「无线调试」主页显示的地址。两个端口通常不同。 [Android 无线调试说明](https://developer.android.com/tools/adb#connect-to-a-device-over-wi-fi)

选择设备并点击「开始转发」。Android 11 在启动音频采集时需要保持手机解锁。 [scrcpy 音频说明](https://github.com/Genymobile/scrcpy/blob/v3.3.4/doc/audio.md)

## 命令行

在包含 EXE 和 `.local` 的目录中运行以下命令。如果从仓库根目录运行，将 `.\waverelay.exe` 替换为 `.\target\release\waverelay.exe`。以下网络地址仅用于示例。

```powershell
.\waverelay.exe --help
.\waverelay.exe gui
.\waverelay.exe devices
.\waverelay.exe play
.\waverelay.exe play --serial '192.0.2.10:39111' --volume 75 --buffer-ms 100
```

- `play` 使用 PCM。多台设备在线时，用 `--serial` 指定 `devices` 显示的序列号。按 Ctrl+C 停止。
- `--buffer-ms` 设置目标播放缓冲，单位为毫秒。实际端到端延迟还受采集、传输和输出设备影响。
- 可用 `--adb` 或 `WAVERELAY_ADB` 指定 ADB。CLI 通过 `play --server` 指定 Android 服务文件，GUI 通过 `WAVERELAY_SERVER` 指定。

### 在终端连接 Wi-Fi 设备

首次连接时，用 `adb pair` 连接配对弹窗的地址，按提示输入六位码，再用 `adb connect` 连接无线调试主页的地址。请将下列示例地址替换为手机显示的实际地址。

```powershell
adb pair 192.0.2.10:37099
adb connect 192.0.2.10:39111
```

## 源码构建

在 Windows 上安装 Rust stable、Visual Studio C++ 构建工具、Windows SDK 和 WebView2 Runtime。 [Rust 安装](https://rust-lang.org/tools/install/) · [Tauri 系统要求](https://v2.tauri.app/start/prerequisites/#windows)

```powershell
git clone https://github.com/MuRongPIG/WaveRelay.git
Set-Location WaveRelay
cargo build --release --locked
```

构建产物位于 `target\release\waverelay.exe`。按[快速开始](#快速开始)在仓库根目录准备 Android 服务，再运行 `.\target\release\waverelay.exe`。

在仓库根目录检查格式并运行测试。

```powershell
cargo fmt --check
cargo test --locked
```

## 项目结构

| 路径 | 用途 |
| --- | --- |
| [`src/main.rs`](src/main.rs) | 桌面界面和命令行的统一入口 |
| [`src/desktop.rs`](src/desktop.rs)、[`src/ui/`](src/ui/) | Tauri 窗口、桌面操作和界面资源 |
| [`src/adb.rs`](src/adb.rs)、[`src/relay.rs`](src/relay.rs) | 设备发现、Android 服务启动和转发会话 |
| [`src/protocol.rs`](src/protocol.rs)、[`src/audio.rs`](src/audio.rs) | 音频数据解析、解码、缓冲和播放 |

## 项目状态与反馈

已验证 Windows 桌面和命令行入口，并使用模拟设备测试了无线配对、PCM 和 Opus 播放。Android 真机的 USB 和 Wi-Fi 转发仍待验证。

提交反馈时，请附上 Windows 版本、Android 版本、USB 或 Wi-Fi 连接方式和复现步骤。 [GitHub Issues](https://github.com/MuRongPIG/WaveRelay/issues)

## 许可证与致谢

WaveRelay 自身的源码许可证待确定。独立下载的 Android 音频采集服务来自 Genymobile 的 scrcpy，采用 Apache-2.0。 [scrcpy 许可证](https://github.com/Genymobile/scrcpy/blob/v3.3.4/LICENSE)
