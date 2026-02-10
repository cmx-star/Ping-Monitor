# Ping Monitor

A beautiful, high-performance ping monitoring utility built with **Tauri**, **Vue 3**, and **Rust**.

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)
![Vue](https://img.shields.io/badge/Vue-3.0-green)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)

## 🛠 Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** (v18 or newer)
- **pnpm** (Recommended package manager)
- **Rust** (Latest stable release)
  - Install via: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Tauri Dependencies**
  - macOS: `xcode-select --install`

## 🚀 Getting Started

Follow these steps to get the project running locally.

### 1. Clone the Repository

```bash
git clone https://github.com/cmx-star/Ping-Monitor.git
cd Ping-Monitor
```

### 2. Install Dependencies

Install frontend dependencies using pnpm:

```bash
pnpm install
```

(Rust dependencies will be automatically handled by Cargo when you run the app).

### 3. Run in Development Mode

Start the Tauri development window with hot-reload:

```bash
pnpm tauri dev
```

### 4. Build for Production

To create an optimized release bundle (DMG on macOS):

```bash
pnpm tauri build
```

## 🧩 Project Structure

- `src/` - Vue 3 Frontend (UI)
- `src-tauri/` - Rust Backend (System Tray, Pinger, Event Loop)
