# Kiroku

> 简洁、优雅的个人番剧评分与收藏管理桌面应用。本地优先存储，对接 Bangumi 资料库，守护属于你的动画观影记忆。

[![Tauri 2](https://img.shields.io/badge/Tauri-2.x-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Vue 3](https://img.shields.io/badge/Vue-3.5-4FC08D?logo=vue.js&logoColor=white)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.9-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![SQLite](https://img.shields.io/badge/SQLite-WAL-003B57?logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**简体中文** · [English](README_EN.md)

---

## 📖 产品概述

**Kiroku** 是一款专为动画爱好者打造的个人番剧评分与收藏管理桌面客户端。

在算法推荐与商业平台繁盛的当下，个人的观影记录往往受困于中心化服务的壁垒、隐私窥探或突如其来的平台关停。Kiroku 坚持**本地优先（Local-First）**与**数据主权**理念：

- **数据绝对掌控**：所有个人评分、五维细分、观看进度、长篇短评及自定义分档均保存在本地 SQLite 数据库中。无网络追踪、无中心化服务器强绑定、无需注册账户。
- **权威元数据支持**：直接对接 [Bangumi 番组计划](https://bgm.tv/) 开放接口，便捷检索并拉取官方中文译名、制作阵容、放送日历、系列作品关联、社区均分与高清封面。
- **典雅现代的交互体验**：基于 Tauri 2 与 Vue 3 构建，兼备现代杂志感排版、深浅色模式、方向化转场与封面共享元素动效，拥有原生桌面级的极速冷启与轻量低资源占用。

---

## ✨ 核心特性

### 🔍 Bangumi 资料联动与本地安全缓存
- **精准与模糊检索**：支持输入作品中文名、日文原名或 Bangumi 条目 ID（Subject ID）进行实时检索与快捷录入，展示近期搜索记录。
- **丰富条目档案**：自动同步集数、放送年份、制作公司（Studio）、官方标签及详尽剧情梗概。
- **本地图片流转**：通过 Tauri 自定义 Asset 协议将封面缓存至本地磁盘，兼顾极致浏览速度与离线访问体验；更新资料时自动补齐缺失封面。

### 📅 每日放送与追番日历
- **当周新番放送表**：集成 Bangumi 实时当周每日放送表，按周一至周日清晰展示当前正在连载的新番动画。
- **今日高亮与状态感知**：自动标记当前星期，清晰区分已收录与未收录作品；已收录番剧可直达条目详情，未收录番剧支持一键快捷收录（自动预填「想看」）。

### ⏱️ 追番进度管理与系列作品全景
- **集数实时追踪**：番剧卡片与详情页直观呈现「已看进度 / 总集数」，支持在卡片与详情页一键「+1 话」快速打卡；标记为「看过」时自动补齐进度。
- **系列作品深度关联**：详情页集成 Bangumi 条目关联网络，完整展示原作、前传、续作、剧场版及衍生篇等系列条目，支持库内直达与未收录作品一键收录。
- **单条记录管理**：支持从个人收藏库中单独移除条目，便捷清理误录或不再关心的作品。

### ⭐️ 细粒度评分与直接操纵控件
- **直接操纵式评分**：采用平滑刻度滑杆（0.0 – 10.0 分）、分档快捷选择排（Tier Rail）与分段式状态切换器，告别生硬的表单录入。
- **五维星级解构**：从 **剧情**、**角色**、**演出**、**作画**、**音乐** 五个独立维度进行 0.5 – 5.0 星级评定（半星步进），作为细致鉴赏档位参考，不强行与总分数值绑定。
- **三态流转与长篇心得**：支持「想看（Planned）」、「在看（Watching）」、「看过（Completed）」三种观影状态管理，并支持长达 5000 字的个人长篇随笔与短评。

### 📊 评分洞察与五维口味雷达
- **五维口味雷达图（Taste Profile）**：基于个人在各维度的小分评定，生成直观的多边形雷达图，智能提炼均分最高与最低维度，剖析个人审美偏好。
- **大众均分对照**：直观比对个人评分均值与 Bangumi 全网大众评分的整体偏离走势。
- **分数分布直方图**：清晰展示 0 – 10 分各分值区间的作品分布格局，洞悉个人的打分倾向与宽松度。
- **口味偏离度透视**：自动提炼出个人打分与全网共识差异最大的作品，助你发现属于自己的“沧海遗珠”与“独家雷区”。

### 🏷️ 自由定制分档体系（Tier List）
- **可视化梯队管理**：内置与自定义多级 Tier 梯队（如神作、力荐、值得一看、弃番等），告别扁平列表。
- **高自由度定制**：自由配置分档名称、色彩标识、档位简介，支持拖拽调整梯队权重顺序。
- **即时筛选与联动统计**：侧栏分档快捷入口（含未分档），支持分档与观看状态、年份、评分等多条件即时过滤与组合排序，状态 Tab 实时联动当前分档计数。

### 🎨 现代设计、深色模式与原生动效
- **深浅色主题**：深度适配浅色（Light）、深色（Dark）及跟随系统自动切换，侧栏底部与设置页皆可一键调适。
- **沉浸式无边框窗口**：集成自定义标题栏与原生窗口交互按钮，支持窗口拖拽，并基于 `tauri-plugin-window-state` 自动记忆窗口尺寸与位置。
- **全局流动动效体系**：搭载基于页面层级深度的方向化路由转场、条目封面共享元素形态变换（Shared Element Morph）、列表重排 FLIP 动画与优雅 Toast 反馈。

### 🔒 数据主权、自由备份与自动更新
- **标准 JSON 备份与迁移**：一键导出包含全部个人记录、作品信息快照与分档配置的标准格式备份文件。
- **智能导入与冲突裁决**：导入外部数据时提供即时变更预览（新增数、重复数与冲突项），可自主选择覆盖或跳过。
- **物理快照备份**：支持一键导出带时间戳的 SQLite 物理数据库文件，便于冷备与跨设备归档。
- **独立封面缓存管理**：支持单独清空本地封面图片缓存以释放磁盘空间，绝不影响任何个人记录与评分数据；亦提供彻底重置选项。
- **应用内自动更新**：桌面端集成 `tauri-plugin-updater`，启动时静默检查更新，支持在应用内一键下载并自动升级。

---

## 🛠️ 技术架构

- **前端界面**：Vue 3（Composition API + `<script setup>`）、TypeScript、Vite、Pinia、Vue Router
- **桌面框架**：Tauri 2（Rust）
- **本地存储**：SQLite（基于 `rusqlite`，默认开启 WAL 预写日志模式，读写并发高效稳定）
- **官方插件支持**：
  - `tauri-plugin-updater`：应用内检查更新与静默升级
  - `tauri-plugin-window-state`：跨会话记忆窗口大小、位置与最大化状态
  - `tauri-plugin-process`：应用平滑重启与生命周期处理
- **网络通信**：Reqwest（桌面端接口交互与封面异步拉取，支持 TLS / Gzip）
- **设计与排版**：Lucide Vue Next、现代化原生 CSS 令牌系统（Design Tokens）、FLIP 布局动画与共享元素转场

---

## 🚀 快速上手

### 环境要求

请确保你的开发环境已安装以下基础依赖：

1. **Node.js**：`^20.19.0 || >=22.12.0`
2. **包管理器**：`npm` 或 `pnpm`
3. **Rust 工具链**（构建桌面端必需）：`rustc` 与 `cargo`（参见 [Rust 官方安装文档](https://www.rust-lang.org/tools/install)）
4. **C++ 编译环境**（Windows）：Visual Studio C++ Build Tools 或带有 C++ 桌面开发工作负载的 Visual Studio

### 本地克隆与安装

```bash
git clone https://github.com/kanzaki-chiya/Kiroku.git
cd Kiroku
npm install
```

---

## 💻 运行与构建

### 1. 浏览器轻量演示（Web Demo）

适合快速体验前端界面排版与交互逻辑（使用内存态 Mock 数据，不依赖本地数据库）：

```bash
npm run dev
```
启动后在浏览器访问控制台提示地址（默认 `http://localhost:5173`）。

### 2. 桌面开发模式（Desktop Dev）

启动完整的 Tauri 桌面客户端，连接本地 SQLite 数据库与 Bangumi 真实接口：

```bash
npm run desktop
```

> **Windows 提示**：若终端未预先配置 MSVC 编译环境变量，可使用项目内置的封装脚本运行：
> ```cmd
> scripts\with-msvc.cmd npm run desktop
> ```

### 3. 构建发布安装包

编译生成本地操作系统的原生安装程序（Windows 平台生成定制化 NSIS 安装包）：

```bash
npm run desktop:build
```
构建产物输出于 `src-tauri/target/release/bundle/` 目录中。

---

## 📜 常用命令清单

| 命令 | 说明 |
| :--- | :--- |
| `npm run dev` | 启动 Vite 开发服务器（浏览器 Mock 模式） |
| `npm run desktop` | 启动 Tauri 桌面应用开发模式（连接 SQLite 与 Bangumi 接口） |
| `npm run build` | 检查 TypeScript 类型并编译前端生产静态资源 |
| `npm run desktop:build` | 编译 Rust 后端并生成桌面原生安装包（如 NSIS `.exe`） |
| `npm run typecheck` | 执行 `vue-tsc --noEmit` 进行全量类型检查 |
| `npm run test` | 运行 Vitest 单元测试套件 |
| `npm run preview` | 本地预览编译后的前端静态页面 |

---

## 📂 本地数据存储路径

桌面端运行时，Kiroku 会将数据妥善保存在操作系统标准的应用支持目录下：

- **Windows**：`%APPDATA%\com.kiroku.app\`
  - `kiroku.db`：SQLite 数据库主文件（包含个人记录、条目元数据及自定义分档）
  - `kiroku.db-wal` / `kiroku.db-shm`：SQLite WAL 日志与共享内存文件
  - `covers/`：本地下载并缓存的番剧封面图片
  - `logs/kiroku.log`：应用运行日志

---

## 📄 开源协议

本项目基于 [MIT License](LICENSE) 开源。
