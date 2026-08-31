// 关于页面

function initDisclaimerModal() {
  const overlay = document.getElementById("modal-disclaimer-overlay");
  if (!overlay) return;

  const btnOpen = document.getElementById("btn-about-disclaimer");
  const btnClose = document.getElementById("btn-disclaimer-close");
  const btnCloseBottom = document.getElementById("btn-disclaimer-close-bottom");
  const linkHome = document.getElementById("link-home-disclaimer");

  function open() { overlay.style.display = "flex"; }
  function close() { overlay.style.display = "none"; }

  if (btnOpen) btnOpen.addEventListener("click", open);
  if (btnClose) btnClose.addEventListener("click", close);
  if (btnCloseBottom) btnCloseBottom.addEventListener("click", close);
  if (linkHome) linkHome.addEventListener("click", open);

  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) close();
  });
}

// 更新日志

const CHANGELOG_DATA = [
  {
    version: "Tauri-v1.0.0",
    date: "2026-08-28",
    items: [
      { text: "首个Tauri正式版发布", type: "plain" },
      { text: "防火墙优化方案被废弃", type: "breaking" },
      { text: "添加免责声明", type: "add" },
      { text: "添加更新日志", type: "add" },
      { text: "修复V6版本的system.vdi优化", type: "fix" },
      { text: "修复防火墙优化项目的部分问题", type: "fix" },
      { text: "调整优化列表分类", type: "opt" },
      { text: "移除项目中死代码和无用资源", type: "opt" },
      { text: "优化资源包管理界面布局", type: "opt" }
    ]
  },
  {
    version: "Tauri-v0.1.0",
    date: "2026-08-25",
    items: [
      { text: "首个Tauri预览版发布", type: "plain" },
      { text: "全新GUI界面设计", type: "add" },
      { text: "支持在线下载MuMu安装包", type: "add" },
      { text: "支持MuMu12-V6版本", type: "add" },
      { text: "添加MuMu帮助项目，便于查找文档和联系客服", type: "add" },
      { text: "新增每日一句和小彩蛋", type: "add" }
    ]
  },
  {
    version: "Python-v5.4",
    date: "0000-00-00",
    items: [
      { text: "最后一个Python版本", type: "plain" },
      { text: "移除资源包MuMu远程控制插件", type: "breaking" },
      { text: "移除了不再受支持的 备份多开Pro 功能", type: "breaking" },
      { text: "资源包模块进行了调整，不再兼容之前版本！", type: "breaking" },
      { text: "新增UU远控官网地址", type: "add" },
      { text: "支持存储和加载资源包状态", type: "add" },
      { text: "新增二维码扫描工具", type: "add" },
      { text: "命令行支持自动加载配置文件", type: "add" },
      { text: "资源管理界面显示优化", type: "opt" },
      { text: "移除了部分无用的提示信息", type: "opt" },
      { text: "MuMu多开管理操作界面优化", type: "opt" }
    ]
  },
  {
    version: "Python-v5.3",
    date: "0000-00-00",
    items: [
      { text: "新增可选5.x仿专版优化渠道类型", type: "add" },
      { text: "新增命令行-v参数", type: "add" },
      { text: "新增修改host自动识别相同域名", type: "add" },
      { text: "Mutools有自己的新图标了", type: "add" },
      { text: "修复禁用遥测失败", type: "fix" },
      { text: "修复日志权限错误", type: "fix" },
      { text: "修复启动时路径检测错误", type: "fix" },
      { text: "程序编译优化，提升性能", type: "opt" },
      { text: "主界面视觉体验优化", type: "opt" },
      { text: "选择优化界面操作体验优化，使用+-*标注", type: "opt" },
      { text: "日志文件上限配置由修改为10MB 90d", type: "opt" },
      { text: "MuMu12版本检测逻辑优化", type: "opt" },
      { text: "调整未安装MuMu12显示警告", type: "opt" },
      { text: "版本号显示调整", type: "opt" },
      { text: "优化模块加载", type: "opt" }
    ]
  },
  {
    version: "Python-v5.2",
    date: "0000-00-00",
    items: [
      { text: "新增system.vdi优化包", type: "add" },
      { text: "支持自动更新", type: "add" },
      { text: "新增关闭MuMu遥测优化项目", type: "add" },
      { text: "新增更新后的更新提示", type: "add" },
      { text: "更新了V5.0版本data优化包", type: "add" },
      { text: "新增提示及日志SUCCESS类型", type: "add" },
      { text: "修复部分情况下命令行字符串解码失败的问题", type: "fix" },
      { text: "修复host更改成功后，意外的错误提示及日志", type: "fix" },
      { text: "修复host恢复的一个提示错误", type: "fix" },
      { text: "优化修改startupImageDir的提示信息", type: "opt" },
      { text: "优化一键优化与一键撤销优化的选项", type: "opt" },
      { text: "优化了更新日志输出显示", type: "opt" },
      { text: "重写了保存设置的代码，提升性能", type: "opt" },
      { text: "优化了输出信息及日志的检测顺序", type: "opt" },
      { text: "精简了反调试代码", type: "opt" },
      { text: "选择资源界面设置了更详细的错误提示", type: "opt" },
      { text: "优化命令行参数识别读取效率", type: "opt" }
    ]
  },
  {
    version: "Python-v5.1",
    date: "0000-00-00",
    items: [
      { text: "新增GUI弹窗提示", type: "add" },
      { text: "新增MuMu国际版下载链接", type: "add" },
      { text: "命令行支持执行优化", type: "add" },
      { text: "支持bitWidth自动获取", type: "add" },
      { text: "修复序号9不能正常查看更新日志的问题", type: "fix" },
      { text: "修复了在自动创建路径中的异常崩溃", type: "fix" },
      { text: "修复在未安装模拟器前导致的无限递归检测版本号", type: "fix" },
      { text: "命令行参数错误将会提示而不是正常启动", type: "opt" },
      { text: "优化了用户协议", type: "opt" }
    ]
  },
  {
    version: "Python-v5.0",
    date: "0000-00-00",
    items: [
      { text: "新增对MuMuPlayerUpdater.exe的自动修改", type: "add" },
      { text: "新增MuMu5.x版本优化", type: "add" },
      { text: "新增MuMu历史版本官网", type: "add" },
      { text: "新增命令行资源包提取工具", type: "add" },
      { text: "新增自动检查更新", type: "add" },
      { text: "新增检测项目vbox-env", type: "add" },
      { text: "新增32位版本", type: "add" },
      { text: "更新5.2和5.5离线安装包", type: "add" },
      { text: "更新5.0.1和5.0.2的在线安装包", type: "add" },
      { text: "更新UU远程软件版本", type: "add" },
      { text: "更新检测工具ColaBoxChecker为MuMuChecker", type: "add" },
      { text: "新增检测MuMu模拟器版本", type: "add" },
      { text: "获取版本号支持读取配置文件", type: "add" },
      { text: "融合Mupro测试版功能", type: "add" },
      { text: "新增详细日志功能", type: "add" },
      { text: "开放DEV调试，命令行支持启用调试模式", type: "add" },
      { text: "修复重复优化时，导致删除startupImage失败的问题", type: "fix" },
      { text: "修复do-2-1任务时5.x版本失败问题", type: "fix" },
      { text: "修复了读取注册表未处理错误的异常崩溃", type: "fix" },
      { text: "修复5.x版本注册表读取异常", type: "fix" },
      { text: "修复5.x版本读取MuMuManager.exe异常", type: "fix" },
      { text: "修复overlay优化包图标异常", type: "fix" },
      { text: "修复在Win1122H2中部分版本中资源识别和安装异常", type: "fix" },
      { text: "修复新版资源包识别为损坏的错误", type: "fix" },
      { text: "修复读取注册表因为重定向而导致报错的问题", type: "fix" },
      { text: "优化下载运行，推出懒人一键包，自带资源解压即用", type: "opt" },
      { text: "优化安装资源包，支持从安全选项卡中复制路径", type: "opt" },
      { text: "优化请求更新地址", type: "opt" },
      { text: "优化界面操作逻辑，去除了不必要的重复Enter", type: "opt" },
      { text: "优化信息输出显示，支持多种颜色显示", type: "opt" },
      { text: "优化了一下请求服务器的User-Agent", type: "opt" },
      { text: "简化资源包管理页面输入内容长度", type: "opt" },
      { text: "删除更改配置时重复显示的[配置已保存]信息", type: "opt" }
    ]
  },
  {
    version: "Python-v4.1",
    date: "0000-00-00",
    items: [
      { text: "新增软件设置", type: "add" },
      { text: "新增MuMu组件安装功能", type: "add" },
      { text: "新增运行环境检查功能", type: "add" },
      { text: "新增对资源包的列表修改", type: "add" },
      { text: "新增全局数据配置", type: "add" },
      { text: "新增对data优化包的描述", type: "add" },
      { text: "新增一键清理多开(MuMu数据)", type: "add" },
      { text: "修复了几个路径不规范报错的问题", type: "fix" },
      { text: "修复导入备份和压缩备份设置7z路径", type: "fix" },
      { text: "修复备份Pro中的多个bug", type: "fix" },
      { text: "移除了安装路径提示", type: "fix" },
      { text: "优化输出显示", type: "opt" },
      { text: "优化代码重复内容", type: "opt" },
      { text: "优化各项按键描述内容", type: "opt" },
      { text: "优化创建的不必要目录文件", type: "opt" }
    ]
  },
  {
    version: "Python-v4.0",
    date: "0000-00-00",
    items: [
      { text: "新增一键备份、增强备份功能", type: "add" },
      { text: "新增模块化资源包设计", type: "add" },
      { text: "新增一键撤销优化", type: "add" },
      { text: "新增可选优化项目", type: "add" },
      { text: "新增程序反篡改", type: "add" },
      { text: "新增更多优化方式及操作", type: "add" },
      { text: "更新并重构之前的shit山代码", type: "add" },
      { text: "更新各类MuMu相关资源版本至2025-04-27，修复一些奇妙的旧版bug", type: "add" },
      { text: "修复由于模拟器名称导致的导出报错，可以通过修改模拟器名称恢复正常", type: "fix" },
      { text: "优化界面设计", type: "opt" },
      { text: "优化资源包发布模式", type: "opt" },
      { text: "优化更新版本时发布的公告", type: "opt" },
      { text: "优化更新模式(由于网盘改版现已不支持自动下载更新包)", type: "opt" },
      { text: "优化host修改，解决修改失败和重复添加等问题", type: "opt" }
    ]
  }
];

