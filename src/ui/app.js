const invoke = window.__TAURI__.core.invoke;
const { t } = i18n;
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

let status = { phase: "idle" };
let devices = [];
let refreshing = false;
let acting = false;
let networkBusy = false;
let discoveryError;
let actionError;
let networkMessage;
let volumeTimer;
let statusRequest = 0;

function active() {
  return ["connecting", "streaming", "stopping"].includes(status.phase);
}

function messageText(message) {
  if (!message) return "";
  return [message.key && t(message.key, message.values), message.detail].filter(Boolean).join(" ");
}

function render() {
  const running = active();
  const selected = devices.find((entry) => entry.serial === device.value);
  byId("badge").dataset.phase = status.phase;
  const label = byId("state-label");
  if (label.textContent !== t(status.phase)) {
    label.textContent = t(status.phase);
  }
  byId("state-detail").textContent = t(status.phase + "Detail", { serial: status.serial });
  const networkText = messageText(networkMessage);
  if (networkStatus.textContent !== networkText) networkStatus.textContent = networkText;

  const controlsLocked = running || acting || networkBusy;
  usb.disabled = controlsLocked;
  networkMode.disabled = controlsLocked;
  network.hidden = !networkMode.checked;
  for (const form of [pairForm, connectForm]) {
    for (const control of form.elements) control.disabled = controlsLocked;
  }
  refresh.disabled = refreshing || controlsLocked;
  refresh.textContent = t(refreshing ? "refreshing" : "refresh");
  device.disabled = refreshing || controlsLocked || !device.value;
  buffer.disabled = running || acting;
  quality.disabled = running || acting;
  toggle.disabled = acting || status.phase === "stopping" ||
    (!running && (refreshing || networkBusy || selected?.state !== "device"));
  toggle.dataset.mode = running ? "stop" : "start";
  byId("toggle-text").textContent = status.phase === "stopping" ? t("stoppingButton") :
    t(running ? "stop" : "start");
  byId("toggle-icon").setAttribute("d", running ? "M5 5h10v10H5z" : "M6 4l10 6-10 6z");

  const error = messageText(actionError || (status.error && { key: "relayFailed", detail: status.error }) || discoveryError);
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
    device.add(new Option(t(discoveryError ? "deviceError" : "noDevices"), ""));
    detail.textContent = discoveryError
      ? t("adbHint")
      : networkMode.checked
        ? t("emptyNetwork")
        : t("emptyUsb");
    return;
  }

  for (const entry of visible) {
    const state = t(entry.state);
    device.add(new Option(entry.serial + " · " + state, entry.serial));
  }
  device.value = visible.find((entry) => entry.serial === preferred)?.serial ||
    visible.find((entry) => entry.state === "device")?.serial || visible[0].serial;
  const ready = visible.filter((entry) => entry.state === "device").length;
  detail.textContent = ready
    ? t("deviceCount", { ready, total: visible.length })
    : t("authorizationHint");
}

async function loadDevices(preferred) {
  refreshing = true;
  discoveryError = null;
  actionError = null;
  render();
  try {
    devices = await invoke("list_devices");
  } catch (error) {
    devices = [];
    discoveryError = { key: "discoveryFailed", detail: String(error) };
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
    actionError = { key: "statusFailed", detail: String(error) };
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
      actionError = { key: "windowFailed", detail: String(error) };
      render();
    }
  });
}

toggle.addEventListener("click", async () => {
  acting = true;
  actionError = null;
  render();
  try {
    if (active()) {
      await invoke("stop_relay");
    } else {
      await invoke("start_relay", {
        language: i18n.language,
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
    actionError = { key: "requestFailed", detail: String(error) };
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
  networkMessage = { key: pair ? "pairing" : "networkConnecting" };
  render();
  try {
    const serial = await invoke("wireless_device", { address, pairingCode, language: i18n.language });
    networkMessage = { key: pair ? "paired" : "networkRefreshing" };
    await loadDevices(pair ? undefined : serial);
    if (!pair) networkMessage = { key: "networkConnected", values: { serial } };
  } catch (error) {
    networkStatus.dataset.error = "true";
    networkMessage = { detail: String(error) };
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
        actionError = { key: "volumeFailed", detail: String(error) };
        render();
      }
    }, 50);
  }
});

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

let languageChanged = false;
byId("language-toggle").addEventListener("click", () => {
  languageChanged = true;
  i18n.apply(i18n.language === "en" ? "zh-CN" : "en", true);
  renderDevices();
  render();
});

async function initialize() {
  i18n.apply();
  try {
    const language = await invoke("gui_language");
    if (language && !languageChanged) i18n.apply(language);
  } catch (error) {
    console.warn("Could not read the startup language:", error);
  }
  void loadDevices();
  void poll();
}

void initialize();
