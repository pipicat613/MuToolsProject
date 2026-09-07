# MuTools

<h1 align="center">
  <img src="./MuToolsCode/src/assets/images/icon.png" alt="MuTools" width="128" />
</h1>

<p align="center">
  <strong>一站式 MuMu12 模拟器安装与优化工具</strong>
</p>


## 功能特性

- **MuMu 安装** – 支持在线与本地安装模拟器
- **MuMu 优化** – 支持批量处理
  - 去除开屏广告、消息中心广告、模拟器内桌面广告
  - 禁止模拟器更新
  - 仿专版 / 海外版修改
  - 禁用 MuMu 遥测
- **广泛支持** - 支持 MuMu12 V4/V5/V6 版本
- **MuMu 信息** – 检测并展示模拟器信息
- **帮助中心** – 支持内嵌打开官方文档与客服页面

## 资源包下载

- 前往[MuToolsResource](https://github.com/pipicat613/MuToolsResource)查看

## 技术栈

| 模块 | 技术 |
| --- | --- |
| 前端 | Vanilla JS + Vite |
| 桌面框架 | Tauri v2 |
| 后端 | Rust |

## 项目结构

```
MuToolsProject/
├── MuToolsCode/                 # 主程序
│   ├── package.json
│   ├── vite.config.js
│   │
│   ├── src/                     # 前端源码
│   │   ├── index.html           # 应用入口
│   │   ├── main.js              # 入口逻辑
│   │   ├── styles.css           # 全局样式
│   │   ├── assets/images/       # 图标等静态资源
│   │   └── js/                  
│   │       ├── router.js        # 页面路由
│   │       ├── install.js       # 安装
│   │       ├── packages.js      # 资源包管理
│   │       ├── optimize.js      # 优化项配置
│   │       ├── mumu-info.js     # 模拟器信息
│   │       ├── settings.js      # 设置
│   │       ├── logger.js        # 前端日志
│   │       ├── toast.js         # Toast 提示
│   │       ├── daily-quotes.js  # 每日一言
│   │       ├── help.js          # 帮助中心
│   │       └── about.js         # 关于页面
│   │
│   └── src-tauri/               # Rust 后端
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       └── src/
│           ├── main.rs          # 入口
│           ├── lib.rs           # 命令注册
│           ├── downloader.rs    # 下载器
│           ├── packages.rs      # 资源包管理
│           ├── optimize.rs      # 优化
│           ├── mumu_info.rs     # 信息获取
│           ├── config.rs        # 设置
│           ├── elevation.rs     # 提权
│           └── logger.rs        # 后端日志
│
├── Tools_desc_generator/        # 资源包描述生成器
├── Tools_portable_builder/      # 便携版构建器
└── build.bat                    # 构建脚本
```

## 特别鸣谢

晚安 | Razgriz | piedge

## 免责声明

- MuTools 是**非官方的第三方工具**，与 MuMu 官方无关。
- 使用前请先阅读应用内的**免责声明**，继续使用即视为同意。

## 开源协议

- 本项目代码基于 GPLv3 协议开源。
- 本项目引用的 7-Zip、aria2 等相关第三方组件遵循其相关协议。
