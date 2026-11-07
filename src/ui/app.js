const invoke = window.__TAURI__.core.invoke;
const byId = (id) => document.getElementById(id);
const device = byId("device");
const refresh = byId("refresh");
const usb = byId("mode-usb");
const networkMode = byId("mode-network");
const network = byId("network");
const pairForm = byId("pair-form");
const connectForm = byId("connect-form");
const networkStatus = byId("network-status");
const quality = byId("quality");
const volume = byId("volume");
const buffer = byId("buffer");
const toggle = byId("toggle");
const errorBox = byId("error");

const phaseNames = {
  idle: "待连接",
  connecting: "正在连接",
  streaming: "转发中",
  stopping: "正在停止",
  error: "转发中断",
};

let status = { phase: "idle" };
let devices = [];
let refreshing = false;
let acting = false;
let networkBusy = false;
let discoveryError = "";
let actionError = "";
let volumeTimer;
let statusRequest = 0;

function active() {
  return ["connecting", "streaming", "stopping"].includes(status.phase);
}

function render() {
  const running = active();
  const selected = devices.find((entry) => entry.serial === device.value);
  byId("badge").dataset.phase = status.phase;
  const label = byId("state-label");
  if (label.textContent !== phaseNames[status.phase]) {
    label.textContent = phaseNames[status.phase];
  }
  byId("state-detail").textContent = {
    idle: "选择设备后开始转发。",
    connecting: "正在启动 Android 音频服务，等待接收音频。",
    streaming: `${status.serial} · 48 kHz · 立体声`,
    stopping: "正在关闭音频服务并清理 ADB 连接。",
    error: "检查手机的连接与调试授权后，可以重新开始。",
  }[status.phase];

  const controlsLocked = running || acting || networkBusy;
  usb.disabled = controlsLocked;
  networkMode.disabled = controlsLocked;
  network.hidden = !networkMode.checked;
  for (const form of [pairForm, connectForm]) {
    for (const control of form.elements) control.disabled = controlsLocked;
  }
  refresh.disabled = refreshing || controlsLocked;
  refresh.textContent = refreshing ? "刷新中…" : "刷新";
  device.disabled = refreshing || controlsLocked || !device.value;
  buffer.disabled = running || acting;
  quality.disabled = running || acting;
  toggle.disabled = acting || status.phase === "stopping" ||
    (!running && (refreshing || networkBusy || selected?.state !== "device"));
  toggle.dataset.mode = running ? "stop" : "start";
  byId("toggle-text").textContent = status.phase === "stopping" ? "正在停止…" :
    running ? "停止转发" : "开始转发";
  byId("toggle-icon").setAttribute("d", running ? "M5 5h10v10H5z" : "M6 4l10 6-10 6z");

  const error = actionError || status.error || discoveryError;
  if (errorBox.textContent !== error) errorBox.textContent = error;
  errorBox.hidden = !error;
}

function isNetwork(entry) {
  return entry.serial.includes(":") || entry.serial.includes("._adb-tls-connect.");
}

function renderDevices(preferred = device.value) {
  const visible = devices.filter((entry) => isNetwork(entry) === networkMode.checked);
  device.replaceChildren();
  const detail = byId("device-detail");
  if (!visible.length) {
    device.add(new Option(discoveryError ? "无法读取设备" : "未发现设备", ""));
    detail.textContent = discoveryError
      ? "请检查 ADB 是否可用，然后重试。"
      : networkMode.checked
        ? "通过上方入口连接手机，或刷新已连接的网络设备。"
        : "连接 USB 数据线，在手机上授权 USB 调试，然后刷新。";
    return;
  }

  const descriptions = { device: "已就绪", unauthorized: "待授权", offline: "离线" };
  for (const entry of visible) {
    const state = descriptions[entry.state] || entry.state;
    device.add(new Option(entry.serial + " · " + state, entry.serial));
  }
  device.value = visible.find((entry) => entry.serial === preferred)?.serial ||
    visible.find((entry) => entry.state === "device")?.serial || visible[0].serial;
  const ready = visible.filter((entry) => entry.state === "device").length;
  detail.textContent = ready
    ? ready + " 台可用设备 · 共发现 " + visible.length + " 台"
    : "请在手机上确认调试授权，或重新连接离线设备。";
}

