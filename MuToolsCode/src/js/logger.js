// 日志模块

const { invoke } = window.__TAURI__.core;

let allLogEntries = [];

export function getLogEntries() {
  return allLogEntries;
}

export function formatTime() {
  const d = new Date();
  const pad = (n) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

export function addLog(message, source = "前端") {
  const time = formatTime();
  const entry = `[${time}] [${source}] ${message}`;
  allLogEntries.push(entry);
  renderMergedLogs();
  invoke("log_action", { message: `[${source}] ${message}` }).catch(() => { });
}

export function renderMergedLogs() {
  const container = document.getElementById("merged-logs");
  if (!container) return;
  container.innerHTML = "";
  if (allLogEntries.length === 0) {
    container.textContent = "暂无日志";
    return;
  }
  allLogEntries.forEach((entry) => {
    const div = document.createElement("div");
    div.className = "log-entry";
    div.textContent = entry;
    container.appendChild(div);
  });
  container.scrollTop = container.scrollHeight;
}

export async function loadBackendLogs() {
  try {
    const logs = await invoke("get_logs");
    if (logs && logs.length > 0) {
      logs.forEach((entry) => {
        if (!allLogEntries.some((e) => e === entry)) {
          allLogEntries.push(entry);
        }
      });
      renderMergedLogs();
    }
  } catch (e) {
    console.error("加载后端日志失败:", e);
  }
}

export function clearLogs() {
  allLogEntries = [];
  renderMergedLogs();
  invoke("clear_logs").catch(() => { });
  addLog("日志已清空");
}