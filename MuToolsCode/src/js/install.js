// MuMu安装
import { showToast, formatSize } from "./toast.js";
import { addLog } from "./logger.js";
import { refreshMuMuInfoAfterInstall } from "./mumu-info.js";
const { invoke } = window.__TAURI__.core;

// 安装进度状态
let installProgressPhase = null; // "download" | "install" | null
let simulatedInstallTimer = null;
let totalProgressPct = 0;
let downloadPaused = false;

// 当前下载状态
let downloadState = {
  active: false,
  version: null,
  downloadDir: null,
  installDir: null,
  url: null,
  components: null,
  productVersion: null,
};


export function showInlineProgress() {
  const bar = document.getElementById("install-progress-bar");
  if (bar) bar.style.display = "";
  document.getElementById("install-progress-task").textContent = "准备中...";
  document.getElementById("install-progress-percent").textContent = "0%";
  document.getElementById("install-progress-fill").style.width = "0%";
  document.getElementById("install-progress-detail").textContent = "";
  document.getElementById("install-progress-actions").style.display = "none";
  totalProgressPct = 0;
  installProgressPhase = null;
  downloadPaused = false;
  updatePauseButtonLabel();
}

export function hideInlineProgress() {
  const bar = document.getElementById("install-progress-bar");
  if (bar) bar.style.display = "none";
  document.getElementById("install-progress-actions").style.display = "none";
  cancelSimulatedInstall();
  installProgressPhase = null;
  downloadPaused = false;
}

function showDownloadButtons() {
  document.getElementById("install-progress-actions").style.display = "";
  downloadPaused = false;
  updatePauseButtonLabel();
}

function updatePauseButtonLabel() {
  const btn = document.getElementById("btn-pause-download");
  if (btn) {
    btn.textContent = downloadPaused ? "继续" : "暂停";
  }
}

export function updateInlineProgress(totalPct, task, detail) {
  totalProgressPct = totalPct;
  const fill = document.getElementById("install-progress-fill");
  const percentEl = document.getElementById("install-progress-percent");
  const taskEl = document.getElementById("install-progress-task");
  const detailEl = document.getElementById("install-progress-detail");
  if (fill) fill.style.width = `${Math.round(totalPct)}%`;
  if (percentEl) percentEl.textContent = `${Math.round(totalPct)}%`;
  if (taskEl && task) taskEl.textContent = task;
  if (detailEl) detailEl.textContent = detail || "";
}

function cancelSimulatedInstall() {
  if (simulatedInstallTimer) {
    clearInterval(simulatedInstallTimer);
    simulatedInstallTimer = null;
  }
}

export function startSimulatedInstallProgress() {
  cancelSimulatedInstall();
  const startTime = Date.now();
  const duration = 120 * 1000; // 120s
  return new Promise((resolve) => {
    simulatedInstallTimer = setInterval(() => {
      const elapsed = Date.now() - startTime;
      const t = Math.min(elapsed / duration, 1);
      const simulatedPct = 60 + 38 * (1 - Math.pow(1 - t, 2));
      updateInlineProgress(simulatedPct, "安装中...", "正在静默安装，请稍候...");
      if (t >= 1) {
        clearInterval(simulatedInstallTimer);
        simulatedInstallTimer = null;
      }
    }, 200);
    simulatedInstallTimer._resolve = resolve;
  });
}

export function finishInstallProgress() {
  cancelSimulatedInstall();
  if (simulatedInstallTimer && simulatedInstallTimer._resolve) {
    simulatedInstallTimer._resolve();
    simulatedInstallTimer._resolve = null;
  }
  simulatedInstallTimer = null;
  updateInlineProgress(100, "安装完成!", "MuMu 模拟器已成功安装");
  setTimeout(() => {
    hideInlineProgress();
  }, 3000);
}

// 下载控制

