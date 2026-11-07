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
- **Compact desktop interface** — a window that fits its content, with English and Simplified Chinese, plus light and dark themes.
- **One executable** — launch the GUI by double-clicking `waverelay.exe`, or pass a subcommand to use it from a terminal.

## Quick start

### Requirements

- A Windows PC with Microsoft Edge WebView2 Runtime for the GUI. [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#windows)
- A phone running Android 11 or later with USB debugging or wireless debugging enabled.

### 1. Run WaveRelay and set up its dependencies

Open `waverelay.exe`. A [source build](#build-from-source) puts the executable in `target\release`, where you can run it directly. Expand **Dependencies** in the app to download missing components.

From a terminal in the EXE folder, choose the setup command for the components you need. To install both ADB and the scrcpy Android service, review and accept the [Android SDK terms](https://developer.android.com/studio/terms), then run the following command.

```powershell
.\waverelay.exe setup all --accept-android-sdk-license
```

To install just the scrcpy Android service, use this command.

```powershell
.\waverelay.exe setup scrcpy
```

WaveRelay downloads the ADB files from Google's Windows Platform-Tools 37.0.1 archive and the Android service from scrcpy v3.3.4. It verifies each download against a pinned SHA-256 and installs it under `%LOCALAPPDATA%\WaveRelay\tools`. The app reuses installed files on subsequent launches. [Platform-Tools release notes](https://developer.android.com/tools/releases/platform-tools) · [scrcpy v3.3.4 release](https://github.com/Genymobile/scrcpy/releases/tag/v3.3.4)

<details>
<summary>Manual setup</summary>

Add an existing Platform-Tools directory to `PATH` or pass `--adb` with its `adb.exe` path. Place the v3.3.4 Android service in `.local\scrcpy-server-v3.3.4` next to the EXE or in the repository root. Its SHA-256 is `8588238c9a5a00aa542906b6ec7e6d5541d9ffb9b5d0f6e1bc0e365e2303079e`. [Official Android service download](https://github.com/Genymobile/scrcpy/releases/download/v3.3.4/scrcpy-server-v3.3.4)

</details>

### 2. Connect and play

- **USB** — connect the phone, approve the debugging prompt on Android, and refresh the USB device list.
- **Wi-Fi** — connect the phone and PC to the same local network. Open **Network** in WaveRelay. For first-time pairing, enter the address and six-digit code from Android's pairing dialog. To connect, use the address shown on the main **Wireless debugging** screen. The two ports are usually different. [Android wireless debugging](https://developer.android.com/tools/adb#connect-to-a-device-over-wi-fi)

Select the device and click **Start streaming**. Keep the phone unlocked when starting audio capture on Android 11. [scrcpy audio documentation](https://github.com/Genymobile/scrcpy/blob/v3.3.4/doc/audio.md)

Use the **中文 / EN** button in the title bar to switch the interface language. The GUI follows the system language on first launch and remembers a manual selection.

## Command line

Run these commands from the folder containing the EXE. From the repository root, use `.\target\release\waverelay.exe` in place of `.\waverelay.exe`. The network address below is an example.

```powershell
.\waverelay.exe --help
.\waverelay.exe --lang zh-CN --help
.\waverelay.exe gui
.\waverelay.exe devices
.\waverelay.exe play
.\waverelay.exe play --serial '192.0.2.10:39111' --volume 75 --buffer-ms 100
```

- `play` streams PCM. When multiple devices are online, pass the serial shown by `devices` to `--serial`. Press Ctrl+C to stop.
- `--buffer-ms` sets the target playback buffer in milliseconds. Capture, transport, and the audio device also contribute to end-to-end latency.
- Use `--adb` or `WAVERELAY_ADB` to select an ADB executable. WaveRelay otherwise prefers its managed ADB and falls back to `PATH`. For the Android service, `play --server` takes precedence, followed by `WAVERELAY_SERVER`, the managed copy, and `.local`.

Use `--lang en` or `--lang zh-CN` for command descriptions and application messages. The CLI defaults to English. You can set `WAVERELAY_LANG` as a default or use `gui --lang en` to choose the initial language for one GUI session. ADB and operating-system diagnostics may appear in their original language.

### Wi-Fi from the terminal

For a first-time connection, use the pairing dialog address with `adb pair` and enter the six-digit code at the prompt. Then use the wireless debugging screen address with `adb connect`. Replace the example addresses with the ones shown on your phone. If you already have ADB on `PATH`, you can use that executable instead.

```powershell
$adb = Join-Path $env:LOCALAPPDATA 'WaveRelay\tools\platform-tools-37.0.1\adb.exe'
& $adb pair 192.0.2.10:37099
& $adb connect 192.0.2.10:39111
```

## Build from source

Install stable Rust with the MSVC toolchain, the Visual Studio C++ build tools, the Windows SDK, and WebView2 Runtime. [Rust installation](https://rust-lang.org/tools/install/) · [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/#windows)

```powershell
git clone https://github.com/MuRongPIG/WaveRelay.git
Set-Location WaveRelay
cargo build --release --locked
```

If you already have a checkout, run `cargo build --release --locked` from its root. The build produces one executable for the GUI and CLI at `target\release\waverelay.exe`. The Windows x64 release build and tests were verified with Rust 1.99.1 and the `x86_64-pc-windows-msvc` toolchain.

From the repository root, check the version and setup options, then launch the GUI.

```powershell
.\target\release\waverelay.exe --version
.\target\release\waverelay.exe setup --help
.\target\release\waverelay.exe gui
```

Use [Quick start](#quick-start) to configure runtime dependencies after building. When running its setup commands from the repository root, use `.\target\release\waverelay.exe` as the executable path.

Run the following checks from the repository root.

```powershell
cargo fmt --check
cargo test --release --locked
```

## Project layout

| Path | Purpose |
| --- | --- |
| [`src/main.rs`](src/main.rs) | Shared executable entry point for the GUI and CLI |
| [`src/desktop.rs`](src/desktop.rs), [`src/ui/`](src/ui/) | Tauri window, desktop controls, and interface assets |
| [`src/tools.rs`](src/tools.rs) | Download, checksum verification, and local dependency lookup |
| [`src/adb.rs`](src/adb.rs), [`src/relay.rs`](src/relay.rs) | Device discovery, Android service startup, and relay sessions |
| [`src/protocol.rs`](src/protocol.rs), [`src/audio.rs`](src/audio.rs) | Stream framing, decoding, buffering, and playback |

## Project status and feedback

The Windows GUI and CLI have been tested. Simulated-device tests cover wireless pairing and PCM/Opus playback. USB and Wi-Fi streaming with physical Android devices still need validation.

When filing a report, include your Windows version, Android version, USB or Wi-Fi connection mode, and steps to reproduce the issue. [GitHub Issues](https://github.com/MuRongPIG/WaveRelay/issues)

## Licensing and acknowledgments

WaveRelay's source is licensed under the [GNU General Public License v3.0](LICENSE). The package uses the SPDX identifier `GPL-3.0-only`. [SPDX license entry](https://spdx.org/licenses/GPL-3.0-only.html)

The separately downloaded Android capture service comes from Genymobile's scrcpy and is licensed under Apache-2.0. [scrcpy license](https://github.com/Genymobile/scrcpy/blob/v3.3.4/LICENSE) ADB is downloaded from Google under the applicable Android SDK terms and third-party notices. [Android SDK terms](https://developer.android.com/studio/terms)

When distributing the Windows EXE, follow GPLv3's corresponding-source requirements and include the license notices for the third-party components you distribute. For any included MPL-2.0 code, tell recipients how to obtain its corresponding source. [GPLv3 section 6](https://opensource.org/license/gpl-3.0) · [Mozilla MPL FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/)
