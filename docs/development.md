# 开发指南

从界面使用和游戏内容开始阅读，可以先看[产品与交互](product.md)及[游戏规则](rules.md)。程序的调用关系、状态模型和通信过程见[程序架构](architecture.md)。本文说明如何运行工程，以及修改玩法、界面和资源时涉及哪些位置。

## 运行项目

开发环境需要 Node.js、pnpm、Rust 和平台对应的 Tauri 构建依赖。格式化与检查使用系统中的 Biome。项目的 `packageManager` 指定 pnpm，依赖解析结果保存在 `pnpm-lock.yaml` 和 `Cargo.lock`。

在仓库根目录运行：

```sh
pnpm install
pnpm dev
```

`pnpm dev` 启动 Tauri，Tauri 再启动 Vite 和 Rust 应用。开发页面使用 `http://localhost:1420`，窗口中的界面通过 Tauri IPC 访问本机后端。`pnpm web` 单独启动前端服务，桌面身份、房间和存档功能由 Tauri 进程提供。

Rust 工作区包含 `cataland-core` 与桌面应用 `cataland`，使用 Rust 2024 edition。前端使用 React、TypeScript、Vite 和 PixiJS。实际版本及依赖选项见 [package.json](../package.json)、[Cargo.toml](../Cargo.toml) 与各子包清单。

## 常用命令

| 命令 | 内容 |
| --- | --- |
| `pnpm dev` | 启动桌面开发环境 |
| `pnpm web` | 单独启动 Vite |
| `pnpm bindings` | 从 Rust 类型生成 TypeScript，随后运行项目格式化 |
| `pnpm check` | Biome 自动整理与检查、TypeScript 检查、Rust 格式化和 Clippy |
| `cargo test --workspace` | 执行核心规则与原生网络测试 |
| `pnpm build` | TypeScript 检查和前端生产构建 |
| `pnpm desktop` | 构建本平台桌面应用与安装包 |

`pnpm check` 会修改需要整理格式的文件。TypeScript 使用严格类型检查，包括未使用变量与数组索引检查；Clippy 和 Biome 的警告也会使检查失败。

`pnpm desktop` 通过 Tauri 调用 `pnpm build`，构建时需要的前端资源由同一条流程产生。

## 目录阅读顺序

| 要理解的内容 | 入口 |
| --- | --- |
| 页面如何得到房间 | [src/app.tsx](../src/app.tsx)、[src/session.ts](../src/session.ts) |
| 一次操作如何到达规则 | [src-tauri/src/desktop.rs](../src-tauri/src/desktop.rs)、[src-tauri/src/network.rs](../src-tauri/src/network.rs)、[crates/core/src/room.rs](../crates/core/src/room.rs) |
| 开局与回合如何前进 | [crates/core/src/game.rs](../crates/core/src/game.rs) |
| 前端为什么能展示合法选项 | [crates/core/src/view.rs](../crates/core/src/view.rs)、[src/game.tsx](../src/game.tsx) |
| 棋盘如何画、如何点击 | [src/scene.ts](../src/scene.ts)、[src/camera.ts](../src/camera.ts) |
| 客机如何恢复历史 | [crates/core/src/sync.rs](../crates/core/src/sync.rs)、[src-tauri/src/network.rs](../src-tauri/src/network.rs) |
| 文件保存与历史对局 | [src-tauri/src/storage.rs](../src-tauri/src/storage.rs)、[src/rooms.tsx](../src/rooms.tsx) |

`game.rs` 组织规则状态和流程，各具体玩法分布在经济、发展卡、双人、城市、骑士和进步卡模块中。理解一个动作时，把执行函数与它产生的 `AvailableAction` 或 `Prompt` 一起阅读，可以看到它何时出现、需要什么选择、如何生效。

## 修改玩法

### 费用、位置和动作

建造费用集中在经济、发展卡与城市模块中。合法位置由 `can_settle`、`can_road`、`can_city`、骑士路径和相应模式函数计算。`Game::actions` 使用这些结果生成界面的动作列表。

例如修改城市建造费用，需要沿普通建造、医学卡牌、费用展示和测试决策阅读相关实现。医学有自己的优惠费用，但共用城市位置与棋子规则。规则文本中的普通费用和优惠费用也需要表达相同含义。

位置类动作携带 `Target`。前端把它变为棋盘高亮，命中后发送完整动作。普通操作进入 `Game::apply`，带选择的操作进入效果解析。