async function pauseOrResumeDownload() {
  if (downloadPaused) {
    downloadPaused = false;
    updatePauseButtonLabel();
    try {
      await invoke("resume_download");
      addLog("恢复下载");
    } catch (e) {
      addLog(`恢复下载失败: ${e}`);
    }

    if (downloadState.version === "MuMu12-V6") {
      try {
        const downloadedFilesJson = await invoke("download_v6_installer", {
          saveDir: downloadState.downloadDir,
          componentsJson: JSON.stringify(downloadState.components),
          resume: true,
        });
        addLog(`V6 所有组件下载完成`);

        installProgressPhase = "install";
        document.getElementById("install-progress-actions").style.display = "none";
        updateInlineProgress(60, "安装中...", "正在静默安装，请稍候...");
        startSimulatedInstallProgress();

        const result = await invoke("install_v6_setup", {
          componentsJson: JSON.stringify(downloadState.components),
          downloadedFilesJson: downloadedFilesJson,
          installDir: downloadState.installDir,
          productVersion: downloadState.productVersion,
        });
        finishInstallProgress();
        showToast(`${downloadState.version} 安装成功！`, "success");
        addLog(`${downloadState.version} 安装成功: ${result}`);
        downloadState.active = false;
        refreshMuMuInfoAfterInstall();
      } catch (e) {
        handleInstallError(e);
      }
    } else {
      try {
        const installerPath = await invoke("download_v4_installer", {
          saveDir: downloadState.downloadDir,
          url: downloadState.url || "",
          version: downloadState.version,
          resume: true,
        });
        addLog(`${downloadState.version} 安装包下载完成: ${installerPath}`);
        showToast("下载完成，正在静默安装...", "info");

        installProgressPhase = "install";
        document.getElementById("install-progress-actions").style.display = "none";
        updateInlineProgress(60, "安装中...", "正在静默安装，请稍候...");
        startSimulatedInstallProgress();

        const result = await invoke("install_v4_setup", {
          installerPath: installerPath,
          installDir: downloadState.installDir,
        });
        finishInstallProgress();
        showToast(`${downloadState.version} 安装成功！`, "success");
        addLog(`${downloadState.version} 安装成功: ${result}`);
        downloadState.active = false;
        refreshMuMuInfoAfterInstall();
      } catch (e) {
        handleInstallError(e);
      }
    }
  } else {
    try {
      await invoke("pause_download");
      showToast("下载已暂停", "info");
    } catch (e) {
      addLog(`暂停下载失败: ${e}`);
    }
    downloadPaused = true;
  }
  updatePauseButtonLabel();
}

async function cancelDownload() {
  try {
    await invoke("cancel_download");
    addLog("下载已取消");
    showToast("下载已取消，已删除下载的文件", "info");
  } catch (e) {
    addLog(`取消下载失败: ${e}`);
  }
  hideInlineProgress();
  downloadState.active = false;
}

function handleInstallError(e) {
  if (String(e).includes("已取消")) {
    hideInlineProgress();
    showToast("下载已取消，文件已删除", "info");
    addLog(`${downloadState.version} 下载已取消`);
    downloadState.active = false;
  } else if (String(e).includes("已暂停")) {
    downloadPaused = true;
    updatePauseButtonLabel();
    showToast("下载已暂停", "info");
    addLog(`${downloadState.version} 下载已暂停`);
  } else {
    hideInlineProgress();
    showToast(`${downloadState.version} 安装失败: ${e}`, "error");
    downloadState.active = false;
  }
}

// 安装逻辑