async function loadDevices(preferred) {
  refreshing = true;
  discoveryError = "";
  actionError = "";
  render();
  try {
    devices = await invoke("list_devices");
  } catch (error) {
    devices = [];
    discoveryError = `读取 ADB 设备失败。 ${error}`;
  } finally {
    renderDevices(preferred);
    refreshing = false;
    render();
  }
}

async function loadStatus() {
  const request = ++statusRequest;
  const next = await invoke("relay_status");
  if (request === statusRequest) {
    status = next;
    render();
  }
}

async function poll() {
  try {
    await loadStatus();
  } catch (error) {
    actionError = `无法读取转发状态。 ${error}`;
    render();
  } finally {
    setTimeout(poll, 350);
  }
}

byId("window-drag").addEventListener("mousedown", (event) => {
  if (event.button === 0 && event.detail === 1) {
    window.__TAURI__.window.getCurrentWindow().startDragging().catch(console.warn);
  }
});

for (const [id, method] of [
  ["window-minimize", "minimize"],
  ["window-close", "close"],
]) {
  byId(id).addEventListener("click", async () => {
    try {
      await window.__TAURI__.window.getCurrentWindow()[method]();
    } catch (error) {
      actionError = "无法操作窗口。 " + error;
      render();
    }
  });
}

toggle.addEventListener("click", async () => {
  acting = true;
  actionError = "";
  render();
  try {
    if (active()) {
      await invoke("stop_relay");
    } else {
      await invoke("start_relay", {
        settings: {
          serial: device.value,
          bufferMs: Number(buffer.value),
          volume: Number(volume.value),
          quality: quality.value,
        },
      });
    }
    await loadStatus();
  } catch (error) {
    actionError = String(error);
  } finally {
    acting = false;
    render();
  }
});

async function wireless(pair) {
  const pairCode = byId("pair-code");
  const pairingCode = pair ? pairCode.value.trim() : null;
  const address = byId(pair ? "pair-address" : "connect-address").value.trim();
  if (pair) pairCode.value = "";
  networkBusy = true;
  networkStatus.dataset.error = "false";
  networkStatus.textContent = pair ? "正在配对…" : "正在连接…";
  render();
  try {
    const serial = await invoke("wireless_device", { address, pairingCode });
    networkStatus.textContent = pair
      ? "配对成功。如果设备尚未出现，请用无线调试主页的地址连接。"
      : "连接成功，正在更新设备列表。";
    await loadDevices(pair ? undefined : serial);
    if (!pair) networkStatus.textContent = "已连接 " + serial;
  } catch (error) {
    networkStatus.dataset.error = "true";
    networkStatus.textContent = String(error);
  } finally {
    networkBusy = false;
    render();
  }
}

for (const mode of [usb, networkMode]) {
  mode.addEventListener("change", () => {
    if (!networkMode.checked) byId("pair-code").value = "";
    renderDevices();
    render();
  });
}
pairForm.addEventListener("submit", (event) => {
  event.preventDefault();
  return wireless(true);
});
connectForm.addEventListener("submit", (event) => {
  event.preventDefault();
  return wireless(false);
});
refresh.addEventListener("click", () => loadDevices());
device.addEventListener("change", render);
buffer.addEventListener("input", () => {
  byId("buffer-value").textContent = `${buffer.value} ms`;
});
volume.addEventListener("input", () => {
  byId("volume-value").textContent = `${volume.value}%`;
  clearTimeout(volumeTimer);
  if (active()) {
    volumeTimer = setTimeout(async () => {
      try {
        await invoke("set_volume", { volume: Number(volume.value) });
      } catch (error) {
        actionError = `无法更新音量。 ${error}`;
        render();
      }
    }, 50);
  }
});

loadDevices();
poll();

let fittingHeight = false;
let lastHeight = 0;

async function fitWindowHeight() {
  if (fittingHeight) return;
  const height = Math.ceil(document.body.getBoundingClientRect().height);
  if (!height || height === lastHeight) return;

  fittingHeight = true;
  try {
    await invoke("fit_window_height", { height });
    lastHeight = height;
  } catch (error) {
    console.warn("Unable to fit the WaveRelay window to its content:", error);
    return;
  } finally {
    fittingHeight = false;
  }

  // A layout change may have arrived while the native resize was in progress.
  if (Math.ceil(document.body.getBoundingClientRect().height) !== lastHeight) {
    void fitWindowHeight();
  }
}

new ResizeObserver(fitWindowHeight).observe(document.body);
