const i18n = (() => {
  const messages = {
    "en": {
      "controls": "Window and appearance controls",
      "languageTitle": "Switch to Simplified Chinese",
      "languageToggle": "中文",
      "themeTitle": "Switch color theme",
      "themeLabel": "Dark mode",
      "minimize": "Minimize",
      "close": "Close",
      "source": "01 / Audio source",
      "devices": "Android device",
      "refresh": "Refresh",
      "refreshing": "Refreshing…",
      "connection": "Connection",
      "usb": "USB",
      "network": "Network",
      "networkHint": "Enable Wireless debugging on your phone. Connect both devices to the same local network.",
      "firstPair": "First-time pairing",
      "pairHint": "Open the pairing code dialog on your phone.",
      "pairAddress": "Pairing address",
      "pairPlaceholder": "IP:pairing port",
      "pairCode": "Six-digit pairing code",
      "pair": "Pair",
      "pairedDevice": "Connect a paired device",
      "connectAddress": "Connection address",
      "connectPlaceholder": "IP:connection port",
      "connect": "Connect",
      "connectHint": "Use the address on the Wireless debugging screen. The pairing and connection ports are usually different.",
      "selectDevice": "Select a device",
      "looking": "Looking for devices…",
      "readingDevices": "Reading the local ADB device list.",
      "playback": "02 / Playback",
      "pcAudio": "PC audio",
      "defaultOutput": "Default output",
      "quality": "Audio quality",
      "pcm": "Lossless · PCM",
      "opus192": "High quality · Opus 192 kbps",
      "opus96": "Low bandwidth · Opus 96 kbps",
      "qualityNote": "PCM is lossless. Opus uses less bandwidth.",
      "volume": "Volume",
      "buffer": "Target buffer",
      "bufferNote": "Increase the buffer if Wi-Fi playback stutters.",
      "idle": "Idle",
      "connecting": "Connecting",
      "streaming": "Live",
      "stopping": "Stopping",
      "error": "Error",
      "idleDetail": "Select a device to start streaming.",
      "connectingDetail": "Starting the Android audio service.",
      "streamingDetail": "{serial} · 48 kHz · Stereo",
      "stoppingDetail": "Stopping the audio service and cleaning up ADB.",
      "errorDetail": "Check the connection and debugging authorization, then retry.",
      "start": "Start streaming",
      "stop": "Stop streaming",
      "stoppingButton": "Stopping…",
      "deviceError": "Unable to read devices",
      "noDevices": "No devices found",
      "adbHint": "Check that ADB is available and try again.",
      "emptyNetwork": "Connect your phone above or refresh the network device list.",
      "emptyUsb": "Connect the USB cable, authorize debugging on your phone, and refresh.",
      "device": "Ready",
      "unauthorized": "Unauthorized",
      "offline": "Offline",
      "deviceCount": "{ready} available · {total} found",
      "authorizationHint": "Approve debugging on your phone or reconnect an offline device.",
      "discoveryFailed": "Could not read ADB devices.",
      "statusFailed": "Could not read the relay status.",
      "windowFailed": "Could not control the window.",
      "relayFailed": "Audio relay failed.",
      "requestFailed": "Could not change the relay state.",
      "volumeFailed": "Could not update the volume.",
      "pairing": "Pairing…",
      "networkConnecting": "Connecting…",
      "paired": "Pairing succeeded. If the device does not appear, connect using the address on the Wireless debugging screen.",
      "networkRefreshing": "Connected. Refreshing the device list…",
      "networkConnected": "Connected to {serial}"
    },
    "zh-CN": {
      "controls": "窗口和外观控制",
      "languageTitle": "切换到 English",
      "languageToggle": "EN",
      "themeTitle": "切换明暗主题",
      "themeLabel": "暗黑模式",
      "minimize": "最小化",
      "close": "关闭",
      "source": "01 / 音频来源",
      "devices": "Android 设备",
      "refresh": "刷新",
      "refreshing": "刷新中…",
      "connection": "连接方式",
      "usb": "USB 连接",
      "network": "网络连接",
      "networkHint": "开启手机的「无线调试」，并让手机和电脑接入同一局域网。",
      "firstPair": "首次配对",
      "pairHint": "在手机上打开配对码弹窗。",
      "pairAddress": "配对地址",
      "pairPlaceholder": "IP:配对端口",
      "pairCode": "六位配对码",
      "pair": "配对",
      "pairedDevice": "连接已配对设备",
      "connectAddress": "连接地址",
      "connectPlaceholder": "IP:连接端口",
      "connect": "连接",
      "connectHint": "使用无线调试主页显示的地址。配对和连接端口通常不同。",
      "selectDevice": "选择设备",
      "looking": "正在查找设备…",
      "readingDevices": "正在读取本机 ADB 设备列表。",
      "playback": "02 / 播放设置",
      "pcAudio": "电脑音频",
      "defaultOutput": "系统默认输出",
      "quality": "音质",
      "pcm": "无损 · PCM",
      "opus192": "高音质 · Opus 192 kbps",
      "opus96": "省带宽 · Opus 96 kbps",
      "qualityNote": "PCM 无损，Opus 更省带宽。",
      "volume": "音量",
      "buffer": "目标缓冲",
      "bufferNote": "无线播放卡顿时，可尝试增大缓冲。",
      "idle": "待连接",
      "connecting": "正在连接",
      "streaming": "转发中",
      "stopping": "正在停止",
      "error": "转发中断",
      "idleDetail": "选择设备后开始转发。",
      "connectingDetail": "正在启动 Android 音频服务，等待接收音频。",
      "streamingDetail": "{serial} · 48 kHz · 立体声",
      "stoppingDetail": "正在关闭音频服务并清理 ADB 连接。",
      "errorDetail": "检查手机的连接与调试授权后，可以重新开始。",
      "start": "开始转发",
      "stop": "停止转发",
      "stoppingButton": "正在停止…",
      "deviceError": "无法读取设备",
      "noDevices": "未发现设备",
      "adbHint": "请检查 ADB 是否可用，然后重试。",
      "emptyNetwork": "通过上方入口连接手机，或刷新已连接的网络设备。",
      "emptyUsb": "连接 USB 数据线，在手机上授权 USB 调试，然后刷新。",
      "device": "已就绪",
      "unauthorized": "待授权",
      "offline": "离线",
      "deviceCount": "{ready} 台可用设备 · 共发现 {total} 台",
      "authorizationHint": "请在手机上确认调试授权，或重新连接离线设备。",
      "discoveryFailed": "读取 ADB 设备失败。",
      "statusFailed": "无法读取转发状态。",
      "windowFailed": "无法操作窗口。",
      "relayFailed": "音频转发失败。",
      "requestFailed": "无法更改转发状态。",
      "volumeFailed": "无法更新音量。",
      "pairing": "正在配对…",
      "networkConnecting": "正在连接…",
      "paired": "配对成功。如果设备尚未出现，请用无线调试主页的地址连接。",
      "networkRefreshing": "连接成功，正在更新设备列表。",
      "networkConnected": "已连接 {serial}"
    }
  };
  const storageKey = "waverelay.language";
  let language = navigator.language.toLowerCase().startsWith("zh") ? "zh-CN" : "en";
  try {
    const saved = localStorage.getItem(storageKey);
    if (saved === "en" || saved === "zh-CN") language = saved;
  } catch {
    // The system language remains the default when storage is unavailable.
  }

  function t(key, values = {}) {
    const catalog = messages[language];
    const message = Object.hasOwn(catalog, key) ? catalog[key] : key;
    return message.replace(/\{(\w+)\}/g, (_, name) => String(values[name] ?? ""));
  }

  function apply(next = language, remember = false) {
    language = next === "zh-CN" ? "zh-CN" : "en";
    document.documentElement.lang = language;
    for (const [selector, attribute, key] of [
      ["[data-i18n]", null, "i18n"],
      ["[data-i18n-title]", "title", "i18nTitle"],
      ["[data-i18n-label]", "aria-label", "i18nLabel"],
      ["[data-i18n-placeholder]", "placeholder", "i18nPlaceholder"],
    ]) {
      for (const element of document.querySelectorAll(selector)) {
        const value = t(element.dataset[key]);
        if (attribute) element.setAttribute(attribute, value);
        else element.textContent = value;
      }
    }
    if (remember) {
      try {
        localStorage.setItem(storageKey, language);
      } catch {
        // The selection still applies to the current session.
      }
    }
  }

  return { t, apply, get language() { return language; } };
})();