export async function fetchDownloadUrl() {
  const version = document.getElementById("select-version").value;
  showToast("正在获取下载链接", "info");

  try {
    const url = await invoke("fetch_download_url", { version: version });

    if (version === "MuMu12-V6") {
      try {
        const v6Data = JSON.parse(url);
        window._v6Components = v6Data.components || [];
        window._v6Version = v6Data.version || "";
        const components = window._v6Components;
        const nxmain = components.find(c => c.name.toUpperCase() === "NXMAIN");
        const nemux = components.find(c => c.name.toUpperCase() === "NEMUX");
        const mumu15 = components.find(c => c.name.toUpperCase() === "MUMU15");
        document.getElementById("v6-link-nxmain").value = nxmain ? nxmain.link : "";
        document.getElementById("v6-link-nemux").value = nemux ? nemux.link : "";
        document.getElementById("v6-link-mumu15").value = mumu15 ? mumu15.link : "";
        showToast(`获取 V6 组件列表成功: ${components.length} 个组件`, "success");
        addLog(`获取 V6 组件列表成功: version=${v6Data.version}, ${components.length} 个组件`);
      } catch (e) {
        document.getElementById("install-path").value = url;
        showToast("获取下载链接成功", "success");
        addLog(`获取下载链接成功: ${url}`);
      }
    } else {
      document.getElementById("install-path").value = url;
      showToast("获取下载链接成功", "success");
      addLog(`获取下载链接成功: ${url}`);
    }
  } catch (e) {
    showToast(`获取下载链接失败: ${e}`, "error");
  }
}

export async function browseDirectory() {
  try {
    const path = await invoke("browse_directory");
    if (path) {
      document.getElementById("install-dir").value = path;
      addLog(`选择安装目录: ${path}`);
    }
  } catch (e) {
    addLog(`选择目录失败: ${e}`);
  }
}

export async function browseFile() {
  try {
    const path = await invoke("browse_file");
    if (path) {
      document.getElementById("install-path").value = path;
      addLog(`选择安装包文件: ${path}`);
    }
  } catch (e) {
    addLog(`选择文件失败: ${e}`);
  }
}

async function browseV6File(targetId) {
  try {
    const path = await invoke("browse_file");
    if (path) {
      document.getElementById(targetId).value = path;
      addLog(`选择 ${targetId} 文件: ${path}`);
    }
  } catch (e) {
    addLog(`选择文件失败: ${e}`);
  }
}

export async function startInstall() {
  const version = document.getElementById("select-version").value;
  const method = document.getElementById("select-method").value;
  const path = document.getElementById("install-path").value.trim();
  const installDir = document.getElementById("install-dir").value.trim();

  if (!installDir) {
    showToast("输入安装目录", "error");
    addLog("安装失败: 未输入安装目录");
    return;
  }

  if (version === "MuMu12-V4" || version === "MuMu12-V5" || version === "MuMu12-V6") {
    if (method === "local") {
      if (version === "MuMu12-V6") {
        await doV6LocalInstall(installDir);
        return;
      }

      if (!path) {
        showToast("选择安装包路径", "error");
        addLog("安装失败: 未输入安装包路径");
        return;
      }

      updateInlineProgress(0, "安装中...", "正在静默安装，请稍候...");
      addLog(`${version} 本地安装: ${path} -> ${installDir}`);
      startSimulatedInstallProgress();

      try {
        const result = await invoke("install_v4_setup", { installerPath: path, installDir });
        finishInstallProgress();
        showToast(`${version} 安装成功！`, "success");
        addLog(`${version} 安装成功: ${result}`);
        refreshMuMuInfoAfterInstall();
      } catch (e) {
        hideInlineProgress();
        showToast(`${version} 安装失败: ${e}`, "error");
      }
      return;
    }

    // 在线安装
    await doOnlineInstall(version, path, installDir);
    return;
  }

  // 其他版本沿用原有逻辑
  if (!path) {
    showToast("输入安装包路径", "error");
    addLog("安装失败: 未输入安装包路径");
    return;
  }

  showToast("正在安装，请稍候...", "info");
  addLog(`开始安装: 版本=${version}, 方式=${method}, 路径=${path}, 目录=${installDir}`);

  try {
    const result = await invoke("start_install", { version, method, path, installDir });
    showToast(result, "success");
    addLog(`安装成功: ${result}`);
  } catch (e) {
    showToast(`安装失败: ${e}`, "error");
  }
}

