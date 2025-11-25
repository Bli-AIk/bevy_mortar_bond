# bevy_mortar_bond

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_mortar_bond.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_mortar_bond.svg"/> <br> <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

**bevy_mortar_bond** — A Bevy "bond" (binding) plugin for the [mortar](https://github.com/Bli-AIk/mortar) language.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`bevy_mortar_bond` is a Bevy plugin that integrates the Mortar scripting language into the Bevy game engine. It provides a powerful framework for creating dynamic dialogue systems, interactive events, and complex game logic using Mortar scripts.

It addresses the problem of integrating a flexible external scripting language for content creators and game designers, allowing users to conveniently define game flow, character interactions, and dynamic scenes.

You can simply write your game logic and dialogue in `.mortar` script files and seamlessly integrate them into your Bevy application.

In the future, it may also support more advanced scripting features and integrations.

## Features

*   **Mortar Script Integration**: Seamlessly load and execute `.mortar` script files within your Bevy application.
*   **Bevy ECS Compatibility**: Designed to work idiomatically with Bevy's Entity Component System (ECS), allowing scripts to interact with game entities and components.
*   **Resource Loading**: Provides a Bevy resource loader for `.mortar` files, enabling easy management and hot-reloading of script resources.
*   **Dialogue System Foundation**: Offers core utilities and examples for building dynamic and branching dialogue systems.
*   **Bindable Event Indexes**: The `MortarEventBinding` component lets you drive events from any index source (typewriter effects, audio cues, etc.), and the examples demonstrate a `bevy_ecs_typewriter` integration.

## Usage

1.  **Add to Cargo.toml**:

    ```toml
    [dependencies]
    bevy_mortar_bond = "0.1.0"
    ```

2.  **Basic Usage**:

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

## Examples

### Running Examples

**Dialogue UI Example** - Demonstrates basic dialogue boxes and clickable option buttons:

```bash
cargo run --example dialogue_ui
```

### Example Description

*   **dialogue_ui**: This example showcases a fully functional dialogue UI with typewriter text effects, dynamic Mortar event handling (e.g., animations, color changes, sound playback), variable state management, and conditional text, all integrated into a custom Bevy UI.
*   Clicking option buttons updates the dialogue text.
*   Buttons have visual feedback for hover and click states.

## Dependencies

This project uses the following crates:

| Crate                                                                 | Version | Description                                |
|-----------------------------------------------------------------------|---------|--------------------------------------------|
| [bevy](https://crates.io/crates/bevy)                                 | 0.17.2  | Game Engine                                |
| [mortar_compiler](https://github.com/Bli-AIk/mortar)                  | Local   | Mortar Language Compiler                   |
| [serde_json](https://crates.io/crates/serde_json)                     | 1.0     | JSON Serialization/Deserialization         |
| bevy_mortar_bond_macros                                               | Local   | bevy_mortar_bond Macros                    |
| [bevy_ecs_typewriter](https://github.com/Bli-AIk/bevy_ecs_typewriter) | Local   | Bevy ECS Typewriter Effect (examples only) |

## Contribution Guide

Contributions are welcome!
Whether you want to fix bugs, add features, or improve documentation:

*   Submit an **Issue** or **Pull Request**.
*   Share ideas and discuss design or architecture.

## License

This project can be distributed under the terms of either of the following licenses:

*   Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
    or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
*   MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

You may choose either one.
