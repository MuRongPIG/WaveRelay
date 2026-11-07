use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    thread,
};

use crate::{
    language::Language,
    protocol::AudioQuality,
    relay::{RelayOptions, run_relay},
    tools::{Component, ToolStatus, Tools},
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, PhysicalSize, State, Window, WindowEvent};

#[derive(Clone, Copy, Default, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Phase {
    #[default]
    Idle,
    Connecting,
    Streaming,
    Stopping,
    Error,
}

#[derive(Clone, Default, Serialize)]
struct Status {
    phase: Phase,
    serial: Option<String>,
    error: Option<String>,
}

struct ActiveRelay {
    stop: Arc<AtomicBool>,
    volume: Arc<AtomicU32>,
}

#[derive(Default)]
struct SessionState {
    active: Option<ActiveRelay>,
    status: Status,
    closing: bool,
}

impl SessionState {
    fn stop(&mut self) {
        if let Some(relay) = &self.active {
            self.status.phase = Phase::Stopping;
            relay.stop.store(true, Ordering::Relaxed);
        }
    }
}

struct Controller {
    session: Arc<Mutex<SessionState>>,
    tools: Tools,
    language: Option<Language>,
}

#[derive(Serialize)]
struct Device {
    serial: String,
    state: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    serial: String,
    buffer_ms: u32,
    volume: u32,
    quality: AudioQuality,
}

