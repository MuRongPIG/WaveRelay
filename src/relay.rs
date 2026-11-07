use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

use anyhow::{Context, Result, bail, ensure};

use crate::adb::{self, Adb, Session};
use crate::audio::Playback;
use crate::protocol::{AudioQuality, AudioReader};

pub const DEFAULT_SERVER: &str = ".local/scrcpy-server-v3.3.4";

pub struct RelayOptions<'a> {
    pub serial: Option<&'a str>,
    pub server: &'a Path,
    pub buffer_ms: u32,
    pub quality: AudioQuality,
}

pub fn run_relay(
    adb: &Adb,
    options: RelayOptions<'_>,
    stop: Arc<AtomicBool>,
    volume: Arc<AtomicU32>,
    on_streaming: impl FnOnce(&str),
) -> Result<()> {
    let RelayOptions {
        serial,
        server,
        buffer_ms,
        quality,
    } = options;
    if stop.load(Ordering::Relaxed) {
        return Ok(());
    }
    ensure!(
        (20..=250).contains(&buffer_ms),
        "Playback buffer must be 20–250 ms"
    );
    ensure!(
        server.is_file(),
        "Missing scrcpy {} server at {}. See the project README.",
        adb::SERVER_VERSION,
        server.display()
    );
    let server =
        std::path::absolute(server).context("Could not resolve the Android server file")?;
    let serial = adb.select(serial)?;
    if stop.load(Ordering::Relaxed) {
        return Ok(());
    }

    let mut playback = Playback::start(buffer_ms, volume, stop.clone())?;
    let result = (|| {
        let mut session = Session::start(adb, &serial, &server, quality, &stop)?;
        let socket = session.open(&stop)?;
        let mut reader =
            AudioReader::new(socket, quality).context("Could not read the scrcpy audio header")?;
        let mut notify = Some(on_streaming);
        let mut frames = Vec::new();
        while reader
            .next_frames(&mut frames)
            .context("Could not read scrcpy audio")?
        {
            playback.push(frames.iter().copied());
            if let Some(on_streaming) = notify.take() {
                on_streaming(&serial);
            }
        }
        bail!("Android audio stream closed")
    })();
    if playback.failed() {
        bail!("Playback stopped because the desktop audio device failed");
    }
    if stop.load(Ordering::Relaxed) {
        return Ok(());
    }
    result
}
