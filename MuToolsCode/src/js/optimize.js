// 优化页面
import { showToast } from "./toast.js";
import { addLog } from "./logger.js";
const { invoke } = window.__TAURI__.core;

const optimizeGroups = [
  {
    id: "special-edition",
    title: "仿专版/海外版修改",
    items: [
      {
        id: "overlay-4x",
        title: "[4.x]仿专版修改",
        command: "apply_overlay_package",
        undoCommand: "undo_overlay_package",
        needsResource: "overlay",
        pros: ["修改为专版，解锁了专版模拟器功能限制，体积小，功能强大，去除了搜索栏"],
        cons: ["可能对旧版data数据有兼容性问题，需要重新新建多开"],
        notes: ["提取与修改专版模拟器的overlay文件制成，overlay修改对所有多开有效"],
      },
      {
        id: "fchannel-5x",
        title: "[5.x][6.x]版本仿专版修改",
        command: "apply_fchannel",
        undoCommand: "undo_fchannel",
        pros: ["修改为专版，几乎不影响原版体验"],
        cons: [],
        notes: ["修改安装配置信息，普通版仅修改此参数不会自动下载游戏"],
      },
      {
        id: "patch-vdi-overseas",
        title: "[5.x][6.x]修改system.vdi",
        command: "patch_system_vdi_overseas",
        undoCommand: "undo_patch_system_vdi_overseas",
        pros: ["修改system.vdi添加海外版标识"],
        cons: ["将会临时占用1-2G内存资源"],
        notes: ["直接在system.vdi中修改build.prod"],
      }
    ],
  },
  {
    id: "startup-ad",
    title: "开屏广告",
    items: [
      {
        id: "startup-image",
        title: "修改startupImage文件夹",
        command: "disable_startup_image",
        undoCommand: "restore_startup_image",
        pros: ["取消开屏广告"],
        cons: ["无法显示MuMu活动"],
        notes: ["可以通过游戏中心进入活动"],
      },
      {
        id: "hosts-update",
        title: "host代理127",
        command: "block_update_domains",
        undoCommand: "unblock_update_domains",
        pros: ["取消消息中心、模拟器更新"],
        cons: ["无法使用官网下载更新、在线安装包"],
        notes: ["不会影响模拟器内部网络环境"],
      },
    ],
  },
  {
    id: "message‌-ad",
    title: "消息中心广告",
    items: [
      {
        id: "hosts-update",
        title: "host代理127",
        command: "block_update_domains",
        undoCommand: "unblock_update_domains",
        pros: ["取消消息中心、模拟器更新"],
        cons: ["无法使用官网下载更新、在线安装包"],
        notes: ["不会影响模拟器内部网络环境"],
      },
    ],
  },
  {
    id: "desktop-ad",
    title: "模拟器内桌面广告",
    items: [
      {
        id: "data-package",
        title: "使用data包",
        command: "import_data_package",
        undoCommand: "undo_import_data_package",
        needsResource: "data",
        pros: ["取消模拟器桌面广告"],
        cons: ["多开应用的图标无法显示在桌面，可通多应用多开软件进入"],
        notes: [],
      },
    ],
  },
  {
    id: "block-update",
    title: "禁止更新",
    items: [
      {
        id: "hosts-update-2",
        title: "host代理127",
        command: "block_update_domains",
        undoCommand: "unblock_update_domains",
        pros: ["取消消息中心、模拟器更新"],
        cons: ["无法使用官网下载更新、在线安装包"],
        notes: ["不会影响模拟器内部网络环境"],
      },
      {
        id: "disable-updater",
        title: "修改更新文件",
        command: "disable_updater",
        undoCommand: "restore_updater",
        pros: ["禁用模拟器更新"],
        cons: ["无法体验新版"],
        notes: ["把MuMuPlayerUpdater.exe修改为MuMuPlayerUpdater.exe.bak"],
      },
    ],
  },
  {
    id: "telemetry",
    title: "MuMu遥测",
    items: [
      {
        id: "block-report",
        title: "全局代理127(修改host)",
        command: "block_report_domains",
        undoCommand: "unblock_report_domains",
        pros: ["禁用MuMu遥测"],
        cons: ["错误日志上报失败"],
        notes: ["禁用范围包括但不限于MuMu12遥测"],
      },
    ],
  },
];

const optimizeFilePaths = {};

