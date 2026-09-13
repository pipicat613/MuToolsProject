// 帮助页面

// 远程页面 URL 映射
const helpRemoteUrls = {
  official: "https://mumu.163.com/help/win/",
  support: "https://mumu.163.com/redirect/customerservice"
};

// 记录已加载的 iframe
const helpLoadedIframes = { official: false, support: false };

const helpFaqData = [
  {
    category: "安装问题",
    question: "安装失败怎么办？",
    answer: `1. 查看 <strong>日志</strong> 页面中的错误信息<br>
2. 确保安装目录有充足的存储空间<br>
3. 确认应用拥有管理员权限<br>
4. 尝试使用官方安装包和安装器安装<br>
5. 联系 <strong>官方客服</strong> 以寻求帮助`
  },
  {
    category: "优化问题",
    question: "我该选择哪个优化方案？",
    answer: `推荐使用 仿专版/海外版优化 + 修改startupImage文件夹`
  },
  {
    category: "优化问题",
    question: "导入data包失败怎么解决？",
    answer: `请确认模拟器版本在V4.0.0.3179及以上，过低版本暂不支持调用MuMuManager`
  },
  {
    category: "安装问题",
    question: "请求下载链接失败怎么办？",
    answer: `请求API时会走系统代理<br>
1. 请确认未使用host优化方案，如有请撤销优化<br>
2. 请确认host文件未重定向API地址至127.0.0.1或其他无法访问的地址<br>
3. 请确认已关闭其他代理工具`
  },
  {
    category: "常见问题",
    question: "为什么需要管理员权限？",
    answer: `MuTools 的 <strong>大量功能</strong> 需要管理员权限才能正常运行<br>
非特殊情况请务必给予软件管理员权限`
  },
  {
    category: "常见问题",
    question: "如何安装资源包？",
    answer: `MuTools的 项目地址README.md 里有相关链接`
  },
  {
    category: "常见问题",
    question: "MuMu模拟器经常卡死、闪退怎么办？",
    answer: `1.查看官方文档，寻找是否有解决方案<br>
2.将MuMu升级到最新版<br>
2.将MuMu回退到旧版本<br>
4.联系 <strong>官方客服</strong> 以寻求帮助`
  },
  {
    category: "安装问题",
    question: "MuMu模拟器安装卡98%怎么办？",
    answer: `安装进度条是模拟的，卡98%意味着安装未完成`
  }
];

const helpCategories = [...new Set(helpFaqData.map(item => item.category))];

function filterByCategory(category) {
  document.querySelectorAll(".help-category-tag").forEach(t => {
    t.classList.toggle("active", t.dataset.category === category);
  });
  const searchInput = document.getElementById("help-search-input");
  if (searchInput) searchInput.value = "";
  document.getElementById("help-search-clear").style.display = "none";
  document.getElementById("help-no-results").style.display = "none";

  if (category === "all") {
    renderFaqList(helpFaqData);
  } else {
    const filtered = helpFaqData.filter(item => item.category === category);
    renderFaqList(filtered);
  }
}

function performSearch(query) {
  const results = helpFaqData.filter(item => {
    return item.question.toLowerCase().includes(query) ||
      item.category.toLowerCase().includes(query) ||
      item.answer.toLowerCase().includes(query);
  });

  document.getElementById("help-no-results").style.display = results.length === 0 ? "" : "none";

  if (results.length > 0) {
    renderFaqList(results, query);
  }
}

