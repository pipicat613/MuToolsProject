// 资源包管理页面
import { showToast, formatSize, escapeHtml } from "./toast.js";
import { addLog } from "./logger.js";
const { invoke } = window.__TAURI__.core;

let pendingDelete = null; // { packageName, descFile?, descName? }

// 删除确认

export function showDeleteConfirm(packageName, descFile, descName) {
  pendingDelete = { packageName, descFile: descFile || null, descName: descName || null };
  const msg = descFile
    ? `确定删除子包 "${descName || descFile}" 吗？`
    : `确定删除资源包 "${packageName}" 吗？`;
  document.getElementById("modal-delete-message").textContent = msg;
  document.getElementById("modal-delete-overlay").style.display = "flex";
}

export function hideDeleteConfirm() {
  pendingDelete = null;
  document.getElementById("modal-delete-overlay").style.display = "none";
}

async function confirmDeletePackage() {
  const del = pendingDelete;
  if (!del) return;
  const confirmBtn = document.getElementById("btn-delete-confirm");
  confirmBtn.disabled = true;
  try {
    if (del.descFile) {
      await invoke("delete_sub_package", { packageName: del.packageName, descFile: del.descFile });
      addLog(`已删除子包: ${del.descName || del.descFile}`);
    } else {
      await invoke("delete_resource_package", { name: del.packageName });
      addLog(`已删除资源包: ${del.packageName}`);
    }
    hideDeleteConfirm();
    await loadPackages();
  } catch (e) {
    showToast(`删除失败: ${e}`, "error");
    addLog(`删除失败: ${e}`);
  } finally {
    confirmBtn.disabled = false;
  }
}

// 加载资源包列表

