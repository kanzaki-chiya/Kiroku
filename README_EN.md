# Kiroku

> A minimalist, local-first desktop application for anime rating and collection management. Seamlessly integrated with Bangumi, crafted to keep your personal anime memories truly yours.

[![Tauri 2](https://img.shields.io/badge/Tauri-2.x-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Vue 3](https://img.shields.io/badge/Vue-3.5-4FC08D?logo=vue.js&logoColor=white)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.9-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![SQLite](https://img.shields.io/badge/SQLite-WAL-003B57?logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[简体中文](README.md) · **English**

---

## 📖 Overview

**Kiroku** is an elegant, lightweight desktop client designed for anime enthusiasts to catalog, rate, and reflect on their viewing journey.

In an era where modern media trackers lock user data behind commercial walled gardens, algorithm-driven rankings, and privacy risks, Kiroku embraces a strict **Local-First** philosophy:

- **True Data Sovereignty**: All personal ratings, dimensional evaluations, watch progress, reviews, and custom tier classifications are stored locally in an embedded SQLite database. No tracking, no mandatory cloud accounts, and no data lock-in.
- **Authoritative Metadata**: Seamlessly integrates with [Bangumi](https://bgm.tv/) open APIs to search, fetch, and synchronize official Chinese and Japanese titles, production studios, broadcast schedules, franchise relations, community ratings, and high-resolution covers.
- **Refined Desktop Aesthetics**: Built with Tauri 2 and Vue 3, pairing modern editorial typography, dark mode, directional page transitions, and shared element cover morphing with instantaneous startup and low resource consumption.

---

## ✨ Key Features

### 🔍 Bangumi Metadata Integration & Local Cover Cache
- **Multi-Query Search**: Find anime subjects by Chinese title, Japanese original title, or Bangumi Subject ID with real-time suggestions and recent query history.
- **Comprehensive Metadata**: Automatically sync episode counts, broadcast year, animation studio, official tags, and full storyline summaries.
- **Local Asset Protocol**: Cover artwork is fetched and cached on disk via Tauri's custom asset protocol, ensuring fast loading and full offline capability; missing artwork is automatically retrieved when refreshing metadata.

### 📅 Broadcast Calendar & Weekly Schedule
- **Weekly Broadcast Lineup**: Integrates Bangumi's live daily broadcast calendar, neatly organizing currently airing series from Monday through Sunday.
- **Current Day Awareness**: Automatically highlights today's broadcast lineup and differentiates collected versus uncollected series. Jump directly to collected titles or add new series in a single click (prefilled with "Planned" status).

### ⏱️ Watch Progress Tracking & Franchise Relations
- **Episode Tracking**: Track viewing progress directly on anime cards and detail pages (e.g., "Watched 8 / 12 eps"). Increment progress with a single "+1 Ep" click, with automatic completion when switching status to "Completed".
- **Franchise Network**: Inspect prequels, sequels, movies, OVAs, and side stories directly on the detail page, with smooth navigation between library entries and one-click additions for new titles.
- **Single-Entry Removal**: Cleanly remove individual entries from your personal library without touching the rest of your collection.

### ⭐️ Granular Scoring & Direct Manipulation Controls
- **Direct Manipulation UI**: Smooth score sliders (0.0 to 10.0 scale), one-click tier selection rails, and segmented watch-status selectors replace cumbersome traditional forms.
- **5-Star Dimensional Breakdown**: Score individual craft dimensions across **Story**, **Characters**, **Direction**, **Animation**, and **Music** with 0.5–5.0 star ratings (half-star steps), functioning as autonomous quality signals without rigid arithmetic constraints.
- **Lifecycle & Long-Form Notes**: Organize titles into **Watching**, **Completed**, and **Planned** statuses, accompanied by rich personal reflections and essays of up to 5,000 characters.

### 📊 Taste Radar & Score Analytics
- **5-Dimensional Taste Radar**: Generate an interactive polygon radar chart based on your craft ratings, surfacing your highest and lowest dimension averages to characterize your aesthetic preferences.
- **Community Contrast**: Compare your average rating against the Bangumi community consensus in real time.
- **Score Distribution Histogram**: Visualize rating patterns across the 0–10 score range to understand your grading tendencies.
- **Taste Discrepancy Highlights**: Automatically surface anime where your evaluation deviates most from the public consensus, celebrating your hidden gems and distinctive personal taste.

### 🏷️ Customizable Tier Lists
- **Visual Tier Hierarchy**: Categorize titles into custom tier levels (e.g., Masterpiece, Highly Recommended, Worth Watching, Dropped).
- **Extensive Customization**: Freely customize tier titles, color palettes, badge descriptions, and drag-and-drop priority order.
- **Quick Sidebar Access & Linked Counters**: Instant sidebar filters (including an Unassigned filter), with status tabs dynamically reflecting item counts within the selected tier.

### 🎨 Fluid Motion, Dark Mode & Frameless Window
- **Theme Modes**: Comprehensive support for Light, Dark, and System-following themes with instant switching from the sidebar or settings view.
- **Frameless Window Experience**: Sleek custom title bar with native window controls (minimize, maximize, close) and draggable title region, retaining window dimensions and position via `tauri-plugin-window-state`.
- **Fluid Motion System**: Depth-aware directional routing transitions, shared element cover morphing between library cards and detail views, smooth FLIP reordering for lists, and graceful toast dismissals.

### 🔒 Data Sovereignty, Portability & In-App Updates
- **Standard JSON Backups**: One-click export of personal records, subject snapshots, and tier definitions into portable JSON format with smart import preview and conflict resolution.
- **Physical SQLite Snapshots**: Export raw, timestamped SQLite database files (`.db`) directly for cold storage and archival.
- **Decoupled Cache Management**: Clear local cover caches anytime to free up disk space without affecting your ratings or notes; complete data reset is also supported.
- **In-App Auto Updates**: Integrated `tauri-plugin-updater` checks GitHub Releases on launch, allowing one-click download and seamless upgrade.
- **Branded Windows Installer**: Customized NSIS installer visuals for a polished setup experience.

---

## 🛠️ Tech Stack

- **Frontend**: Vue 3 (Composition API with `<script setup>`), TypeScript, Vite, Pinia, Vue Router
- **Desktop Framework**: Tauri 2 (Rust)
- **Embedded Database**: SQLite (via `rusqlite` with Write-Ahead Logging / WAL mode enabled)
- **Official Plugins**:
  - `tauri-plugin-updater`: In-app update checks and seamless installation
  - `tauri-plugin-window-state`: Window size, position, and maximization persistence
  - `tauri-plugin-process`: Graceful app reboot and process handling
- **Networking**: Reqwest (native asynchronous requests with rustls & gzip support)
- **UI & Motion**: Lucide Vue Next, modern CSS design tokens, FLIP layout animations, and shared element transitions

---

## 🚀 Getting Started

### Prerequisites

Ensure the following tools are installed in your development environment:

1. **Node.js**: `^20.19.0 || >=22.12.0`
2. **Package Manager**: `npm` or `pnpm`
3. **Rust Toolchain** (required for desktop compilation): `rustc` and `cargo` ([Rust Installation Guide](https://www.rust-lang.org/tools/install))
4. **C++ Build Environment** (Windows): Visual Studio C++ Build Tools or Visual Studio with "Desktop development with C++" workload

### Installation

```bash
git clone https://github.com/kanzaki-chiya/Kiroku.git
cd Kiroku
npm install
```

---

## 💻 Development & Building

### 1. Web Demo Mode

Quickly preview UI components and layout in your browser (runs in in-memory state with seeded mock data; does not write to the SQLite database):

```bash
npm run dev
```
Open `http://localhost:5173` (or the URL shown in your terminal) in your browser.

### 2. Desktop Development Mode

Launch the full desktop application backed by Tauri and the local SQLite database:

```bash
npm run desktop
```

> **Windows Note**: If your terminal session does not have the MSVC build environment loaded, use the bundled wrapper script:
> ```cmd
> scripts\with-msvc.cmd npm run desktop
> ```

### 3. Production Desktop Build

Compile the native desktop installer for your operating system (generates a customized NSIS `.exe` installer on Windows):

```bash
npm run desktop:build
```
The output will be generated under `src-tauri/target/release/bundle/`.

---

## 📜 Available Scripts

| Command | Description |
| :--- | :--- |
| `npm run dev` | Start the Vite development server (in-browser mock demo) |
| `npm run desktop` | Launch the Tauri desktop app with SQLite backend and Bangumi integration |
| `npm run build` | Validate TypeScript types and compile frontend production assets |
| `npm run desktop:build` | Build the optimized Rust binary and platform installer (e.g., NSIS `.exe`) |
| `npm run typecheck` | Run `vue-tsc --noEmit` for full static type validation |
| `npm run test` | Run the Vitest unit test suite |
| `npm run preview` | Locally preview the compiled frontend production build |

---

## 📂 Local Storage Paths

When running as a desktop app, Kiroku stores persistent data under the standard user application directory:

- **Windows**: `%APPDATA%\com.kiroku.app\`
  - `kiroku.db`: Primary SQLite database file (stores ratings, subjects, and tier definitions)
  - `kiroku.db-wal` / `kiroku.db-shm`: SQLite WAL journal and shared-memory files
  - `covers/`: Downloaded local anime cover image cache
  - `logs/kiroku.log`: Application runtime log file

---

## 📄 License

This project is open source under the [MIT License](LICENSE).
