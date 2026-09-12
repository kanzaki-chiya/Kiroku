# Kiroku · 记录

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

- **True Data Sovereignty**: All ratings, dimensional evaluations, personalized reviews, and custom tier classifications are stored locally in an embedded SQLite database. No tracking, no mandatory cloud accounts, and no data lock-in.
- **Authoritative Metadata**: Seamlessly integrates with [Bangumi](https://bgm.tv/) open APIs to search, fetch, and synchronize official Chinese and Japanese titles, production studios, broadcast dates, community ratings, and high-resolution covers.
- **Editorial Desktop Aesthetics**: Built with Tauri 2 and Vue 3, pairing modern typography and clean editorial layouts with instantaneous startup, low memory footprint, and native performance.

---

## ✨ Key Features

### 🔍 Bangumi Search & Local Cover Caching
- **Multi-Query Search**: Find anime subjects by Chinese title, Japanese original title, or Bangumi Subject ID with real-time suggestions.
- **Comprehensive Metadata**: Automatically sync episode counts, broadcast year, animation studio, official tags, and full storyline summaries.
- **Local Asset Protocol**: High-resolution cover artwork is fetched and securely cached on disk via Tauri's custom asset protocol, ensuring ultra-fast image loading and full offline capability.

### ⭐️ Granular Scoring & Deep Evaluation
- **10-Point Score Scale**: Granular scoring from 0.0 to 10.0 for your independent overall verdict.
- **5-Star Dimensional Breakdown**: Score individual craft dimensions across **Story**, **Characters**, **Direction**, **Animation**, and **Music** with 0.5–5.0 star ratings (half-star steps), functioning as autonomous quality signals without rigid arithmetic constraints.
- **Watch Status & In-Depth Reviews**: Organize your anime library into **Watching**, **Completed**, and **Planned** statuses, accompanied by rich personal notes of up to 5,000 characters.

### 🏷️ Customizable Tier Lists
- **Visual Tier Hierarchy**: Categorize titles into custom tier levels (e.g., Masterpiece, Highly Recommended, Worth Watching, Dropped).
- **Extensive Customization**: Freely customize tier titles, color palettes, badge descriptions, and drag-and-drop priority order.
- **Multi-Criteria Filtering**: Filter and sort your collection in real time by tier, watch status, release year, score, or last updated timestamp.

### 📊 Score Analytics & Community Taste Comparison
- **Community Contrast**: Compare your average rating against the Bangumi community consensus in real time.
- **Score Distribution Histogram**: Visualize rating patterns and distribution across the 0–10 score range.
- **Taste Discrepancy Highlights**: Automatically surface anime where your evaluation deviates most from the public consensus, celebrating your hidden gems and distinctive personal taste.

### 🔒 Data Sovereignty, Portability & Backups
- **Standard JSON Backups**: One-click export of personal records, subject snapshots, and tier definitions into portable JSON format.
- **Smart Import Preview**: Inspect additions, duplicates, and conflicts prior to importing, with flexible overwrite or skip policies and seamless dimension migration.
- **Physical SQLite Snapshots**: Export raw, timestamped SQLite database files (`.db`) directly for cold storage and easy multi-device archiving.
- **Decoupled Cache Management**: Clear local cover caches anytime to free up disk space without affecting your ratings or notes; complete data reset is also supported.

---

## 🛠️ Tech Stack

- **Frontend**: Vue 3 (Composition API with `<script setup>`), TypeScript, Vite, Pinia, Vue Router
- **Desktop Framework**: Tauri 2 (Rust)
- **Embedded Database**: SQLite (via `rusqlite` with Write-Ahead Logging / WAL mode enabled for robust concurrency)
- **Networking**: Reqwest (native asynchronous requests with rustls & gzip support)
- **UI & Icons**: Lucide Vue Next, modern CSS custom properties (tokens), and fluid responsive layout

---

## 🚀 Getting Started

### Prerequisites

Ensure the following tools are installed in your development environment:

1. **Node.js**: `>= 18.0.0`
2. **Package Manager**: `npm` or `pnpm`
3. **Rust Toolchain** (required for desktop compilation): `rustc` and `cargo` ([Rust Installation Guide](https://www.rust-lang.org/tools/install))
4. **C++ Build Environment** (Windows): Visual Studio C++ Build Tools or Visual Studio with "Desktop development with C++" workload

### Installation

```bash
git clone https://github.com/kanzakichiya/Kiroku.git
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

Compile the native desktop installer for your operating system (generates an NSIS `.exe` installer on Windows):

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
