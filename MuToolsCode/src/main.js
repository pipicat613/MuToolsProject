// 主入口
import { addLog, loadBackendLogs, clearLogs } from "./js/logger.js";
import { showToast } from "./js/toast.js";
import { getDailyQuote, onVersionClick } from "./js/daily-quotes.js";
import { switchPage } from "./js/router.js";
import { updateProgress, initInstallPage } from "./js/install.js";
import { loadSettings, loadAdminStatus, initSettingsPage } from "./js/settings.js";
import { initPackagesPage } from "./js/packages.js";
import { initOptimizePage } from "./js/optimize.js";
import { startMuMuAutoRefresh, onAutoRefreshSettingChanged, initMuMuInfoPage } from "./js/mumu-info.js";
import { initHelpPage } from "./js/help.js";
import { initAboutPage } from "./js/about.js";

const { invoke } = window.__TAURI__.core;

window.addEventListener("DOMContentLoaded", () => {
  addLog("应用启动");

  // 监听后端下载进度事件
  const { listen } = window.__TAURI__.event;
  listen("download-progress", (event) => {
    updateProgress(event.payload);
  }).catch(() => {});

  // 监听后端 toast 通知事件
  listen("toast", (event) => {
    const payload = event.payload;
    if (typeof payload === "string") {
      showToast(payload, "info");
    } else if (payload && typeof payload === "object") {
      const msg = payload.message || payload.msg || "通知";
      const type = payload.type || "info";
      showToast(msg, type);
    }
  }).catch(() => {});

  // 加载设置
  loadSettings();
  loadAdminStatus();

  // 获取数据目录
  invoke("get_data_dir").then(dataDir => {
    addLog(`数据目录: ${dataDir}`);
  }).catch(() => {});

  // 从设置恢复安装目录
  const savedSettings = localStorage.getItem("mutools_settings");
  if (savedSettings) {
    try {
      const s = JSON.parse(savedSettings);
      document.getElementById("install-dir").value = s.installDir || "D:\\Program Files\\Netease\\MuMu";
    } catch (e) {}
  }

  // 导航
  document.querySelectorAll(".nav-item").forEach((item) => {
    item.addEventListener("click", () => {
      switchPage(item.dataset.page);
    });
  });

  // 初始化各页面事件绑定
  initInstallPage();
  initSettingsPage();
  initPackagesPage();
  initMuMuInfoPage();
  initOptimizePage();

  // 日志按钮
  document.getElementById("btn-refresh-logs").addEventListener("click", () => {
    loadBackendLogs();
    addLog("刷新日志");
  });
  document.getElementById("btn-clear-logs").addEventListener("click", clearLogs);

  // 自动刷新开关
  const autoRefreshCheckbox = document.getElementById("settings-auto-refresh-mumu");
  if (autoRefreshCheckbox) {
    autoRefreshCheckbox.addEventListener("change", () => {
      try {
        const saved = JSON.parse(localStorage.getItem("mutools_settings") || "{}");
        saved.autoRefreshMuMu = autoRefreshCheckbox.checked;
        localStorage.setItem("mutools_settings", JSON.stringify(saved));
      } catch (e) {}
      onAutoRefreshSettingChanged();
    });
  }

  // 启动自动刷新
  startMuMuAutoRefresh();

  // 每日一句
  const quoteEl = document.getElementById("daily-quote-text");
  const quoteBox = document.querySelector(".daily-quote");
  if (quoteEl) {
    quoteEl.textContent = getDailyQuote();
  }
  if (quoteBox) {
    quoteBox.addEventListener("click", () => {
      if (quoteEl) {
        quoteEl.textContent = getDailyQuote();
      }
    });
  }

  // 关于页面版本号彩蛋
  const versionEl = document.getElementById("about-version");
  if (versionEl) {
    versionEl.addEventListener("click", onVersionClick);
  }

  // 帮助页面
  initHelpPage();

  // 关于页面
  initAboutPage();
});