### 增加进步卡

进步卡目录位于 `progress.rs`。卡牌枚举和 `info` 同时定义其类型、所属路线、份数、名称与短描述，`decks` 根据这些资料生成三摞牌。

直接改变状态的效果放在 `progress_actions.rs`，例如起重机增加本回合可用折扣。需要选择玩家、牌或位置的效果由 `ProgressChoice` 表达，再由 `progress_choices.rs` 产生提示并解析回应。

前端 `ProgressCards` 按卡牌资料展示名称、数量和说明。地图目标、玩家按钮、手牌数量选择通常复用已有控件。额外的持续操作可以像商业港一样，在回合状态中保存次数，并通过 `AvailableAction` 提供按钮。

新增卡牌同时涉及英文词条和插画名称。插画图集的键与序列化卡牌名一致，制作方式见[美术与声音](assets.md)。

### 多步结算与回合时机

`Effect` 保存需要完成的游戏步骤，`ProgressChoice` 保存进步卡选择的具体上下文。需要跨玩家选择时，执行者也是状态的一部分，例如商业港由出牌者选资源，然后由对方选商品。

选择完成后继续处理队列。当前阶段和已产生的骰子决定返回第一次生产、第二次生产、行动或下一位玩家。增加效果时，需要同时考虑它从哪个时机触发，以及它完成后原回合还剩下什么。

五至六人的 `turn.player` 和 `turn.primary` 分别表达当前行动者与本组生产者。两者分别适合判断卡牌行动和普通玩家交易。双人中立势力位于真实玩家之后，适合棋盘占用与路径逻辑；手牌交换、轮到谁和胜利则按真实玩家处理。

`Game::apply` 是核心规则入口；桌面运行时通过 `Host::apply` 在房间副本上执行，并在存档成功后发布结果。独立使用核心库时，动作的调用和提交由使用者组织。

## 修改共享类型

公开视图、动作、错误与设置的数据源是 Rust 类型。序列化定义与 `TS` 派生一起决定网络消息和前端联合类型。

修改共享类型后执行：

```sh
pnpm bindings
pnpm check
```

生成器位于 `crates/core/src/bin/bindings.rs`，输出 `src/bindings.ts`，从顶层类型遍历所有依赖类型。需要扩展类型时修改 Rust 定义与导出入口，再重新生成文件。

资源数组、三条城市路线、顶点和边的索引都被多处消费。调整这些结构时，规则、视图、生成器、渲染与资源映射需要使用同一顺序。

## 界面与本地化

界面入口由 `App` 选择首页、大厅或游戏页。`useSession` 订阅状态、聊天和通知，并将用户操作发送到 Tauri。游戏页复用资源选择、城市、骑士、交易和进步卡面板。

普通界面文案通过 `useText` 取得翻译函数，词条中文作为键，英文位于 `src/en.json`。例如代码中的 `t("第 {0} 回合", game.turn.number)` 使用相同参数模板显示回合编号。

Rust 中动态文本使用 `text!`，例如：

```rust
crate::text!(
    "{0}获得 {1} 张{2}",
    self.player_name(player),
    count,
    resource.name()
)
```

玩家名通过 `player_name` 取得原文参数，资源名称作为可翻译词条。`Text::List` 用于一组文本，客户端按语言组合列表。

前端异步异常使用 `Failure` 保存模板和参数，再由 `errorText` 转为页面通知。已有错误对象可以随语言切换重新渲染。

主机在消息中传递结构化文本，每个客户端选择自己的语言。新增规则事件时，事件模板与英文词条需要一起编写。

## 棋盘与动效

地形、棋子与可选位置由 `scene.ts` 绘制。地图索引来自 `GameView.board`，视觉布局使用同一份顶点与边。摄像机管理缩放、拖动和视区变换；状态更新通过场景的 `update` 应用。

临时的界面选择保存在 React，例如当前选择的是道路还是村庄工具。游戏结果仍由返回的视图决定，棋盘高亮也由当前动作列表重新取得。

棋盘资源加载基于文档 URL：

```ts
new URL("/art/terrain.json", document.baseURI).href
```

完整地址使图集加载器与关联贴图在开发 HTTP 地址和桌面资源协议中使用相同位置。图集中的 `meta.image` 相对于图集文件解析。

棋子移动事件可以提供 `origin` 和 `target`。场景以棋子图像播放浏览器动画，完成后展示最终棋子。界面上的骰子、资源、分数和船只由 `motion.ts` 处理。音效则在 `sound.ts` 中按事件种类播放。