#[tauri::command]
async fn list_devices(controller: State<'_, Controller>) -> Result<Vec<Device>, String> {
    let adb = controller.tools.adb();
    tauri::async_runtime::spawn_blocking(move || {
        adb.devices()
            .map(|devices| {
                devices
                    .into_iter()
                    .map(|device| Device {
                        serial: device.serial,
                        state: device.state,
                    })
                    .collect()
            })
            .map_err(|error| format!("{error:#}"))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn wireless_device(
    address: String,
    pairing_code: Option<String>,
    language: Language,
    controller: State<'_, Controller>,
) -> Result<String, String> {
    let adb = controller.tools.adb();
    tauri::async_runtime::spawn_blocking(move || {
        adb.wireless(&address, pairing_code.as_deref(), language)
            .map_err(|error| format!("{error:#}"))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn dependency_status(controller: State<'_, Controller>) -> Result<ToolStatus, String> {
    let tools = controller.tools.clone();
    tauri::async_runtime::spawn_blocking(move || tools.status())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn install_dependency(
    component: Component,
    accept_sdk_license: bool,
    controller: State<'_, Controller>,
) -> Result<(), String> {
    let tools = controller.tools.clone();
    tauri::async_runtime::spawn_blocking(move || {
        tools
            .install(component, accept_sdk_license)
            .map(|_| ())
            .map_err(|error| format!("{error:#}"))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn gui_language(controller: State<'_, Controller>) -> Option<Language> {
    controller.language
}

#[tauri::command]
fn relay_status(controller: State<'_, Controller>) -> Status {
    controller.session.lock().unwrap().status.clone()
}

#[tauri::command]
fn start_relay(
    settings: Settings,
    language: Language,
    controller: State<'_, Controller>,
    app: AppHandle,
) -> Result<(), String> {
    if settings.serial.is_empty()
        || !(20..=250).contains(&settings.buffer_ms)
        || settings.volume > 100
    {
        return Err(language
            .text(
                "Select a device and check the buffer and volume settings.",
                "请选择设备，并检查缓冲和音量设置。",
            )
            .into());
    }
    let shared = controller.session.clone();
    let mut state = controller.session.lock().unwrap();
    if state.active.is_some() || state.closing {
        return Err(language
            .text(
                "A relay session is already active.",
                "当前转发会话仍在运行。",
            )
            .into());
    }

    let stop = Arc::new(AtomicBool::new(false));
    let volume = Arc::new(AtomicU32::new(settings.volume));
    let adb = controller.tools.adb();
    let server = controller.tools.server();
    let worker_stop = stop.clone();
    let worker_volume = volume.clone();
    let serial = settings.serial.clone();
    thread::Builder::new()
        .name("waverelay-audio".into())
        .spawn(move || {
            let options = RelayOptions {
                serial: Some(&settings.serial),
                server: &server,
                buffer_ms: settings.buffer_ms,
                quality: settings.quality,
            };
            let result = run_relay(&adb, options, worker_stop, worker_volume, |_| {
                let mut state = shared.lock().unwrap();
                if state.status.phase == Phase::Connecting {
                    state.status.phase = Phase::Streaming;
                }
            });
            let closing = {
                let mut state = shared.lock().unwrap();
                let stopping = state.status.phase == Phase::Stopping;
                state.status = match result {
                    Err(error) if !stopping => Status {
                        phase: Phase::Error,
                        serial: Some(settings.serial),
                        error: Some(format!("{error:#}")),
                    },
                    _ => Status::default(),
                };
                state.active = None;
                state.closing
            };
            if closing {
                app.exit(0);
            }
        })
        .map_err(|error| {
            format!(
                "{} {error}",
                language.text("Could not start the relay thread.", "无法启动转发线程。")
            )
        })?;
    state.active = Some(ActiveRelay { stop, volume });
    state.status = Status {
        phase: Phase::Connecting,
        serial: Some(serial),
        error: None,
    };
    Ok(())
}

#[tauri::command]
fn stop_relay(controller: State<'_, Controller>) {
    controller.session.lock().unwrap().stop();
}

#[tauri::command]
fn set_volume(volume: u32, controller: State<'_, Controller>) {
    if let Some(relay) = &controller.session.lock().unwrap().active {
        relay.volume.store(volume.min(100), Ordering::Relaxed);
    }
}

#[tauri::command]
fn fit_window_height(window: Window, height: u32) -> tauri::Result<()> {
    let current = window.inner_size()?;
    let scale = window.scale_factor()?;
    let mut height = (f64::from(height.max(1)) * scale).ceil() as u32;
    if let Some(monitor) = window.current_monitor()? {
        let area = monitor.work_area();
        let frame = window.outer_size()?.height.saturating_sub(current.height);
        height = height.min(area.size.height.saturating_sub(frame).max(1));

        if height > current.height {
            let mut position = window.outer_position()?;
            let top = i64::from(area.position.y);
            let bottom = top + i64::from(area.size.height);
            let last_top = bottom - i64::from(height) - i64::from(frame);
            let y = i64::from(position.y).clamp(top, last_top.max(top)) as i32;
            if y != position.y {
                position.y = y;
                window.set_position(position)?;
            }
        }
    }

    if height != current.height {
        window.set_size(PhysicalSize::new(current.width, height))?;
    }
    Ok(())
}

pub fn run(tools: Tools, language: Option<Language>) -> tauri::Result<()> {
    #[cfg(windows)]
    detach_private_console();

    tauri::Builder::default()
        .manage(Controller {
            session: Arc::new(Mutex::new(SessionState::default())),
            tools,
            language,
        })
        .invoke_handler(tauri::generate_handler![
            gui_language,
            dependency_status,
            install_dependency,
            list_devices,
            wireless_device,
            relay_status,
            start_relay,
            stop_relay,
            fit_window_height,
            set_volume
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let controller = window.state::<Controller>();
                let mut state = controller.session.lock().unwrap();
                if state.active.is_some() {
                    api.prevent_close();
                    state.closing = true;
                    state.stop();
                }
            }
        })
        .run(tauri::generate_context!())
}

#[cfg(windows)]
fn detach_private_console() {
    use windows_sys::Win32::System::Console::{FreeConsole, GetConsoleProcessList};

    let mut process = 0;
    // SAFETY: The buffer holds one process ID. Detach only when this process is the
    // sole console user, as with the console created for a launch from Explorer.
    unsafe {
        if GetConsoleProcessList(&mut process, 1) == 1 && process == std::process::id() {
            FreeConsole();
        }
    }
}