export function renderOptimizePage() {
  const accordion = document.getElementById("optimize-accordion");
  if (!accordion || accordion.dataset.rendered) return;
  accordion.innerHTML = "";

  const faqList = document.createElement("div");
  faqList.className = "optimize-faq-list";

  optimizeGroups.forEach((group) => {
    const faqItem = document.createElement("div");
    faqItem.className = "optimize-faq-item";

    const question = document.createElement("div");
    question.className = "optimize-faq-question";
    question.innerHTML = `
      <span>${group.title}</span>
      <div class="optimize-faq-right">
        <span class="opt-group-count">${group.items.length} 项</span>
        <span class="optimize-faq-arrow">▼</span>
      </div>
    `;
    question.addEventListener("click", () => { faqItem.classList.toggle("open"); });
    faqItem.appendChild(question);

    const answer = document.createElement("div");
    answer.className = "optimize-faq-answer";

    group.items.forEach((item) => {
      const row = document.createElement("div");
      row.className = "opt-item";

      const left = document.createElement("div");
      left.className = "opt-item-left";

      const topRow = document.createElement("div");
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.className = "opt-item-checkbox";
      cb.dataset.groupId = group.id;
      cb.dataset.itemId = item.id;
      cb.addEventListener("change", () => onOptItemCheckChanged(cb, item));
      topRow.appendChild(cb);

      const label = document.createElement("span");
      label.className = "opt-item-title";
      label.textContent = item.title;
      topRow.appendChild(label);
      left.appendChild(topRow);

      if (item.needsResource) {
        const fileRow = document.createElement("div");
        fileRow.className = "opt-item-file";
        fileRow.style.display = "none";
        const select = document.createElement("select");
        select.className = "form-input opt-resource-select";
        select.dataset.itemId = item.id;
        select.innerHTML = '<option value="">加载中...</option>';
        select.disabled = true;
        fileRow.appendChild(select);
        left.appendChild(fileRow);
      }

      const detail = document.createElement("div");
      detail.className = "opt-item-detail";
      if (item.pros.length > 0) {
        item.pros.forEach((p) => {
          const d = document.createElement("div");
          d.className = "opt-detail opt-detail-pro";
          d.textContent = `+ ${p}`;
          detail.appendChild(d);
        });
      }
      if (item.cons.length > 0) {
        item.cons.forEach((c) => {
          const d = document.createElement("div");
          d.className = "opt-detail opt-detail-con";
          d.textContent = `- ${c}`;
          detail.appendChild(d);
        });
      }
      if (item.notes.length > 0) {
        item.notes.forEach((n) => {
          const d = document.createElement("div");
          d.className = "opt-detail opt-detail-note";
          d.textContent = `* ${n}`;
          detail.appendChild(d);
        });
      }
      left.appendChild(detail);

      row.appendChild(left);
      answer.appendChild(row);
    });

    faqItem.appendChild(answer);
    faqList.appendChild(faqItem);
  });

  accordion.appendChild(faqList);
  accordion.dataset.rendered = "true";
}

function onOptItemCheckChanged(cb, item) {
  if (item.needsResource) {
    const fileRow = cb.closest(".opt-item-left").querySelector(".opt-item-file");
    if (fileRow) {
      fileRow.style.display = cb.checked ? "flex" : "none";
      if (cb.checked) {
        loadOptimizeResources(item, fileRow);
      } else {
        delete optimizeFilePaths[item.id];
        const select = fileRow.querySelector(".opt-resource-select");
        if (select) select.value = "";
      }
    }
  }
}

async function loadOptimizeResources(item, fileRow) {
  const select = fileRow.querySelector(".opt-resource-select");
  if (!select) return;
  const cmdMap = { data: "list_data_optimize_packages", overlay: "list_overlay_optimize_packages" };
  const cmd = cmdMap[item.needsResource] || "list_data_optimize_packages";
  try {
    const list = await invoke(cmd);
    select.innerHTML = "";
    if (!list || list.length === 0) {
      select.innerHTML = '<option value="">未找到匹配的资源包</option>';
      select.disabled = true;
      return;
    }
    select.innerHTML = '<option value="">请选择资源包...</option>';
    list.forEach((pkg) => {
      const opt = document.createElement("option");
      opt.value = pkg.filePath;
      let label = pkg.displayName || pkg.packageName;
      if (pkg.version) label += ` v${pkg.version}`;
      if (pkg.fileName) label += ` (${pkg.fileName})`;
      opt.textContent = label;
      select.appendChild(opt);
    });
    select.disabled = false;
    select.addEventListener("change", () => {
      if (select.value) {
        optimizeFilePaths[item.id] = select.value;
      } else {
        delete optimizeFilePaths[item.id];
      }
    });
  } catch (e) {
    select.innerHTML = '<option value="">加载失败</option>';
    select.disabled = true;
    addLog(`加载资源包列表失败: ${e}`);
  }
}

function getCheckedItems() {
  const checked = [];
  document.querySelectorAll(".opt-item-checkbox:checked").forEach((cb) => {
    const groupId = cb.dataset.groupId;
    const itemId = cb.dataset.itemId;
    const group = optimizeGroups.find((g) => g.id === groupId);
    if (!group) return;
    const item = group.items.find((i) => i.id === itemId);
    if (!item) return;
    checked.push({ group, item });
  });
  return checked;
}

async function runOptimizeBatch(mode) {
  const checked = getCheckedItems();
  if (checked.length === 0) {
    showToast("请至少勾选一项优化", "info");
    return;
  }

  const label = mode === "apply" ? "开始优化" : "开始撤销优化";
  addLog(`========== ${label} ==========`);
  showToast(label, "info");

  for (const { item } of checked) {
    const command = mode === "apply" ? item.command : item.undoCommand;
    if (!command) continue;

    const args = {};
    if (mode === "apply" && item.needsResource) {
      const filePath = optimizeFilePaths[item.id];
      if (!filePath) {
        addLog(`跳过「${item.title}」：未选择资源包`);
        showToast(`跳过「${item.title}」：未选择资源包`, "info");
        continue;
      }
      args.path = filePath;
    }

    try {
      addLog(`执行: ${item.title}`);
      showToast(`正在执行: ${item.title}`, "info");
      const result = await invoke(command, args);
      addLog(`成功: ${result}`);
    } catch (e) {
      addLog(`失败: ${item.title} ${e}`);
      showToast(`${item.title} 失败: ${e}`, "error");
    }
  }

  addLog(`========== ${label} 完成 ==========`);
  showToast(`${label} 完成`, "success");
}

export function initOptimizePage() {
  const btnOptimizeStart = document.getElementById("btn-optimize-start");
  if (btnOptimizeStart) {
    btnOptimizeStart.addEventListener("click", () => runOptimizeBatch("apply"));
  }
  const btnOptimizeCancel = document.getElementById("btn-optimize-cancel");
  if (btnOptimizeCancel) {
    btnOptimizeCancel.addEventListener("click", () => runOptimizeBatch("cancel"));
  }
}