function renderFaqList(items, highlightQuery) {
  const container = document.getElementById("help-faq-list");
  if (!container) return;
  container.innerHTML = "";

  items.forEach(item => {
    const faqItem = document.createElement("div");
    faqItem.className = "help-faq-item";

    const question = document.createElement("div");
    question.className = "help-faq-question";

    const qText = document.createElement("span");
    qText.innerHTML = highlightQuery
      ? highlightText(item.question, highlightQuery)
      : item.question;
    question.appendChild(qText);

    const rightGroup = document.createElement("span");
    rightGroup.className = "help-faq-right";

    const catTag = document.createElement("span");
    catTag.className = "help-faq-category";
    catTag.textContent = item.category;
    rightGroup.appendChild(catTag);

    const arrow = document.createElement("span");
    arrow.className = "help-faq-arrow";
    arrow.textContent = "▼";
    rightGroup.appendChild(arrow);

    question.appendChild(rightGroup);

    question.addEventListener("click", () => {
      faqItem.classList.toggle("open");
    });

    const answer = document.createElement("div");
    answer.className = "help-faq-answer";
    answer.innerHTML = highlightQuery
      ? highlightText(item.answer, highlightQuery)
      : item.answer;

    faqItem.appendChild(question);
    faqItem.appendChild(answer);
    container.appendChild(faqItem);
  });
}

function highlightText(text, query) {
  if (!query) return text;
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escaped})`, 'gi');
  return text.replace(regex, '<span class="help-highlight">$1</span>');
}

export function initHelpPage() {
  // Help tab switching
  document.querySelectorAll(".help-tab").forEach(tab => {
    tab.addEventListener("click", () => {
      const tabName = tab.dataset.helpTab;
      document.querySelectorAll(".help-tab").forEach(t => t.classList.remove("active"));
      tab.classList.add("active");
      document.querySelectorAll(".help-panel").forEach(p => p.classList.remove("active"));
      document.getElementById(`help-panel-${tabName}`).classList.add("active");
    });
  });

  // Render categories
  const categoriesEl = document.getElementById("help-categories");
  if (categoriesEl) {
    const allTag = document.createElement("button");
    allTag.className = "help-category-tag active";
    allTag.textContent = "全部";
    allTag.dataset.category = "all";
    allTag.addEventListener("click", () => filterByCategory("all"));
    categoriesEl.appendChild(allTag);

    helpCategories.forEach(cat => {
      const tag = document.createElement("button");
      tag.className = "help-category-tag";
      tag.textContent = cat;
      tag.dataset.category = cat;
      tag.addEventListener("click", () => filterByCategory(cat));
      categoriesEl.appendChild(tag);
    });
  }

  // Render FAQ list
  renderFaqList(helpFaqData);

  // Search
  const searchInput = document.getElementById("help-search-input");
  const searchClear = document.getElementById("help-search-clear");
  if (searchInput) {
    searchInput.addEventListener("input", () => {
      const query = searchInput.value.trim().toLowerCase();
      searchClear.style.display = query ? "" : "none";
      if (query) {
        document.querySelectorAll(".help-category-tag").forEach(t => {
          t.classList.toggle("active", t.dataset.category === "all");
        });
        performSearch(query);
      } else {
        renderFaqList(helpFaqData);
        document.getElementById("help-no-results").style.display = "none";
      }
    });
  }
  if (searchClear) {
    searchClear.addEventListener("click", () => {
      searchInput.value = "";
      searchClear.style.display = "none";
      renderFaqList(helpFaqData);
      document.getElementById("help-no-results").style.display = "none";
      document.querySelectorAll(".help-category-tag").forEach(t => {
        t.classList.toggle("active", t.dataset.category === "all");
      });
    });
  }

  // 远程页面 懒加载
  document.querySelectorAll(".help-open-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      const type = btn.dataset.helpOpen;   // "official" | "support"
      const mode = btn.dataset.helpMode;   // "embed" | "browser"
      const url = helpRemoteUrls[type];
      if (!url) return;

      if (mode === "browser") {
        window.__TAURI__.opener.openUrl(url).catch(err => {
          console.error("无法在浏览器中打开:", err);
        });
      } else if (mode === "embed") {
        const placeholder = document.getElementById(`help-placeholder-${type}`);
        const iframeWrap = document.getElementById(`help-iframe-wrap-${type}`);
        const iframe = document.getElementById(`help-iframe-${type}`);
        if (placeholder) placeholder.style.display = "none";
        if (iframeWrap) iframeWrap.style.display = "";
        if (iframe && !helpLoadedIframes[type]) {
          iframe.src = url;
          helpLoadedIframes[type] = true;
        }
      }
    });
  });
}