const TYPE_CLASS = {
  add: "changelog-add",
  fix: "changelog-fix",
  opt: "changelog-opt",
  breaking: "changelog-breaking",
  plain: ""
};

let changelogIndex = 0;
let changelogItems = [];

/** 从 JSON 数据构建单个版本的 DOM 节点 */
function buildChangelogItem(entry) {
  const item = document.createElement("div");
  item.className = "changelog-item";

  const version = document.createElement("div");
  version.className = "changelog-version";
  version.textContent = entry.version;

  const date = document.createElement("div");
  date.className = "changelog-date";
  date.textContent = entry.date;

  const list = document.createElement("ul");
  list.className = "changelog-list";

  for (const li of entry.items) {
    const el = document.createElement("li");
    el.textContent = li.text;
    if (li.type && TYPE_CLASS[li.type]) {
      el.className = TYPE_CLASS[li.type];
    }
    list.appendChild(el);
  }

  item.appendChild(version);
  item.appendChild(date);
  item.appendChild(list);
  return item;
}

/** 使用内嵌数据渲染到容器 */
function loadChangelogData() {
  const container = document.getElementById("about-changelog");
  if (!container) return;

  changelogItems = CHANGELOG_DATA.map((entry) => buildChangelogItem(entry));

  if (changelogItems.length > 0) {
    container.innerHTML = "";
    container.appendChild(changelogItems[0].cloneNode(true));
  }
}

