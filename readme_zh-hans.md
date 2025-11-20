# bevy_mortar_bond

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br> <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中（初始版本正在开发）

**bevy_mortar_bond** — mortar 语言的 Bevy “绑钉” （绑定） 插件。

| 英语                     | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

## 介绍

`bevy_mortar_bond` 是一个<待补充>。
它解决了<待补充>，让用户能够<待补充>。

使用 `bevy_mortar_bond`，你只需要<待补充>。
未来还计划支持<待补充>。

## 功能

* <待补充>
* <待补充>
* <待补充>
* （计划中）<待补充>

## 使用方法

1. **安装 Rust**（如果尚未安装）：

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **添加到 Cargo.toml**：

   ```toml
   [dependencies]
   bevy_mortar_bond = "0.1.0"
   ```

3. **基本使用**：

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

1. **对话UI示例** - 展示基本的对话框和可点击的选项按钮：
   ```bash
   cargo run --example dialogue_ui
   ```

2. **动态对话示例** - 展示动态增减选项数量的对话系统：
   ```bash
   cargo run --example dynamic_dialogue
   ```

### 示例说明

- **dialogue_ui**: 一个简单的对话显示区域和3个固定的选项按钮
  - 点击选项按钮会更新对话文本
  - 按钮有鼠标悬停和点击的视觉反馈
  
- **dynamic_dialogue**: 支持动态修改选项数量
  - 点击"增加选项"可添加新选项（最多10个）
  - 点击"减少选项"可移除最后一个选项（最少保留1个）
  - 所有选项都可以点击并触发对话更新

## 依赖

本项目使用以下 crate：

| Crate                                             | 版本    | 描述   |
| ------------------------------------------------- | ----- | ---- |
| [bevy](https://crates.io/crates/bevy) | 0.17.2 | 游戏引擎 |

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