async function doV6LocalInstall(installDir) {
  const nxmainPath = document.getElementById("v6-link-nxmain").value.trim();
  const nemuxPath = document.getElementById("v6-link-nemux").value.trim();
  const mumu15Path = document.getElementById("v6-link-mumu15").value.trim();

  if (!nxmainPath && !nemuxPath && !mumu15Path) {
    showToast("请至少选择一个 V6 组件安装文件路径", "error");
    return;
  }

  let components = [];
  let productVersion = window._v6Version || "6.0.0.0";

  if (window._v6Components && window._v6Components.length > 0) {
    components = window._v6Components.filter(c => {
      const name = c.name.toUpperCase();
      if (name === "NXMAIN" && nxmainPath) return true;
      if (name === "NEMUX" && nemuxPath) return true;
      if (name === "MUMU15" && mumu15Path) return true;
      return false;
    });
  }
  if (components.length === 0) {
    if (nxmainPath) components.push({ name: "NXMAIN", version: productVersion, checksum: "", link: "", size: 0 });
    if (nemuxPath) components.push({ name: "NEMUX", version: productVersion, checksum: "", link: "", size: 0 });
    if (mumu15Path) components.push({ name: "MUMU15", version: productVersion, checksum: "", link: "", size: 0 });
  }

  const filePaths = [];
  if (nxmainPath) filePaths.push(nxmainPath);
  if (nemuxPath) filePaths.push(nemuxPath);
  if (mumu15Path) filePaths.push(mumu15Path);

  addLog(`V6 本地安装: ${filePaths.length} 个组件文件`);
  showInlineProgress();
  installProgressPhase = "install";
  updateInlineProgress(0, "安装中...", "正在静默安装，请稍候...");
  startSimulatedInstallProgress();

  try {
    const result = await invoke("install_v6_setup", {
      componentsJson: JSON.stringify(components),
      downloadedFilesJson: JSON.stringify(filePaths),
      installDir,
      productVersion,
    });
    finishInstallProgress();
    showToast(`V6 安装成功！`, "success");
    addLog(`V6 安装成功: ${result}`);
    refreshMuMuInfoAfterInstall();
  } catch (e) {
    hideInlineProgress();
    showToast(`V6 安装失败: ${e}`, "error");
  }
}

async function doOnlineInstall(version, path, installDir) {
  let downloadDir = "";
  try {
    const saved = localStorage.getItem("mutools_settings");
    if (saved) {
      const settings = JSON.parse(saved);
      downloadDir = settings.downloadPath || "./download";
    }
  } catch (e) { }
  if (!downloadDir) downloadDir = "./download";

  showToast(`正在准备 ${version} 在线安装...`, "info");
  addLog(`开始 ${version} 在线安装流程: 下载目录=${downloadDir}, 安装目录=${installDir}`);

  showInlineProgress();
  installProgressPhase = "download";
  showDownloadButtons();

  if (version === "MuMu12-V6") {
    await doV6OnlineInstall(version, downloadDir, installDir);
  } else {
    await doV4V5OnlineInstall(version, path, downloadDir, installDir);
  }
}

