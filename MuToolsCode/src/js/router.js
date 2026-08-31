// 页面路由
import { loadBackendLogs } from "./logger.js";
import { loadSettings, loadAdminStatus } from "./settings.js";
import { loadPackages } from "./packages.js";
import { loadMuMuInfo } from "./mumu-info.js";
import { renderOptimizePage } from "./optimize.js";

export function switchPage(page) {
  document.querySelectorAll(".nav-item").forEach((item) => {
    item.classList.toggle("active", item.dataset.page === page);
  });
  document.querySelectorAll(".page").forEach((p) => {
    p.classList.toggle("active", p.id === `page-${page}`);
  });

  switch (page) {
    case "logs":
      loadBackendLogs();
      break;
    case "settings":
      loadSettings();
      loadAdminStatus();
      break;
    case "packages":
      loadPackages();
      break;
    case "mumu-info":
      loadMuMuInfo();
      break;
    case "optimize":
      renderOptimizePage();
      break;
  }
}