function updateChangelogDisplay() {
  const container = document.getElementById("about-changelog");
  const counter = document.getElementById("changelog-counter");
  const prevBtn = document.getElementById("btn-changelog-prev");
  const nextBtn = document.getElementById("btn-changelog-next");

  if (!container || changelogItems.length === 0) return;

  container.innerHTML = "";
  const item = changelogItems[changelogIndex];
  container.appendChild(item.cloneNode(true));

  if (counter) {
    counter.textContent = `${changelogIndex + 1} / ${changelogItems.length}`;
  }
  if (prevBtn) {
    prevBtn.disabled = changelogIndex === 0;
  }
  if (nextBtn) {
    nextBtn.disabled = changelogIndex >= changelogItems.length - 1;
  }
}

function initChangelogModal() {
  const overlay = document.getElementById("modal-changelog-overlay");
  if (!overlay) return;

  const btnOpen = document.getElementById("btn-about-changelog");
  const btnClose = document.getElementById("btn-changelog-close");
  const btnCloseBottom = document.getElementById("btn-changelog-close-bottom");

  function open() {
    changelogIndex = 0;
    updateChangelogDisplay();
    overlay.style.display = "flex";
  }
  function close() { overlay.style.display = "none"; }

  if (btnOpen) btnOpen.addEventListener("click", open);
  if (btnClose) btnClose.addEventListener("click", close);
  if (btnCloseBottom) btnCloseBottom.addEventListener("click", close);

  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) close();
  });

  // 上一个 / 下一个
  const prevBtn = document.getElementById("btn-changelog-prev");
  const nextBtn = document.getElementById("btn-changelog-next");
  if (prevBtn) {
    prevBtn.addEventListener("click", () => {
      if (changelogIndex > 0) {
        changelogIndex--;
        updateChangelogDisplay();
      }
    });
  }
  if (nextBtn) {
    nextBtn.addEventListener("click", () => {
      if (changelogIndex < changelogItems.length - 1) {
        changelogIndex++;
        updateChangelogDisplay();
      }
    });
  }
}

export function initAboutPage() {
  initDisclaimerModal();
  loadChangelogData();
  initChangelogModal();
}