async function doV6OnlineInstall(version, downloadDir, installDir) {
  const nxmainLink = document.getElementById("v6-link-nxmain").value.trim();
  const nemuxLink = document.getElementById("v6-link-nemux").value.trim();
  const mumu15Link = document.getElementById("v6-link-mumu15").value.trim();

  if (!nxmainLink && !nemuxLink && !mumu15Link) {
    showToast("请至少输入一个 V6 组件链接，或点击 [获取] 按钮自动填充", "error");
    hideInlineProgress();
    return;
  }

  let components = [];
  let productVersion = window._v6Version || "6.0.0.0";

  if (window._v6Components && window._v6Components.length > 0) {
    components = window._v6Components
      .filter(c => {
        const name = c.name.toUpperCase();
        if (name === "NXMAIN" && nxmainLink) return true;
        if (name === "NEMUX" && nemuxLink) return true;
        if (name === "MUMU15" && mumu15Link) return true;
        return false;
      })
      .map(c => {
        const name = c.name.toUpperCase();
        let link = c.link;
        if (name === "NXMAIN" && nxmainLink) link = nxmainLink;
        if (name === "NEMUX" && nemuxLink) link = nemuxLink;
        if (name === "MUMU15" && mumu15Link) link = mumu15Link;
        return { ...c, link };
      });
  }

  if (components.length === 0) {
    if (nxmainLink) components.push({ name: "NXMAIN", version: productVersion, checksum: "", link: nxmainLink, size: 0 });
    if (nemuxLink) components.push({ name: "NEMUX", version: productVersion, checksum: "", link: nemuxLink, size: 0 });
    if (mumu15Link) components.push({ name: "MUMU15", version: productVersion, checksum: "", link: mumu15Link, size: 0 });
  }

  window._v6Components = components;
  window._v6Version = productVersion;

  addLog(`V6 在线安装: ${components.length} 个组件, product_version=${productVersion}`);

  try {
    downloadState.active = true;
    downloadState.version = version;
    downloadState.downloadDir = downloadDir;
    downloadState.installDir = installDir;
    downloadState.components = components;
    downloadState.productVersion = productVersion;

    const downloadedFilesJson = await invoke("download_v6_installer", {
      saveDir: downloadDir,
      componentsJson: JSON.stringify(components),
      resume: false,
    });
    addLog(`V6 所有组件下载完成`);

    installProgressPhase = "install";
    document.getElementById("install-progress-actions").style.display = "none";
    updateInlineProgress(60, "安装中...", "正在静默安装，请稍候...");
    startSimulatedInstallProgress();

    const result = await invoke("install_v6_setup", {
      componentsJson: JSON.stringify(components),
      downloadedFilesJson: downloadedFilesJson,
      installDir,
      productVersion,
    });
    finishInstallProgress();
    showToast(`${version} 安装成功！`, "success");
    addLog(`${version} 安装成功: ${result}`);
    downloadState.active = false;
    refreshMuMuInfoAfterInstall();
  } catch (e) {
    handleInstallError(e);
  }
}

async function doV4V5OnlineInstall(version, path, downloadDir, installDir) {
  if (!path) {
    showToast("输入安装包下载链接，或点击 [获取] 按钮自动填充", "error");
    hideInlineProgress();
    return;
  }

  downloadState.active = true;
  downloadState.version = version;
  downloadState.downloadDir = downloadDir;
  downloadState.installDir = installDir;
  downloadState.url = path;

  try {
    const installerPath = await invoke("download_v4_installer", {
      saveDir: downloadDir,
      url: path,
      version,
      resume: false,
    });
    addLog(`${version} 安装包下载完成: ${installerPath}`);
    showToast("下载完成，正在静默安装...", "info");

    installProgressPhase = "install";
    document.getElementById("install-progress-actions").style.display = "none";
    updateInlineProgress(60, "安装中...", "正在静默安装，请稍候...");
    startSimulatedInstallProgress();

    const result = await invoke("install_v4_setup", { installerPath, installDir });
    finishInstallProgress();
    showToast(`${version} 安装成功！`, "success");
    addLog(`${version} 安装成功: ${result}`);
    downloadState.active = false;
    refreshMuMuInfoAfterInstall();
  } catch (e) {
    handleInstallError(e);
  }
}

// 安装方式 UI 切换