export async function loadPackages() {
  try {
    const list = await invoke("list_resource_packages");
    const container = document.getElementById("pkg-list");
    container.innerHTML = "";
    if (!list || list.length === 0) {
      const empty = document.createElement("div");
      empty.className = "pkg-empty";
      empty.textContent = "暂无资源包";
      container.appendChild(empty);
      return;
    }
    list.forEach((pkg) => {
      const card = document.createElement("div");
      card.className = "pkg-card";

      const header = document.createElement("div");
      header.className = "pkg-card-header";

      const hasSubItems = pkg.descInfos && pkg.descInfos.length > 0;

      const arrow = document.createElement("span");
      arrow.className = "pkg-expand-arrow";
      arrow.textContent = hasSubItems ? "▼" : "✕";
      if (!hasSubItems) {
        arrow.style.opacity = "0.3";
        arrow.style.cursor = "default";
      }
      header.appendChild(arrow);

      const info = document.createElement("div");
      info.className = "pkg-info";

      const name = document.createElement("div");
      name.className = "pkg-name";
      name.title = pkg.path;
      name.textContent = pkg.manifest.name || pkg.name;
      info.appendChild(name);

      if (pkg.manifest.description) {
        const desc = document.createElement("div");
        desc.className = "pkg-desc";
        desc.textContent = pkg.manifest.description;
        info.appendChild(desc);
      }

      const meta = document.createElement("div");
      meta.className = "pkg-meta";
      let metaParts = [];
      if (hasSubItems) {
        metaParts.push(`${pkg.descInfos.length} 个子包`);
      } else {
        metaParts.push(pkg.kind);
      }
      metaParts.push(formatSize(pkg.size));
      meta.textContent = metaParts.join(" · ");
      info.appendChild(meta);

      header.appendChild(info);

      const actions = document.createElement("div");
      actions.className = "pkg-actions";

      const openDirBtn = document.createElement("button");
      openDirBtn.className = "btn-secondary";
      openDirBtn.textContent = "打开目录";
      openDirBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        invoke("open_resource_dir", { name: pkg.name })
          .then(() => addLog(`打开资源包目录: ${pkg.name}`))
          .catch((e) => addLog(`打开资源包目录失败: ${e}`));
      });

      const openLinkBtn = document.createElement("button");
      openLinkBtn.className = "btn-secondary";
      openLinkBtn.textContent = "打开链接";
      openLinkBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        const url = pkg.manifest?.webpage;
        if (url) {
          invoke("open_url", { url })
            .then(() => addLog(`打开链接: ${url}`))
            .catch((e) => addLog(`打开链接失败: ${e}`));
        } else {
          showToast("该资源包没有关联链接", "info");
        }
      });

      actions.appendChild(openDirBtn);
      actions.appendChild(openLinkBtn);

      if (!hasSubItems) {
        const deleteBtn = document.createElement("button");
        deleteBtn.className = "btn-secondary btn-danger";
        deleteBtn.textContent = "删除";
        deleteBtn.addEventListener("click", (e) => { e.stopPropagation(); showDeleteConfirm(pkg.name); });
        actions.appendChild(deleteBtn);
      }

      header.appendChild(actions);

      if (hasSubItems) {
        card.classList.add("pkg-card-expandable");
        header.addEventListener("click", () => { card.classList.toggle("open"); });
      }

      card.appendChild(header);

      if (hasSubItems) {
        const body = document.createElement("div");
        body.className = "pkg-card-body";

        pkg.descInfos.forEach((di) => {
          const subItem = document.createElement("div");
          subItem.className = "pkg-sub-item";

          const subItemRow = document.createElement("div");
          subItemRow.className = "pkg-sub-item-row";

          const subInfo = document.createElement("div");
          subInfo.className = "pkg-sub-info";

          const subName = document.createElement("div");
          subName.className = "pkg-sub-name";
          subName.textContent = di.name || "未命名";
          subInfo.appendChild(subName);

          const subMeta = document.createElement("div");
          subMeta.className = "pkg-sub-meta";
          const parts = [];
          if (di.builder) parts.push(di.builder);
          if (di.displayVersion) parts.push(`版本: ${di.displayVersion}`);
          if (di.buildVersion) parts.push(`构建版本: ${di.buildVersion}`);
          if (di.time) parts.push(new Date(di.time * 1000).toLocaleString());
          subMeta.textContent = parts.join(" · ");
          subInfo.appendChild(subMeta);

          subItemRow.appendChild(subInfo);

          const subDeleteBtn = document.createElement("button");
          subDeleteBtn.className = "btn-secondary btn-danger btn-sm";
          subDeleteBtn.textContent = "删除";
          subDeleteBtn.addEventListener("click", (e) => {
            e.stopPropagation();
            showDeleteConfirm(pkg.name, di.descFile, di.name);
          });
          subItemRow.appendChild(subDeleteBtn);

          subItem.appendChild(subItemRow);

          if (di.description) {
            const subDesc = document.createElement("div");
            subDesc.className = "pkg-sub-desc";
            subDesc.textContent = di.description;
            subItem.appendChild(subDesc);
          }

          if (di.fileList && di.fileList.length > 0) {
            const flInfo = document.createElement("div");
            flInfo.className = "pkg-sub-files";
            flInfo.textContent = `文件: ${di.fileList.map(f => `${f.filename} (${formatSize(f.size)})`).join(", ")}`;
            subItem.appendChild(flInfo);
          }

          body.appendChild(subItem);
        });

        card.appendChild(body);
      }

      container.appendChild(card);
    });
  } catch (e) {
    showToast(`加载资源包列表失败: ${e}`, "error");
  }
}

// 安装模态框

function showModal() {
  document.getElementById("modal-install-overlay").style.display = "flex";
  document.getElementById("modal-install-method").value = "local";
  document.getElementById("modal-install-path").value = "";
  document.getElementById("modal-install-url").value = "";
  updateModalInstallMethodUI();
}

function hideModal() {
  document.getElementById("modal-install-overlay").style.display = "none";
}

function updateModalInstallMethodUI() {
  const method = document.getElementById("modal-install-method").value;
  const pathGroup = document.getElementById("modal-install-path").closest(".form-group");
  const urlGroup = document.getElementById("modal-install-url").closest(".form-group");
  pathGroup.style.display = method === "local" ? "" : "none";
  urlGroup.style.display = method === "online" ? "" : "none";
}

