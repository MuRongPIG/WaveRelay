mod adb;
mod audio;
mod desktop;
mod protocol;
mod relay;

use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

use crate::{
    adb::Adb,
    relay::{DEFAULT_SERVER, RelayOptions, run_relay},
};
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to Android Platform Tools' adb executable.
    #[arg(long, global = true, default_value = "adb", env = "WAVERELAY_ADB")]
    adb: PathBuf,
    #[command(subcommand)]
    command: Option<Action>,
}

#[derive(Subcommand)]
enum Action {
    /// Open the WaveRelay window (also opens when no command is supplied).
    Gui,
    /// List Android devices visible to ADB, including unauthorized devices.
    Devices,
    /// Relay Android system audio to the default desktop output.
    Play {
        /// Select a USB or Wi-Fi device from the devices list.
        #[arg(long)]
        serial: Option<String>,
        /// Official scrcpy Android server file for the supported version.
        #[arg(long, default_value = DEFAULT_SERVER)]
        server: PathBuf,
        /// Target playback buffer in milliseconds.
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(20..=250))]
        buffer_ms: u32,
        /// Output volume in percent.
        #[arg(long, default_value_t = 100, value_parser = clap::value_parser!(u32).range(0..=100))]
        volume: u32,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let adb = Adb::new(cli.adb);
    match cli.command {
        None | Some(Action::Gui) => Ok(desktop::run(adb)?),
        Some(Action::Devices) => {
            let devices = adb.devices()?;
            if devices.is_empty() {
                println!(
                    "No devices found. Connect an Android device with USB debugging or pair it over Wi-Fi."
                );
            }
            for device in devices {
                println!("{}\t{}", device.serial, device.state);
            }
            Ok(())
        }
        Some(Action::Play {
            serial,
            server,
            buffer_ms,
            volume,
        }) => {
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
                |serial| eprintln!("Relaying Android audio from {serial}. Press Ctrl+C to stop."),
            )?;
            if stop.load(Ordering::Relaxed) {
                eprintln!("Relay stopped.");
            }
            Ok(())
        }
    }
}
