use std::{
    env, fs,
    io::{self, Cursor, Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result, ensure};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{adb::Adb, relay::DEFAULT_SERVER};

const ADB_DIR: &str = "platform-tools-37.0.1";
const ADB_URL: &str = "https://dl.google.com/android/repository/platform-tools_r37.0.1-win.zip";
const ADB_SHA256: &str = "45f4d63113e895ebde0c90f194099a4676b6ac653bd28d54314a9e022bbc1a99";
const SERVER_FILE: &str = "scrcpy-server-v3.3.4";
const SERVER_URL: &str =
    "https://github.com/Genymobile/scrcpy/releases/download/v3.3.4/scrcpy-server-v3.3.4";
const SERVER_SHA256: &str = "8588238c9a5a00aa542906b6ec7e6d5541d9ffb9b5d0f6e1bc0e365e2303079e";
pub const ANDROID_SDK_TERMS: &str = "https://developer.android.com/studio/terms";
const ADB_FILES: &[&str] = &[
    "adb.exe",
    "AdbWinApi.dll",
    "AdbWinUsbApi.dll",
    "libwinpthread-1.dll",
    "NOTICE.txt",
    "source.properties",
];

#[derive(Clone, Copy, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Component {
    Adb,
    Scrcpy,
    All,
}

impl Component {
    pub fn includes_adb(self) -> bool {
        matches!(self, Self::Adb | Self::All)
    }
}

#[derive(Serialize)]
pub struct ToolStatus {
    pub adb: bool,
    pub scrcpy: bool,
}

#[derive(Clone)]
pub struct Tools {
    adb_override: Option<PathBuf>,
    root: Option<PathBuf>,
}

impl Tools {
    pub fn new(adb_override: Option<PathBuf>) -> Self {
        Self {
            adb_override,
            root: dirs::data_local_dir().map(|path| path.join("WaveRelay").join("tools")),
        }
    }

    pub fn adb(&self) -> Adb {
        let path = self
            .adb_override
            .clone()
            .or_else(|| {
                let directory = self.root.as_ref()?.join(ADB_DIR);
                adb_ready(&directory).then(|| directory.join("adb.exe"))
            })
            .unwrap_or_else(|| PathBuf::from("adb"));
        Adb::new(path)
    }

    pub fn server(&self) -> PathBuf {
        if let Some(path) = env::var_os("WAVERELAY_SERVER") {
            return path.into();
        }
        if let Some(path) = self
            .root
            .as_ref()
            .map(|root| root.join(SERVER_FILE))
            .filter(|path| path.is_file())
        {
            return path;
        }
        let relative = PathBuf::from(DEFAULT_SERVER);
        if relative.is_file() {
            return relative;
        }
        if let Ok(exe) = env::current_exe()
            && let Some(path) = exe
                .ancestors()
                .skip(1)
                .take(5)
                .map(|parent| parent.join(&relative))
                .find(|file| file.is_file())
        {
            return path;
        }
        relative
    }

    pub fn status(&self) -> ToolStatus {
        ToolStatus {
            adb: self.adb().available(),
            scrcpy: self.server().is_file(),
        }
    }

    pub fn install(&self, component: Component, accept_sdk_license: bool) -> Result<Vec<PathBuf>> {
        if component.includes_adb() {
            ensure!(
                accept_sdk_license,
                "Review and accept the Android SDK terms at {ANDROID_SDK_TERMS} before downloading ADB"
            );
            ensure!(
                cfg!(windows),
                "Managed Platform-Tools downloads are available on Windows"
            );
        }
        let root = self
            .root
            .as_deref()
            .context("Could not locate the local application data directory")?;
        fs::create_dir_all(root)?;
        let mut installed = Vec::new();
        if component.includes_adb() {
            installed.push(install_adb(root).context("Could not install Android Platform-Tools")?);
        }
        if matches!(component, Component::Scrcpy | Component::All) {
            installed
                .push(install_server(root).context("Could not install the scrcpy Android server")?);
        }
        Ok(installed)
    }
}

fn adb_ready(directory: &Path) -> bool {
    ADB_FILES.iter().all(|name| directory.join(name).is_file())
}

fn install_adb(root: &Path) -> Result<PathBuf> {
    let destination = root.join(ADB_DIR);
    if adb_ready(&destination) {
        return Ok(destination.join("adb.exe"));
    }
    ensure!(
        !destination.exists(),
        "The managed Platform-Tools directory is incomplete: {}",
        destination.display()
    );
    let bytes = download(ADB_URL, ADB_SHA256, 16 * 1024 * 1024)?;
    let staging = tempfile::tempdir_in(root)?;
    extract_adb(&bytes, staging.path())?;
    if let Err(error) = fs::rename(staging.path(), &destination)
        && !adb_ready(&destination)
    {
        return Err(error.into());
    }
    Ok(destination.join("adb.exe"))
}

fn extract_adb(bytes: &[u8], directory: &Path) -> Result<()> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    // Install only the fixed ADB file set. No archive paths are used as output paths.
    for name in ADB_FILES {
        let mut entry = archive.by_name(&format!("platform-tools/{name}"))?;
        ensure!(
            entry.is_file() && entry.size() <= 32 * 1024 * 1024,
            "Invalid archive entry {name}"
        );
        let mut file = fs::File::create(directory.join(name))?;
        io::copy(&mut entry, &mut file)?;
    }
    Ok(())
}

fn install_server(root: &Path) -> Result<PathBuf> {
    let destination = root.join(SERVER_FILE);
    if fs::metadata(&destination).is_ok_and(|metadata| metadata.len() <= 1024 * 1024)
        && let Ok(bytes) = fs::read(&destination)
        && digest_matches(&bytes, SERVER_SHA256)
    {
        return Ok(destination);
    }
    let bytes = download(SERVER_URL, SERVER_SHA256, 1024 * 1024)?;
    let mut staging = tempfile::NamedTempFile::new_in(root)?;
    staging.write_all(&bytes)?;
    staging.persist(&destination)?;
    Ok(destination)
}

fn download(url: &str, checksum: &str, limit: u64) -> Result<Vec<u8>> {
    let client = reqwest::blocking::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(90))
        .build()?;
    let response = client.get(url).send()?.error_for_status()?;
    let mut bytes = Vec::new();
    response.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "Download exceeds its expected size limit"
    );
    ensure!(
        digest_matches(&bytes, checksum),
        "Downloaded file failed SHA-256 verification"
    );
    Ok(bytes)
}

fn digest_matches(bytes: &[u8], expected: &str) -> bool {
    format!("{:x}", Sha256::digest(bytes)) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adb_extraction_keeps_only_required_files() {
        let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        for name in ADB_FILES {
            archive
                .start_file(format!("platform-tools/{name}"), options)
                .unwrap();
            archive.write_all(name.as_bytes()).unwrap();
        }
        archive.start_file("../outside.exe", options).unwrap();
        archive.write_all(b"unexpected").unwrap();
        let bytes = archive.finish().unwrap().into_inner();
        let directory = tempfile::tempdir().unwrap();
        let staging = directory.path().join("platform-tools");
        fs::create_dir(&staging).unwrap();
        extract_adb(&bytes, &staging).unwrap();
        assert!(adb_ready(&staging));
        assert_eq!(fs::read_dir(&staging).unwrap().count(), ADB_FILES.len());
        assert!(!directory.path().join("outside.exe").exists());
    }

    #[test]
    fn validates_download_checksum() {
        let digest = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(digest_matches(b"abc", digest));
        assert!(!digest_matches(b"abd", digest));
    }
}