export function updateInstallMethodUI() {
  const version = document.getElementById("select-version").value;
  const method = document.getElementById("select-method").value;
  const pathInput = document.getElementById("install-path");
  const actionBtn = document.getElementById("btn-path-action");
  const v6LinksGroup = document.getElementById("v6-links-group");
  const pathGroup = document.getElementById("path-group");
  const v6FetchActions = document.getElementById("v6-fetch-actions");

  pathInput.value = "";
  document.getElementById("v6-link-nxmain").value = "";
  document.getElementById("v6-link-nemux").value = "";
  document.getElementById("v6-link-mumu15").value = "";

  if (version === "MuMu12-V6") {
    pathGroup.style.display = "none";
    v6LinksGroup.style.display = "";

    const browseNxmain = document.getElementById("btn-v6-browse-nxmain");
    const browseNemux = document.getElementById("btn-v6-browse-nemux");
    const browseMumu15 = document.getElementById("btn-v6-browse-mumu15");

    if (method === "local") {
      document.getElementById("v6-link-nxmain").placeholder = "输入 NXMAIN 安装器文件路径";
      document.getElementById("v6-link-nemux").placeholder = "输入 NEMUX 安装器文件路径";
      document.getElementById("v6-link-mumu15").placeholder = "输入 MUMU15 安装器文件路径";
      v6FetchActions.style.display = "none";
      browseNxmain.style.display = "";
      browseNemux.style.display = "";
      browseMumu15.style.display = "";
      addLog("切换安装方式: 本地安装");
    } else {
      document.getElementById("v6-link-nxmain").placeholder = "输入 NXMAIN 下载链接";
      document.getElementById("v6-link-nemux").placeholder = "输入 NEMUX 下载链接";
      document.getElementById("v6-link-mumu15").placeholder = "输入 MUMU15 下载链接";
      v6FetchActions.style.display = "";
      browseNxmain.style.display = "none";
      browseNemux.style.display = "none";
      browseMumu15.style.display = "none";
      addLog("切换安装方式: 在线安装");
    }
  } else {
    v6LinksGroup.style.display = "none";
    pathGroup.style.display = "";
    const browseBtn = document.getElementById("btn-browse-file");
    if (method === "local") {
      pathInput.disabled = false;
      pathInput.placeholder = "输入本地安装包路径";
      actionBtn.style.display = "none";
      browseBtn.style.display = "";
      addLog("切换安装方式: 本地安装");
    } else {
      pathInput.disabled = false;
      pathInput.placeholder = "输入下载链接";
      actionBtn.style.display = "";
      browseBtn.style.display = "none";
      actionBtn.textContent = "获取";
      actionBtn.onclick = fetchDownloadUrl;
      addLog("切换安装方式: 在线安装");
    }
  }
}

// 下载进度事件处理

export function updateProgress(data) {
  const { status, action, percent, downloaded, total } = data;

  if (installProgressPhase === "download") {
    const totalPct = Math.round((percent || 0) * 0.6);
    const detail = total > 0 ? `${formatSize(downloaded)} / ${formatSize(total)}` : "";
    updateInlineProgress(totalPct, action || "下载中...", detail);
  } else if (installProgressPhase === "install") {
    if (action) {
      updateInlineProgress(totalProgressPct, action, "");
    }
  }
}

// 事件绑定

export function initInstallPage() {
  document.getElementById("btn-install").addEventListener("click", startInstall);
  document.getElementById("btn-browse-dir").addEventListener("click", browseDirectory);
  document.getElementById("btn-browse-file").addEventListener("click", browseFile);
  document.getElementById("btn-v6-browse-nxmain").addEventListener("click", () => browseV6File("v6-link-nxmain"));
  document.getElementById("btn-v6-browse-nemux").addEventListener("click", () => browseV6File("v6-link-nemux"));
  document.getElementById("btn-v6-browse-mumu15").addEventListener("click", () => browseV6File("v6-link-mumu15"));
  document.getElementById("btn-v6-fetch").addEventListener("click", fetchDownloadUrl);
  document.getElementById("btn-pause-download").addEventListener("click", pauseOrResumeDownload);
  document.getElementById("btn-cancel-download").addEventListener("click", cancelDownload);

  document.getElementById("select-version").addEventListener("change", updateInstallMethodUI);
  document.getElementById("select-method").addEventListener("change", updateInstallMethodUI);

  updateInstallMethodUI();
}