async function browseFile() {
  try {
    const path = await invoke("browse_file");
    if (path) {
      document.getElementById("modal-install-path").value = path;
      addLog(`选择本地文件: ${path}`);
    }
  } catch (e) {
    addLog(`选择文件失败: ${e}`);
  }
}

async function doInstall() {
  const btn = document.getElementById("btn-modal-install");
  btn.disabled = true;

  try {
    const method = document.getElementById("modal-install-method").value;
    const path = document.getElementById("modal-install-path").value.trim();
    const url = document.getElementById("modal-install-url").value.trim();

    if (method === "local" && path) {
      hideModal();
      showToast("正在安装本地资源包...", "info");
      addLog(`安装本地资源包: ${path}`);
      const pkg = await invoke("install_resource_from_path", { sourcePath: path });
      showToast(`安装成功: ${pkg.manifest.name || pkg.name}`, "success");
      addLog(`资源包安装成功: ${pkg.manifest.name || pkg.name}`);
      await loadPackages();
    } else if (method === "online" && url) {
      hideModal();
      showToast("正在下载并安装...", "info");
      addLog(`下载资源包: ${url}`);
      const pkg = await invoke("install_resource_from_url", { url });
      showToast(`安装成功: ${pkg.manifest.name || pkg.name}`, "success");
      addLog(`资源包下载安装成功: ${pkg.manifest.name || pkg.name}`);
      await loadPackages();
    } else {
      showToast(method === "local" ? "选择本地资源包文件" : "输入远程 URL", "error");
      btn.disabled = false;
    }
  } catch (e) {
    showToast(`安装失败: ${e}`, "error");
    addLog(`资源包安装失败: ${e}`);
  }
  btn.disabled = false;
}



// 刷新列表

export function refreshPackages() {
  addLog("刷新资源包列表");
  loadPackages();
}

// 事件绑定

export function initPackagesPage() {
  document.getElementById("btn-pkg-install").addEventListener("click", showModal);
  document.getElementById("btn-pkg-refresh").addEventListener("click", refreshPackages);
  document.getElementById("btn-pkg-open-folder").addEventListener("click", async () => {
    try { await invoke("open_resource_dir", { name: "" }); }
    catch (e) { addLog(`打开资源目录失败: ${e}`); }
  });

  // Modal
  document.getElementById("btn-modal-close").addEventListener("click", hideModal);
  document.getElementById("btn-modal-cancel").addEventListener("click", hideModal);
  document.getElementById("modal-install-method").addEventListener("change", updateModalInstallMethodUI);
  document.getElementById("btn-modal-browse-file").addEventListener("click", browseFile);
  document.getElementById("btn-modal-install").addEventListener("click", doInstall);
  document.getElementById("modal-install-overlay").addEventListener("click", (e) => {
    if (e.target === e.currentTarget) hideModal();
  });

  // Delete modal
  document.getElementById("btn-delete-close").addEventListener("click", hideDeleteConfirm);
  document.getElementById("btn-delete-cancel").addEventListener("click", hideDeleteConfirm);
  document.getElementById("btn-delete-confirm").addEventListener("click", confirmDeletePackage);
  document.getElementById("modal-delete-overlay").addEventListener("click", (e) => {
    if (e.target === e.currentTarget) hideDeleteConfirm();
  });

  // Detail modal
  document.getElementById("btn-modal-detail-close").addEventListener("click", () => {
    document.getElementById("modal-detail-overlay").style.display = "none";
  });
  document.getElementById("btn-modal-detail-close-bottom").addEventListener("click", () => {
    document.getElementById("modal-detail-overlay").style.display = "none";
  });
  document.getElementById("modal-detail-overlay").addEventListener("click", (e) => {
    if (e.target === e.currentTarget) {
      document.getElementById("modal-detail-overlay").style.display = "none";
    }
  });
}