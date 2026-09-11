# Cataland

Windows、macOS 与 Linux 上的局域网策略桌游。两到六名玩家在群岛上建造、生产与交易，支持基础规则和城市与骑士。

## 开发

```sh
pnpm install
pnpm dev
```

Rust 与系统 WebView 的构建依赖见 [Tauri 开发环境](https://v2.tauri.app/start/prerequisites/)。项目中的 pnpm 配置自动选择所需包管理器。

```sh
pnpm check
pnpm build
pnpm desktop
```

`pnpm bindings` 从 Rust 类型生成前端的数据定义。`uv run scripts/art.py` 生成游戏插画资源。

## 结构

`crates/core` 描述规则与数据；`src-tauri` 管理桌面窗口、局域网房间与存档；`src` 提供界面和棋盘渲染。
