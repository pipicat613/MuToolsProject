// 设置页面
import { showToast } from "./toast.js";
import { addLog } from "./logger.js";
import { onAutoRefreshSettingChanged } from "./mumu-info.js";
const { invoke } = window.__TAURI__.core;

export function loadSettings() {
  try {
    const saved = localStorage.getItem("mutools_settings");
    if (saved) {
      const settings = JSON.parse(saved);
      document.getElementById("settings-log-path").value = settings.logPath || "./log";
      document.getElementById("settings-download-path").value = settings.downloadPath || "./download";

      const dataDirMode = settings.dataDirMode || "appdata";
      const dataDirCustom = settings.dataDirCustom || "";
      document.getElementById("settings-data-dir-mode").value = dataDirMode;
      document.getElementById("settings-data-dir-custom").value = dataDirCustom;
      updateDataDirCustomUI();

      const adminElevation = settings.adminElevation !== undefined ? settings.adminElevation : true;
      document.getElementById("settings-admin-elevation").checked = adminElevation;

      document.getElementById("settings-aria2-connections").value = settings.aria2MaxConnections || 5;
      document.getElementById("settings-aria2-split").value = settings.aria2Split || 5;
      document.getElementById("settings-auto-delete-installer").checked = settings.autoDeleteInstaller || false;
      document.getElementById("settings-auto-refresh-mumu").checked = settings.autoRefreshMuMu || false;
    } else {
      document.getElementById("settings-admin-elevation").checked = true;
      document.getElementById("settings-aria2-connections").value = 5;
      document.getElementById("settings-aria2-split").value = 5;
    }
    loadAria2ConfigFromBackend();
    loadAutoDeleteConfigFromBackend();
  } catch (e) {
    console.error("加载设置失败:", e);
  }
}

async function loadAria2ConfigFromBackend() {
  try {
    const cfg = await invoke("get_aria2_config");
    document.getElementById("settings-aria2-connections").value = cfg.maxConnections;
    document.getElementById("settings-aria2-split").value = cfg.split;
  } catch (e) {
    console.error("加载 aria2 配置失败:", e);
  }
}

async function loadAutoDeleteConfigFromBackend() {
  try {
    const enabled = await invoke("get_auto_delete_installer");
    document.getElementById("settings-auto-delete-installer").checked = enabled;
  } catch (e) {
    console.error("加载自动删除配置失败:", e);
  }
}

export function saveSettings() {
  const logPath = document.getElementById("settings-log-path").value.trim();
  const downloadPath = document.getElementById("settings-download-path").value.trim();
  const dataDirMode = document.getElementById("settings-data-dir-mode").value;
  const dataDirCustom = document.getElementById("settings-data-dir-custom").value.trim();
  const adminElevation = document.getElementById("settings-admin-elevation").checked;
  const aria2MaxConnections = parseInt(document.getElementById("settings-aria2-connections").value) || 1;
  const aria2Split = parseInt(document.getElementById("settings-aria2-split").value) || 5;
  const autoDeleteInstaller = document.getElementById("settings-auto-delete-installer").checked;
  const autoRefreshMuMu = document.getElementById("settings-auto-refresh-mumu").checked;
  const settings = {
    logPath, downloadPath, dataDirMode, dataDirCustom,
    adminElevation, aria2MaxConnections, aria2Split, autoDeleteInstaller, autoRefreshMuMu
  };
  localStorage.setItem("mutools_settings", JSON.stringify(settings));

  invoke("save_data_config", { dataDirMode, dataDirCustom }).catch(e => console.error("保存数据目录配置失败:", e));
  invoke("save_admin_elevation", { enabled: adminElevation }).catch(e => console.error("保存管理员提权设置失败:", e));
  invoke("save_aria2_config", { maxConnections: aria2MaxConnections, split: aria2Split }).catch(e => console.error("保存 aria2 配置失败:", e));
  invoke("save_auto_delete_installer", { enabled: autoDeleteInstaller }).catch(e => console.error("保存自动删除设置失败:", e));
  // 自动刷新设置变更
  onAutoRefreshSettingChanged();

  const prevSettings = (() => {
    try { return JSON.parse(localStorage.getItem("mutools_settings") || "{}"); } catch { return {}; }
  })();
  const prevElevation = prevSettings.adminElevation !== undefined ? prevSettings.adminElevation : true;
  if (prevElevation !== adminElevation) {
    showToast("提权设置已保存，重启应用后生效", "info");
    addLog(`管理员提权设置已切换为: ${adminElevation}（需重启生效）`);
  } else {
    showToast("设置已保存", "success");
  }
}

export function updateDataDirCustomUI() {
  const mode = document.getElementById("settings-data-dir-mode").value;
  const customGroup = document.getElementById("settings-data-dir-custom-group");
  customGroup.style.display = mode === "custom" ? "" : "none";
}

// 管理员权限状态

export async function loadAdminStatus() {
  const valueEl = document.getElementById("admin-status-value");
  if (!valueEl) return;
  valueEl.textContent = "检测中...";
  valueEl.className = "admin-status-value unknown";
  try {
    const elevated = await invoke("check_admin_status");
    if (elevated) {
      valueEl.textContent = "管理员权限";
      valueEl.className = "admin-status-value elevated";
    } else {
      valueEl.textContent = "普通权限";
      valueEl.className = "admin-status-value not-elevated";
    }
  } catch (e) {
    valueEl.textContent = "检测失败";
    valueEl.className = "admin-status-value unknown";
    console.error("检测管理员状态失败:", e);
  }
}

// 事件绑定

export function initSettingsPage() {
  document.getElementById("btn-save-settings").addEventListener("click", saveSettings);

  const dataDirModeSelect = document.getElementById("settings-data-dir-mode");
  if (dataDirModeSelect) {
    dataDirModeSelect.addEventListener("change", updateDataDirCustomUI);
  }
}