语言切换、聊天更新和首次加载都可能带来 React 渲染。动画以实际状态差异与新增事件为依据，因此应沿现有的事件游标和场景更新入口扩展。

## 存档数据

完整房间以 JSON 保存到 Tauri 应用数据目录的 `games` 子目录。首页只读取摘要，打开进行中的存档时读取完整 `Room`。结束的存档生成公共浏览视图。

序列化状态包含回合中尚未使用的能力和待完成选择。增加一个这样的字段时，应明确它在空状态中的含义，例如起重机次数为零。具有明确缺省含义的字段可以使用对应的序列化默认值；需要真实游戏信息的字段应由完整状态提供。

`identity.json` 决定本机身份，恢复房间依靠存档中的席位关联。`preferences.json`、`recent.json` 和窗口状态由应用独立保存。开发者阅读特定存档时，需要把玩家身份、事件可见对象和当前效果队列一起理解。

代码中的随机数在创建地图、洗牌、掷骰和偷牌时使用。存档保存已经产生的结果及牌堆顺序；读取相同存档会恢复相同局面，之后产生的随机结果继续由运行时决定。

## 验证代码

项目测试以完整流程和交叉规则为中心。

| 文件 | 验证内容 |
| --- | --- |
| [game.rs](../crates/core/tests/game.rs) | 两种模式、2–6 人从开局运行到获胜，期间检查资源、商品和双人筹码总量 |
| [cities.rs](../crates/core/tests/cities.rs) | 进步卡与野蛮人的连续结算、被打断回合的恢复和供应数量 |
| [knights.rs](../crates/core/tests/knights.rs) | 骑士阻断、驱逐及沿原所属道路网络重新安置 |
| [network/tests.rs](../src-tauri/src/network/tests.rs) | 真实 WebSocket 加入、观战、断线、房主恢复及历史与私有视图同步 |

修改规则时，选项生成、动作执行和完整对局推进是同一条行为链。修改网络时，则需要观察服务端状态、持久文件、每个席位收到的视图及客户端合并后的历史。

静态检查和自动对局各有用途。桌面运行中的资源加载、窗口、声音、动画时序，以及不同电脑上的房间发现，需要在相应运行环境中观察。测试里的自动决策用于推进规则对局。

## 构建与分发

`src-tauri/tauri.conf.json` 定义应用名称、标识、窗口尺寸、前端入口和图标。产品版本从 `package.json` 读取，Rust 包版本由工作区清单管理。

`pnpm desktop` 的产物位于 `target/release/bundle`。macOS 的应用包位于 `macos/Cataland.app`，DMG 位于 `dmg`；Windows 安装包位于 `nsis` 和 `msi`。其他平台的构建目标由 Tauri 的本平台打包器产生。

图标配置引用 `icon.png`、`icon.icns` 和 `icon.ico`。macOS 的 `Info.plist` 还声明本地网络用途和 `_cataland._tcp` 服务发现。窗口全屏、移动和还原的权限定义在 `capabilities/main.json`。

GitHub Actions 在 push、pull request 和手动触发时运行各平台构建。流程生成共享类型，构建安装包，执行静态检查与工作区测试，再上传平台产物。具体运行环境与操作版本由 [.github/workflows/build.yml](../.github/workflows/build.yml) 管理。

## 依赖与资源维护

前端依赖及 pnpm 选项由 `package.json`、`pnpm-workspace.yaml` 和锁文件共同描述。Rust 依赖由工作区、子包清单及 `Cargo.lock` 描述。更新依赖时，用包管理器生成对应清单与锁文件，再执行类型生成、检查和桌面构建。

项目包含一份 [PixiJS 类型补丁](../patches/pixi.js.patch)，由 pnpm 安装时应用。它处理 WebGPU 类型声明，使用 TypeScript 提供的 GPU 类型，并补充画布的 `getContext("webgpu")` 重载。补丁作用于声明文件，运行时绘制逻辑仍来自 PixiJS。

依赖更新涉及这份补丁时，需要阅读新声明与项目编译结果，判断补丁中的类型处理是否仍对应实际需求。补丁文件是仓库中的维护入口。

地形、卡牌和声音的生成脚本与成品一同提交。一般开发可以使用已有成品；调整资源时执行对应生成脚本，并同步图集元数据。详细尺寸、命名和生成命令见[美术与声音](assets.md)。
