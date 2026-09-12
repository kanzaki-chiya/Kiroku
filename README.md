# Kiroku · 记录

> 简洁、优雅的个人番剧评分与收藏管理桌面应用。本地优先存储，对接 Bangumi 资料库，守护属于你的动画观影记忆。

[English](README_EN.md) · **简体中文**

---

## 📖 产品概述

**Kiroku**（日语「記録」之意）是一款专为动画爱好者打造的个人番剧评分与收藏管理桌面客户端。

传统的在线评分平台往往面临数据被锁死、账户迁移困难以及个人评价受外部算法影响等问题。Kiroku 坚持**本地优先（Local-First）**理念：
- **数据完全自主**：所有个人评分、维度评价、吐槽短评及自定义分档均保存在本地 SQLite 数据库中，不上传至任何中心化个人服务器；
- **权威资料支持**：无缝对接 [Bangumi 番组计划](https://bgm.tv/) 开放接口，便捷检索并拉取官方中文译名、制作阵容、放送信息及全网社区评分；
- **沉浸式交互**：基于 Tauri 2 + Vue 3 构建，兼顾极简排版美学与桌面级极速响应体验。

---

## ✨ 核心功能

### 🔍 Bangumi 资料检索与本地缓存
- **多语言搜索**：支持输入作品中文名、日文原名或 Bangumi 条目 ID 进行实时检索与录入；
- **全方位元数据**：自动同步集数、放送年份、制作公司（Studio）、官方标签及简介；
- **离线封面协议**：通过 Tauri 安全 Asset 协议自动缓存高清封面图片到本地目录，兼顾浏览速度与离线可用性。

### ⭐️ 细粒度五维评分与深度评价
- **十分制总评**：支持 1.0 - 10.0 的个人独立总评分；
- **五维评价雷达**：针对 **故事（Story）**、**角色（Characters）**、**作画（Animation）**、**演出（Direction）**、**音乐（Music）** 进行独立细分打分；
- **长篇评语与状态追踪**：提供「想看（Planned）」、「在看（Watching）」、「看过（Completed）」三态进度管理，记录每一次观影心得。

### 🏷️ 灵活的自定义分档（Tier List）
- **可视化分档系统**：内置或自定义个性化 Tier 梯队（如神作、强烈推荐、值得一看等）；
- **高度自由配置**：支持自定义分档名称、主题色调、档位描述与拖拽排序；
- **快捷筛选联动**：番剧库支持按分档、状态、年份以及更新时间等多维度实时检索与组合排序。

### 📊 评分洞察与社区口味对比
- **均分对比**：直观展示个人均分与 Bangumi 全网社区均分的差值走势；
- **分数分布直方图**：统计 1 至 10 分每个分数区间的条目数量与占比分布；
- **偏离度分析**：自动提炼出个人与大众口味差异最大（最爱/最不符大众预期）的作品，探索独特的个人观影偏好。

### 🔒 数据安全与自由备份
- **标准 JSON 备份**：一键导出包含个人记录、作品快照与分档配置的标准格式备份文件；
- **智能增量导入**：导入时支持预览新增数量、重复条目与冲突项，提供跳过或覆盖策略；
- **数据库快照**：支持一键导出带时间戳的 SQLite 物理数据库快照；
- **隔离与自主管理**：支持独立清空本地封面缓存（保留所有收藏与评分），或一键抹除全部个人数据。

---

## 🛠️ 技术栈

- **前端技术**：Vue 3（Composition API + `<script setup>`）、TypeScript、Vite、Pinia、Vue Router
- **桌面框架**：Tauri 2（Rust）
- **本地存储**：SQLite（rusqlite，启用 WAL 预写式日志模式）
- **网络通信**：Reqwest（桌面端接口与封面拉取，支持 TLS / Gzip）
- **图标与样式**：Lucide Vue Next、原生现代化 CSS 变量与响应式布局

---

## 🚀 快速上手

### 前置要求

在开始之前，请确保您的开发环境已安装以下工具：

1. **Node.js**：`>= 18.0.0`
2. **包管理器**：`npm` 或 `pnpm`
3. **Rust 工具链**（桌面端开发必需）：`rustc` 与 `cargo`（[Rust 官方安装指南](https://www.rust-lang.org/tools/install)）
4. **C++ 编译环境**（Windows）：Visual Studio C++ Build Tools 或带有 C++ 桌面工作负载的 Visual Studio

### 安装步骤

1. 克隆本仓库到本地：
   ```bash
   git clone https://github.com/your-username/kiroku.git
   cd kiroku
   ```

2. 安装前端依赖：
   ```bash
   npm install
   ```

---

## 💻 运行与构建

### 1. 浏览器演示模式（Web Demo）

如需快速预览界面组件与前端交互（此模式下采用内存态与 Mock 数据，不写入本地数据库）：

```bash
npm run dev
```
启动后在浏览器打开终端提示的地址（默认 `http://localhost:5173`）。

### 2. 桌面开发模式（Desktop Dev）

启动带有 Tauri 后端与本地 SQLite 数据库支持的完整桌面应用：

```bash
npm run desktop
```

> **Windows 提示**：若终端未预先加载 MSVC 编译环境，可使用项目内置脚本执行：
> ```cmd
> scripts\with-msvc.cmd npm run desktop
> ```

### 3. 构建发布安装包

打包生成桌面端原生安装程序（Windows 环境下将生成 NSIS 安装包）：

```bash
npm run desktop:build
```
打包产物将位于 `src-tauri/target/release/bundle/` 目录下。

---

## 📜 常用命令清单

| 命令 | 说明 |
| :--- | :--- |
| `npm run dev` | 启动 Vite 开发服务器（浏览器轻量演示） |
| `npm run desktop` | 启动 Tauri 桌面端开发环境（加载 SQLite 与 Bangumi 真实接口） |
| `npm run build` | 执行 TypeScript 类型检查并打包前端静态资源 |
| `npm run desktop:build` | 编译 Rust 后端并生成平台安装包（如 NSIS `.exe`） |
| `npm run typecheck` | 执行 `vue-tsc --noEmit` 进行全量类型检查 |
| `npm run test` | 运行 Vitest 单元测试套件 |
| `npm run preview` | 本地预览打包后的前端静态页面 |

---

## 📂 本地数据存储说明

在桌面端运行时，Kiroku 会在操作系统的标准应用程序数据目录下创建存储空间：

- **Windows**：`%APPDATA%\com.kiroku.app\`
  - `kiroku.db`：SQLite 数据库主文件（包含个人评分、作品元数据及分档配置）
  - `kiroku.db-wal` / `kiroku.db-shm`：SQLite WAL 日志与共享内存文件
  - `covers/`：本地下载的番剧封面图片缓存
  - `logs/kiroku.log`：应用程序运行日志

---

## 📄 开源协议

本项目基于 [MIT License](LICENSE) 开源。
