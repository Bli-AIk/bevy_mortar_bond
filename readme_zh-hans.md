# bevy_mortar_bond

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_mortar_bond.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_mortar_bond.svg"/> <br> <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

**bevy_mortar_bond** — [mortar](https://github.com/Bli-AIk/mortar) 语言的 Bevy “绑钉” （绑定） 插件。

<div><video controls src="https://github.com/user-attachments/assets/9b35cd36-2890-4394-82e0-089d39bcb293" muted="false"></video></div>

<h4 align="center">想亲自试试吗？<br>克隆仓库后运行 `cargo run --example live_terminal`.</h3>

| 英语                     | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

## 简介

将“内容与逻辑严格分离”的设计哲学引入 Bevy 引擎。

`bevy_mortar_bond` 是 [mortar](https://github.com/Bli-AIk/mortar) 脚本语言——一种专为解决叙事文本与事件逻辑耦合而设计的领域特定语言（DSL）——与
Bevy 之间的专属桥梁。

通过将 Mortar 集成到 Bevy 中，本插件提供了一个原生框架，让创作者可以在轻量级的 `.mortar` 文件中编写流畅的分支对话，而无需让技术标记污染叙事内容。它将
Mortar 的逻辑与 Bevy 的 ECS 进行原生桥接，使得驱动游戏事件、播放角色动画以及管理对话 UI 变得简单自然，如同 Rust 代码库的延伸。

## 功能特性

* 📜 **Mortar 脚本集成**: 在你的 Bevy 应用程序中无缝加载和执行 `.mortar` 脚本文件。
* 🧩 **Bevy ECS 兼容性**: 旨在与 Bevy 的实体组件系统 (ECS) 惯用地工作，允许脚本与游戏实体和组件交互。
* 📦 **资源加载**: 为 `.mortar` 文件提供 Bevy 资源加载器，实现脚本资源的轻松管理和热重载。
* 💬 **对话系统基础**: 提供核心实用程序和示例，用于构建动态和分支对话系统。
* 🔗 **可绑定事件索引**：通过 `MortarEventBinding` 将事件索引绑定到任意驱动（打字机效果、音频时间线等 ），示例内置了一个 ECS
  打字机工具，无需额外依赖。

## Bevy 版本支持

| `bevy` | `bevy_mortar_bond` |
|--------|--------------------|
| 0.18   | 0.3.0              |
| 0.17   | < 0.3.0            |

## 使用方法

1. **添加到 Cargo.toml**：

```toml
[dependencies]
bevy_mortar_bond = "0.3.0"
```

2. **基本使用**：

```rust
use bevy::prelude::*;
use bevy_mortar_bond::MortarPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MortarPlugin)
        .run();
}
```

## 示例

### 运行示例

**对话UI示例** - 展示基本的对话框和可点击的选项按钮：

```bash
cargo run --example dialogue_ui
```

### 示例说明

- **dialogue_ui**: 此示例展示了一个功能齐全的对话 UI，具有打字机文本效果、动态 Mortar 事件处理（例如
  动画、颜色变化、声音播放）、变量状态管理和条件文本，所有这些都集成到自定义 Bevy UI 中。
- 点击选项按钮会更新对话文本
- 按钮有鼠标悬停和点击的视觉反馈

## 依赖

本项目使用以下 crate：

| Crate                                                | 版本   | 描述                 |
|------------------------------------------------------|------|--------------------|
| [bevy](https://crates.io/crates/bevy)                | 0.18 | 游戏引擎               |
| [mortar_compiler](https://github.com/Bli-AIk/mortar) | 本地   | Mortar 语言编译器       |
| [serde_json](https://crates.io/crates/serde_json)    | 1.0  | JSON 序列化/反序列化      |
| bevy_mortar_bond_macros                              | 本地   | bevy_mortar_bond 宏 |

### 开发依赖

| Crate                                         | 版本  | 描述          |
|-----------------------------------------------|-----|-------------|
| [proptest](https://crates.io/crates/proptest) | 1.6 | 属性测试 / 模糊测试 |

## 贡献指南

欢迎贡献！
无论你想修复错误、添加功能或改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法并讨论设计或架构。

## 许可证

本项目可依据以下任意一种许可证进行分发：

* Apache License 2.0（[LICENSE-APACHE](LICENSE-APACHE)
  或 [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0)）
* MIT License（[LICENSE-MIT](LICENSE-MIT) 或 [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT)）

可任选其一。