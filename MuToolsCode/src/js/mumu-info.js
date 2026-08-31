
import { addLog } from "./logger.js";
import { escapeHtml } from "./toast.js";
const { invoke } = window.__TAURI__.core;

let mumuAutoRefreshTimer = null;
let mumuAutoRefreshEnabled = false;

export async function loadMuMuInfo() {
  const container = document.getElementById("mumu-info-content");
  if (!container) return;

  container.innerHTML = '<div class="mumu-info-loading">加载中...</div>';

  try {
    const info = await invoke("get_mumu_info");
    renderMuMuInfo(info);
    addLog("MuMu 信息加载成功");
  } catch (e) {
    container.innerHTML = `<div class="mumu-info-error">加载失败: ${e}</div>`;
    addLog(`MuMu 信息加载失败: ${e}`);
  }
}

function renderMuMuInfo(info) {
  const container = document.getElementById("mumu-info-content");
  if (!container) return;

  const majorVer = info.majorVersion || "?";
  const fullVer = info.fullVersion || "未知";
  const channel = info.channel || "未知";
  const installDir = info.installDir || "未检测到";
  const mumuManager = info.mumuManagerPath || "未找到";
  const appdataDir = info.appdataDir || "未知";
  const registryKey = info.registryKey || "未检测到";
  const uninstallStr = info.uninstallString || "未检测到";

  const isMissing = (v) => v === "未检测到" || v === "未找到" || v === "未知";

  let html = '<div class="mumu-info-card">';
  html += '<div class="mumu-info-hero">';
  html += '<div class="mumu-info-versions">';
  html += `<div class="mumu-info-major">MuMu ${escapeHtml(majorVer)}</div>`;
  html += `<div class="mumu-info-full">完整版本号: ${escapeHtml(fullVer)}</div>`;
  html += `<div class="mumu-info-channel">渠道: ${escapeHtml(channel)}</div>`;
  html += '</div></div>';

  html += '<div class="mumu-info-details">';
  const detailItems = [
    { label: "安装根目录", value: installDir },
    { label: "MuMuManager", value: mumuManager },
    { label: "AppData 目录", value: appdataDir },
    { label: "注册表项", value: registryKey },
    { label: "卸载路径", value: uninstallStr },
  ];
  detailItems.forEach((item) => {
    const cls = isMissing(item.value) ? "mumu-info-detail-row missing" : "mumu-info-detail-row";
    html += `<div class="${cls}"><span class="mumu-info-detail-label">${item.label}</span><span class="mumu-info-detail-value" title="${escapeHtml(item.value)}">${escapeHtml(item.value)}</span></div>`;
  });
  html += '</div>';

  if (info.errors && info.errors.length > 0) {
    html += '<div class="mumu-info-errors">';
    info.errors.forEach((err) => {
      html += `<div class="mumu-info-error-item">${escapeHtml(err)}</div>`;
    });
    html += '</div>';
  }

  html += '</div>';
  container.innerHTML = html;
}

// 自动刷新

export function startMuMuAutoRefresh() {
  stopMuMuAutoRefresh();
  const enabled = getAutoRefreshSetting();
  mumuAutoRefreshEnabled = enabled;
  if (enabled) {
    addLog("自动刷新 MuMu 信息");
    loadMuMuInfo().catch(() => { });
  }
}

export function stopMuMuAutoRefresh() {
  if (mumuAutoRefreshTimer) {
    clearInterval(mumuAutoRefreshTimer);
    mumuAutoRefreshTimer = null;
  }
  mumuAutoRefreshEnabled = false;
}

export function refreshMuMuInfoAfterInstall() {
  if (getAutoRefreshSetting()) {
    loadMuMuInfo().catch(() => { });
  }
}

export function getAutoRefreshSetting() {
  try {
    const saved = localStorage.getItem("mutools_settings");
    if (saved) {
      const settings = JSON.parse(saved);
      return settings.autoRefreshMuMu || false;
    }
  } catch (e) { }
  return false;
}

export function onAutoRefreshSettingChanged() {
  const enabled = getAutoRefreshSetting();
  if (enabled !== mumuAutoRefreshEnabled) {
    if (enabled) {
      startMuMuAutoRefresh();
    } else {
      stopMuMuAutoRefresh();
    }
  }
}

// 事件绑定

export function initMuMuInfoPage() {
  document.getElementById("btn-mumu-refresh").addEventListener("click", () => {
    addLog("手动刷新 MuMu 信息");
    loadMuMuInfo();
  });
}