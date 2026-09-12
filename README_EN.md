# Kiroku · 记录

> A minimalist, local-first anime rating and collection manager for desktop. Seamlessly integrated with Bangumi, crafted to keep your personal anime memories truly yours.

**English** · [简体中文](README.md)

---

## 📖 Overview

**Kiroku** (derived from the Japanese word「記録」for *record* or *archive*) is a modern, lightweight desktop application designed for anime enthusiasts to catalog, rate, and reflect on what they watch.

Online anime tracking platforms often lock user data into walled gardens, expose user evaluations to external ranking algorithms, or risk service shutdowns. Kiroku adopts a strict **Local-First** philosophy:

- **True Data Sovereignty**: All your ratings, custom reviews, dimensional scores, and tier rankings are stored locally in an embedded SQLite database. No external tracking, no cloud lock-in, no telemetry.
- **Authoritative Metadata**: Direct integration with [Bangumi](https://bgm.tv/) open APIs to search and synchronize official titles, production studios, broadcast schedules, high-resolution covers, and community ratings.
- **Fast & Responsive Experience**: Built with Tauri 2 and Vue 3, combining modern editorial desktop aesthetics with native performance and minimal memory footprint.

---

## ✨ Key Features

### 🔍 Bangumi Search & Local Cover Caching
- **Fast Search**: Search anime subjects by Chinese title, Japanese title, or Bangumi subject ID.
- **Rich Subject Metadata**: Automatically fetch episodes, broadcast year, production studio, summary, tags, and community consensus score.
- **Secure Cover Caching**: High-resolution cover artwork is fetched and cached locally via Tauri's custom asset protocol for instant loading and offline availability.

### ⭐️ 5-Dimension Rating & Personal Reviews
- **10-Point Score Scale**: Granular scoring from 1.0 to 10.0 for your overall verdict.
- **5 Evaluation Dimensions**: Score individual aspects independently across **Story**, **Characters**, **Animation**, **Direction**, and **Music**.
- **Watch Status Tracking**: Organize your library into **Watching**, **Completed**, and **Planned** statuses, accompanied by detailed markdown-ready personal review notes.

### 🏷️ Custom Tier Lists
- **Visual Tier Ranking**: Categorize anime into custom tier rankings (e.g., Masterpiece, Recommended, Average, etc.).
- **Total Customizability**: Customize tier names, badge colors, descriptions, and sort order.
- **Flexible Library Filtering**: Filter and sort your library in real-time by tier, watch status, release year, score, or last updated timestamp.

### 📊 Score Insights & Taste Comparison
- **Community Contrast**: Compare your average rating against Bangumi community consensus in real time.
- **Score Distribution Histogram**: Visualize your rating patterns across the 1–10 score scale.
- **Taste Discrepancy Highlights**: Discover titles where your personal taste diverges most from the community consensus—highlighting your unique hidden gems or personal disappointments.

### 🔒 Privacy, Backups & Portability
- **Standard JSON Backups**: One-click export of all personal records, cached subject metadata, and tier configurations in a portable JSON schema.
- **Safe Import Preview**: Inspect additions, duplicates, and conflicts prior to importing, with options to skip or overwrite existing records.
- **Database Snapshots**: Export raw timestamped SQLite database snapshot files on demand.
- **Decoupled Cache Management**: Clear local cover caches anytime without affecting your personal ratings or review data.

---

## 🛠️ Tech Stack

- **Frontend**: Vue 3 (Composition API with `<script setup>`), TypeScript, Vite, Pinia, Vue Router
- **Desktop Framework**: Tauri 2 (Rust)
- **Local Database**: SQLite (via `rusqlite` with Write-Ahead Logging / WAL mode enabled)
- **Networking**: `reqwest` (desktop API requests and cover downloads with rustls & gzip support)
- **UI & Icons**: Lucide Vue Next, modern CSS design tokens, and fluid responsive layouts

---

## 🚀 Getting Started

### Prerequisites

Ensure you have the following prerequisites installed on your system:

1. **Node.js**: `>= 18.0.0`
2. **Package Manager**: `npm` or `pnpm`
3. **Rust Toolchain** (required for desktop build): `rustc` and `cargo` ([Install Rust](https://www.rust-lang.org/tools/install))
4. **C++ Build Tools** (Windows): Visual Studio C++ Build Tools or Visual Studio with Desktop development with C++ workload

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/kiroku.git
   cd kiroku
   ```

2. Install frontend dependencies:
   ```bash
   npm install
   ```

---

## 💻 Development & Building

### 1. Web Demo Mode

To quickly preview UI components and interactions in the browser (runs in in-memory state with seeded mock data; does not write to the SQLite database):

```bash
npm run dev
```
Open `http://localhost:5173` in your browser.

### 2. Desktop Development Mode

To launch the full desktop application with Tauri and local SQLite persistence:

```bash
npm run desktop
```

> **Windows Note**: If your terminal session does not have the MSVC build tools environment loaded, run with the bundled helper script:
> ```cmd
> scripts\with-msvc.cmd npm run desktop
> ```

### 3. Production Desktop Build

To compile the native installer for your operating system (creates a Windows NSIS `.exe` installer):

```bash
npm run desktop:build
```
Bundled binaries and installers will be generated in `src-tauri/target/release/bundle/`.

---

## 📜 Available Scripts

| Command | Description |
| :--- | :--- |
| `npm run dev` | Start the Vite development server (in-browser mock demo) |
| `npm run desktop` | Launch the Tauri desktop app in development mode with SQLite backend |
| `npm run build` | Perform TypeScript type check and compile production frontend assets |
| `npm run desktop:build` | Build the optimized desktop binary and platform installer (e.g. NSIS) |
| `npm run typecheck` | Run `vue-tsc --noEmit` to validate TypeScript types |
| `npm run test` | Run Vitest unit tests |
| `npm run preview` | Preview the compiled frontend production build locally |

---

## 📂 Local Storage Details

When running as a desktop app, Kiroku stores its persistent data under the standard user application directory:

- **Windows**: `%APPDATA%\com.kiroku.app\`
  - `kiroku.db`: Primary SQLite database file (stores ratings, subjects, and tier definitions)
  - `kiroku.db-wal` / `kiroku.db-shm`: SQLite WAL journal and shared-memory files
  - `covers/`: Downloaded local cover images
  - `logs/kiroku.log`: Application runtime log file

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
