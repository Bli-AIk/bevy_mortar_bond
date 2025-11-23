# bevy_mortar_bond

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development (Initial version in progress)

**bevy_mortar_bond** — Bevy ‘bonding’ (binding) plug-in for mortar language.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`bevy_mortar_bond` is a Bevy plugin that integrates the Mortar scripting language into the Bevy game engine. It provides a robust framework for creating dynamic dialogue systems, interactive events, and complex game logic using Mortar scripts.
It solves the problem of integrating a flexible, external scripting language for content creators and game designers, allowing users to define game flows, character interactions, and dynamic scenarios without recompiling the game engine.

With `bevy_mortar_bond`, you only need to write your game logic and dialogue in `.mortar` script files and integrate them seamlessly into your Bevy application.
In the future, it may also support more advanced scripting features and integrations.
## Features

*   **Mortar Script Integration**: Seamlessly load and execute `.mortar` script files within your Bevy application.
*   **Bevy ECS Compatibility**: Designed to work idiomatically with Bevy's Entity Component System, allowing scripts to interact with game entities and components.
*   **Asset Loading**: Provides Bevy asset loader for `.mortar` files, enabling easy management and hot-reloading of script assets.
*   **Dialogue System Foundation**: Offers core utilities and examples for building dynamic and branching dialogue systems.

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Add to Cargo.toml**:

   ```toml
   [dependencies]
   bevy_mortar_bond = "0.1.0"
   ```

3. **Basic usage**:

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

## Dependencies

This project uses the following crates:

| Crate                                                                                          | Version | Description                        |
|------------------------------------------------------------------------------------------------|---------|------------------------------------|
| [bevy](https://crates.io/crates/bevy)                                                          | 0.17.2  | Game engine                        |
| [mortar_compiler](https://github.com/Bli-AIk/souprune/tree/main/mortar/crates/mortar_compiler) | Path    | Mortar language compiler           |
| [serde_json](https://crates.io/crates/serde_json)                                              | 1.0     | JSON serialization/deserialization |
| bevy_mortar_bond_macros                                                                        | Path    | Macros for bevy_mortar_bond        |
| [bevy_ecs_typewriter](https://github.com/Bli-AIk/bevy_ecs_typewriter)                          | Path    | Bevy ECS typewriter effect         |

## Examples

### Run Examples

1.  **Dialogue UI Example** - Demonstrates basic dialogue boxes and clickable option buttons:
    ```bash
    cargo run --example dialogue_ui
    ```


### Example Descriptions

-   **dialogue_ui**: This example demonstrates a full-featured dialogue UI with typewriter text effects, dynamic Mortar event handling (e.g., animations, color changes, sound playback), variable state management, and conditional text, all integrated into a custom Bevy UI.
    -   Clicking option buttons updates the dialogue text.
    -   Buttons have visual feedback for hover and click states.


## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.


## License

This project is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
  or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
