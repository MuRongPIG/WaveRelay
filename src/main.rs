mod adb;
mod audio;
mod desktop;
mod language;
mod protocol;
mod relay;
mod tools;

use std::{
    env,
    ffi::OsString,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

use crate::{
    language::Language,
    relay::{RelayOptions, run_relay},
    tools::{Component, Tools},
};
use anyhow::{Context, Result};
use clap::{Command, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to Android Platform Tools' adb executable.
    #[arg(long, global = true, env = "WAVERELAY_ADB")]
    adb: Option<PathBuf>,
    /// Language for CLI messages or this GUI session.
    #[arg(long, global = true, value_enum, env = "WAVERELAY_LANG")]
    lang: Option<Language>,
    #[command(subcommand)]
    command: Option<Action>,
}

#[derive(Subcommand)]
enum Action {
    /// Open the WaveRelay window (also opens when no command is supplied).
    Gui,
    /// List Android devices visible to ADB, including unauthorized devices.
    Devices,
    /// Download and verify ADB or the scrcpy Android server.
    Setup {
        /// Component to install into WaveRelay's local tools directory.
        #[arg(value_enum, default_value = "all")]
        component: Component,
        /// Accept the Android SDK terms at https://developer.android.com/studio/terms for the ADB download.
        #[arg(long = "accept-android-sdk-license")]
        accept_sdk_license: bool,
    },
    /// Relay Android system audio to the default desktop output.
    Play {
        /// Select a USB or Wi-Fi device from the devices list.
        #[arg(long)]
        serial: Option<String>,
        /// Official scrcpy Android server file for the supported version.
        #[arg(long)]
        server: Option<PathBuf>,
        /// Target playback buffer in milliseconds.
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(20..=250))]
        buffer_ms: u32,
        /// Output volume in percent.
        #[arg(long, default_value_t = 100, value_parser = clap::value_parser!(u32).range(0..=100))]
        volume: u32,
    },
}

fn help_language(args: &[OsString]) -> Language {
    // Clap can exit while displaying help, so choose its language before parsing.
    let arguments = &args[1..];
    let requested = arguments
        .iter()
        .enumerate()
        .take_while(|(_, arg)| *arg != "--")
        .find_map(|(index, arg)| {
            let arg = arg.to_str()?;
            if arg == "--lang" {
                arguments.get(index + 1)?.to_str()
            } else {
                arg.strip_prefix("--lang=")
            }
        });
    let environment = env::var("WAVERELAY_LANG").ok();
    requested
        .or(environment.as_deref())
        .and_then(|value| Language::from_str(value, false).ok())
        .unwrap_or_default()
}

fn command_for(language: Language) -> Command {
    let command = Cli::command();
    if language == Language::En {
        return command;
    }
    command
        .about("通过 ADB 将 Android 系统音频转发到电脑")
        .mut_arg("adb", |arg| {
            arg.help("指定 Android Platform-Tools 的 adb 程序路径")
        })
        .mut_arg("lang", |arg| {
            arg.help("指定命令行语言或本次 GUI 的初始语言")
        })
        .mut_subcommand("gui", |cmd| cmd.about("打开 WaveRelay 窗口"))
        .mut_subcommand("devices", |cmd| cmd.about("列出 ADB 检测到的 Android 设备"))
        .mut_subcommand("setup", |cmd| {
            cmd.about("下载并校验 ADB 或 scrcpy Android 服务")
                .mut_arg("component", |arg| arg.help("要安装的组件"))
                .mut_arg("accept_sdk_license", |arg| arg.help("下载 ADB 前接受 https://developer.android.com/studio/terms 的 Android SDK 条款"))
        })
        .mut_subcommand("play", |cmd| {
            cmd.about("将 Android 音频转发到电脑的默认输出设备")
                .mut_arg("serial", |arg| arg.help("指定 devices 列出的设备序列号"))
                .mut_arg("server", |arg| {
                    arg.help("指定配套的 scrcpy Android 服务文件")
                })
                .mut_arg("buffer_ms", |arg| arg.help("设置目标播放缓冲，单位为毫秒"))
                .mut_arg("volume", |arg| arg.help("设置输出音量百分比"))
        })
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().collect();
    let cli = Cli::from_arg_matches(&command_for(help_language(&args)).get_matches_from(args))?;
    let language = cli.lang.unwrap_or_default();
    let tools = Tools::new(cli.adb);
    match cli.command {
        None | Some(Action::Gui) => Ok(desktop::run(tools, cli.lang)?),
        Some(Action::Devices) => {
            let devices = tools
                .adb()
                .devices()
                .context(language.text("Could not read ADB devices", "无法读取 ADB 设备列表"))?;
            if devices.is_empty() {
                println!(
                    "{}", language.text("No devices found. Connect an Android device with USB debugging or pair it over Wi-Fi.", "未找到设备。请连接已开启 USB 调试的 Android 设备，或通过 Wi-Fi 配对。")
                );
            }
            for device in devices {
                println!("{}\t{}", device.serial, device.state);
            }
            Ok(())
        }
        Some(Action::Setup {
            component,
            accept_sdk_license,
        }) => {
            for path in tools
                .install(component, accept_sdk_license)
                .context(language.text("Dependency setup failed", "依赖安装失败"))?
            {
                println!("{} {}", language.text("Ready:", "已就绪"), path.display());
            }
            Ok(())
        }
        Some(Action::Play {
            serial,
            server,
            buffer_ms,
            volume,
        }) => {
            let adb = tools.adb();
            let server = server.unwrap_or_else(|| tools.server());
            let stop = Arc::new(AtomicBool::new(false));
            let interrupt = stop.clone();
            ctrlc::set_handler(move || interrupt.store(true, Ordering::Relaxed))?;
            run_relay(
                &adb,
                RelayOptions {
                    serial: serial.as_deref(),
                    server: &server,
                    buffer_ms,
                    quality: Default::default(),
                },
                stop.clone(),
                Arc::new(AtomicU32::new(volume)),
                |serial| match language {
                    Language::En => {
                        eprintln!("Relaying Android audio from {serial}. Press Ctrl+C to stop.")
                    }
                    Language::ZhCn => {
                        eprintln!("正在转发 {serial} 的 Android 音频。按 Ctrl+C 停止。")
                    }
                },
            )
            .context(language.text("Audio relay failed", "音频转发失败"))?;
            if stop.load(Ordering::Relaxed) {
                eprintln!("{}", language.text("Relay stopped.", "转发已停止。"));
            }
            Ok(())
        }
    }
}
