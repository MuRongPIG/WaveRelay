<p align="center">
  <img src="src/icons/icon.png" alt="" width="64" height="64">
</p>

<h1 align="center">WaveRelay</h1>

<p align="center">Stream Android system audio to Desktop over ADB.</p>

<p align="center">
  <strong>English</strong> · <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <a href="#features">Features</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#command-line">Command line</a> ·
  <a href="#build-from-source">Build from source</a>
</p>

WaveRelay plays audio from an Android phone on the PC's default audio device. Android Debug Bridge (ADB) starts a temporary capture service on the phone, so setup requires neither root access nor a permanently installed Android app.

## Features

- **USB and Wi-Fi** — discover authorized devices and pair over Wi-Fi from the desktop interface.
- **Adjustable audio** — choose lossless PCM or Opus at 96 or 192 kbps in the GUI. Set the target playback buffer before starting and adjust volume while streaming.
- **Compact desktop interface** — a window that fits its content, with light and dark themes.
- **One executable** — launch the GUI by double-clicking `waverelay.exe`, or pass a subcommand to use it from a terminal.

## Quick start

### Requirements

- A Windows PC with Microsoft Edge WebView2 Runtime for the GUI. [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#windows)
- A phone running Android 11 or later with USB debugging or wireless debugging enabled.
- ADB from Android SDK Platform-Tools. Add `platform-tools` to `PATH` and check that `adb version` runs. [Android SDK Platform-Tools](https://developer.android.com/tools/releases/platform-tools)

### 1. Prepare WaveRelay

Place `waverelay.exe` in a folder or [build it from source](#build-from-source). In the folder containing the EXE, or at the repository root for a source build, download the matching Android service. The checksum check uses the expected SHA-256 for the scrcpy v3.3.4 server. [scrcpy v3.3.4 release](https://github.com/Genymobile/scrcpy/releases/tag/v3.3.4)

```powershell
$server = '.local\scrcpy-server-v3.3.4'
$expected = '8588238c9a5a00aa542906b6ec7e6d5541d9ffb9b5d0f6e1bc0e365e2303079e'
New-Item -ItemType Directory -Force '.local' | Out-Null
Invoke-WebRequest -Uri 'https://github.com/Genymobile/scrcpy/releases/download/v3.3.4/scrcpy-server-v3.3.4' -OutFile $server
if ((Get-FileHash -LiteralPath $server -Algorithm SHA256).Hash -ne $expected) {
    throw 'scrcpy server checksum mismatch.'
}
```

### 2. Connect and play

- **USB** — connect the phone, approve the debugging prompt on Android, and refresh the USB device list.
- **Wi-Fi** — connect the phone and PC to the same local network. Open **网络连接** (Network) in WaveRelay. For first-time pairing, enter the address and six-digit code from Android's pairing dialog. To connect, use the address shown on the main **Wireless debugging** screen. The two ports are usually different. [Android wireless debugging](https://developer.android.com/tools/adb#connect-to-a-device-over-wi-fi)

Select the device and click **开始转发** (Start streaming). Keep the phone unlocked when starting audio capture on Android 11. [scrcpy audio documentation](https://github.com/Genymobile/scrcpy/blob/v3.3.4/doc/audio.md)

## Command line

Run these commands from the folder containing the EXE and `.local`. From the repository root, use `.\target\release\waverelay.exe` in place of `.\waverelay.exe`. The network address below is an example.

```powershell
.\waverelay.exe --help
.\waverelay.exe gui
.\waverelay.exe devices
.\waverelay.exe play
.\waverelay.exe play --serial '192.0.2.10:39111' --volume 75 --buffer-ms 100
```

- `play` streams PCM. When multiple devices are online, pass the serial shown by `devices` to `--serial`. Press Ctrl+C to stop.
- `--buffer-ms` sets the target playback buffer in milliseconds. Capture, transport, and the audio device also contribute to end-to-end latency.
- Use `--adb` or `WAVERELAY_ADB` to select an ADB executable. Override the Android service path with `play --server` for the CLI or `WAVERELAY_SERVER` for the GUI.

### Wi-Fi from the terminal

For a first-time connection, use the pairing dialog address with `adb pair` and enter the six-digit code at the prompt. Then use the wireless debugging screen address with `adb connect`. Replace the example addresses with the ones shown on your phone.

```powershell
adb pair 192.0.2.10:37099
adb connect 192.0.2.10:39111
```

## Build from source

On Windows, install stable Rust, the Visual Studio C++ build tools, the Windows SDK, and WebView2 Runtime. [Rust installation](https://rust-lang.org/tools/install/) · [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#windows)

```powershell
git clone https://github.com/MuRongPIG/WaveRelay.git
Set-Location WaveRelay
cargo build --release --locked
```

The build produces `target\release\waverelay.exe`. Prepare the Android service from the repository root as shown in [Quick start](#quick-start), then run `.\target\release\waverelay.exe`.

Run the following checks from the repository root.

```powershell
cargo fmt --check
cargo test --locked
```

## Project layout

| Path | Purpose |
| --- | --- |
| [`src/main.rs`](src/main.rs) | Shared executable entry point for the GUI and CLI |
| [`src/desktop.rs`](src/desktop.rs), [`src/ui/`](src/ui/) | Tauri window, desktop controls, and interface assets |
| [`src/adb.rs`](src/adb.rs), [`src/relay.rs`](src/relay.rs) | Device discovery, Android service startup, and relay sessions |
| [`src/protocol.rs`](src/protocol.rs), [`src/audio.rs`](src/audio.rs) | Stream framing, decoding, buffering, and playback |

## Project status and feedback

The Windows GUI and CLI have been tested. Simulated-device tests cover wireless pairing and PCM/Opus playback. USB and Wi-Fi streaming with physical Android devices still need validation.

When filing a report, include your Windows version, Android version, USB or Wi-Fi connection mode, and steps to reproduce the issue. [GitHub Issues](https://github.com/MuRongPIG/WaveRelay/issues)

## Licensing and acknowledgments

WaveRelay's source license is pending. The separately downloaded Android capture service comes from Genymobile's scrcpy and is licensed under Apache-2.0. [scrcpy license](https://github.com/Genymobile/scrcpy/blob/v3.3.4/LICENSE)
