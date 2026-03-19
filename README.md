# 🎨 *Color Picker (屏幕取色器)*

*基于 Tauri 2.0 和 Vue 3 开发的现代化、轻量级屏幕取色工具。拥有*精致的放大镜 UI，支持自定义快捷键和多种颜色格式导出。

## ✨ 功能特点

- **全局快捷键**：一键随时唤醒/隐藏取色器（默认 `Alt+Shift+C`，支持在设置中自定义录制）。
- **精准取色**：提供跟随鼠标的实时像素放大镜，支持使用**鼠标滚轮**无级缩放查看像素细节。
- **多格式支持**：支持导出 HEX、RGB、HSL 等多种颜色格式（如 `#RRGGBB`, `RRGGBB`, `rgb(R, G, B)`, `hsl(H, S%, L%)`），并支持 HEX 小写切换。
- **自动复制**：左键点击即可完成取色，并自动格式化复制到系统剪贴板，带有友好的浮窗提示。
- **系统托盘**：常驻系统托盘，静默运行不打扰，提供便捷的设置入口。
- **高性能**：后端使用 Rust 结合 Windows GDI 底层 API 进行极速屏幕像素采集，极低延迟。

## 🛠️ 技术栈

- **前端框架**：[Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vitejs.dev/)
- **样式引擎**：[Tailwind CSS v4](https://tailwindcss.com/)
- **桌面端内核**：[Tauri v2](https://v2.tauri.app/) + Rust
- **关键 Rust 依赖**：
  - `rdev`: 处理全局级别鼠标移动、点击和键盘 (Esc) 监听。
  - `tauri-plugin-global-shortcut`: 实现无冲突的全局快捷键动态注册。
  - `tauri-plugin-clipboard-manager`: 处理跨平台剪贴板操作。
  - `Windows GDI`: `GetDIBits` / `BitBlt` 实现高性能屏幕抓取。

## 🚀 开发与构建

### 环境要求

- Node.js (推荐 v18+)
- Rust (最新稳定版)
- Tauri 开发环境依赖 (如 Visual Studio C++ 生成工具等)

### 安装依赖

```bash
npm install
```

### 本地开发 (启动 Dev Server)

```bash
npm run tauri dev
```

### 打包构建 (生成独立可执行文件)

```bash
npm run tauri build
```

构建产物将输出在 `src-tauri/target/release/` 目录以及 `src-tauri/target/release/bundle/` 中。

## ⚙️ 使用说明

1. 启动应用后，应用会静默运行在任务栏系统托盘中。
2. 按下 `Alt+Shift+C` (默认快捷键) 即可唤醒全屏透明取色器。
3. 移动鼠标寻找目标颜色，使用 **鼠标滚轮** 可以调整放大镜的取样范围（缩放级别）。
4. 点击 **鼠标左键** 提取中心点颜色，并根据设置自动复制到剪贴板。
5. 按下 `Esc` 键随时退出取色模式。
6. 右键点击托盘图标 -> 选择“设置”，即可修改颜色输出格式、开关自动复制以及重置全局快捷键。

