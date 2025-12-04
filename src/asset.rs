//! This module defines the `MortarAsset` and `MortarAssetLoader`.
//!
//! 本模块定义了 `MortarAsset` 和 `MortarAssetLoader`。

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::log::warn;
use bevy::prelude::TypePath;
use bevy::tasks::ConditionalSendFuture;
use mortar_compiler::{Deserializer, Language, MortaredData, ParseHandler, Serializer};
use std::path::{Path, PathBuf};

/// A Bevy asset representing a Mortar dialogue file.
///
/// 代表 Mortar 对话文件的 Bevy 资源。
#[derive(Asset, TypePath, Debug)]
pub struct MortarAsset {
    /// The parsed data from the Mortar file.
    ///
    /// 从 Mortar 文件解析的数据。
    pub data: MortaredData,
}

/// An asset loader for `.mortar` and `.mortared` files.
///
/// 用于 `.mortar` 和 `.mortared` 文件的资源加载器。
#[derive(Default)]
pub struct MortarAssetLoader;

impl MortarAssetLoader {
    /// Detects the system language to provide better diagnostics.
    ///
    /// 检测系统语言以提供更好的诊断信息。
    fn detect_language() -> Language {
        let locale = std::env::var("LANG")
            .or_else(|_| std::env::var("LANGUAGE"))
            .unwrap_or_default()
            .to_lowercase();

        if locale.starts_with("zh") {
            Language::Chinese
        } else {
            Language::English
        }
    }

    /// Finds the base path of the assets directory.
    ///
    /// 查找 `assets` 目录的基本路径。
    fn find_asset_base_path() -> Option<PathBuf> {
        // Try multiple possible locations for the assets directory.
        //
        // 尝试多个 `assets` 目录的可能位置。
        let candidates = [
            PathBuf::from("assets"),
            PathBuf::from("crates/bevy_mortar_bond/assets"),
            PathBuf::from("../bevy_mortar_bond/assets"),
        ];

        for candidate in &candidates {
            if candidate.exists() && candidate.is_dir() {
                return Some(candidate.clone());
            }
        }
        None
    }

    /// Checks if a `.mortar` file needs to be recompiled.
    ///
    /// 检查 `.mortar` 文件是否需要重新编译。
    fn should_recompile(
        source_fs_path: &Path,
        mortared_fs_path: &Path,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let source_meta =
            std::fs::metadata(source_fs_path).map_err(|_| "Cannot access .mortar file metadata")?;

        let Ok(compiled_meta) = std::fs::metadata(mortared_fs_path) else {
            dev_info!("No existing .mortared file found, will compile");
            return Ok(true);
        };

        if let (Ok(source_time), Ok(compiled_time)) =
            (source_meta.modified(), compiled_meta.modified())
        {
            let needs_recompile = source_time > compiled_time;
            dev_info!(
                "Checking recompile: source modified={:?}, compiled modified={:?}, needs_recompile={}",
                source_time,
                compiled_time,
                needs_recompile
            );
            Ok(needs_recompile)
        } else {
            dev_info!("Cannot read modification times, will compile");
            Ok(true)
        }
    }

    /// Compiles a `.mortar` source file into `MortaredData`.
    ///
    /// 将 `.mortar` 源文件编译为 `MortaredData`。
    async fn compile_mortar_source(
        reader: &mut dyn Reader,
        source_path: &Path,
        mortared_path: &Path,
        base_path: &Path,
    ) -> Result<MortaredData, Box<dyn std::error::Error + Send + Sync>> {
        dev_info!("Compiling .mortar file: {:?}", source_path);

        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let source_content = std::str::from_utf8(&bytes)?;

        let language = Self::detect_language();
        let (parse_result, diagnostics) =
            ParseHandler::parse_source_code_with_diagnostics_and_language(
                source_content,
                source_path.to_string_lossy().to_string(),
                false,
                language,
            );

        if diagnostics.has_errors() {
            diagnostics.print_diagnostics(source_content);
            return Err("Mortar compilation failed with errors".into());
        }

        let program = parse_result?;
        let json = Serializer::serialize_to_json(&program, true)?;

        // Write the compiled file.
        //
        // 写入编译后的文件。
        let write_path = base_path.join(mortared_path);
        if let Err(e) = std::fs::write(&write_path, json.as_bytes()) {
            warn!("Failed to write .mortared file to {:?}: {}", write_path, e);
        }

        Deserializer::from_json(&json).map_err(Into::into)
    }

    /// Loads a pre-compiled `.mortared` file.
    ///
    /// 加载预编译的 `.mortared` 文件。
    fn load_mortared_file(
        mortared_fs_path: &Path,
    ) -> Result<MortaredData, Box<dyn std::error::Error + Send + Sync>> {
        dev_info!("Loading existing .mortared file: {:?}", mortared_fs_path);
        let json = std::fs::read_to_string(mortared_fs_path)?;
        Deserializer::from_json(&json).map_err(Into::into)
    }

    /// Loads a `.mortared` file directly from the asset reader.
    ///
    /// 直接从资源读取器加载 `.mortared` 文件。
    async fn load_mortared_direct(
        reader: &mut dyn Reader,
        path: &Path,
    ) -> Result<MortaredData, Box<dyn std::error::Error + Send + Sync>> {
        dev_info!("Loading .mortared file: {:?}", path);

        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let json = std::str::from_utf8(&bytes)?;

        Deserializer::from_json(json).map_err(Into::into)
    }

    /// Logs all public constants contained within a Mortar program.
    ///
    /// 记录 Mortar 程序中的所有公开常量。
    fn log_public_constants(path: &Path, data: &MortaredData) {
        let public_constants: Vec<_> = data
            .constants
            .iter()
            .filter(|constant| constant.public)
            .collect();
        if public_constants.is_empty() {
            return;
        }

        dev_info!("Public constants exported by {}:", path.display());

        for constant in public_constants {
            dev_info!(
                "    {} ({}): {}",
                constant.name,
                constant.const_type,
                Self::format_constant_value(&constant.value)
            );
        }
    }

    /// Formats a constant value for human-friendly logging output.
    ///
    /// 格式化常量值，便于阅读日志。
    fn format_constant_value(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Array(items) => {
                let formatted_items: Vec<String> =
                    items.iter().map(Self::format_constant_value).collect();
                format!("[{}]", formatted_items.join(", "))
            }
            serde_json::Value::Object(map) => {
                let formatted_pairs: Vec<String> = map
                    .iter()
                    .map(|(key, val)| format!("{}: {}", key, Self::format_constant_value(val)))
                    .collect();
                format!("{{{}}}", formatted_pairs.join(", "))
            }
            serde_json::Value::Null => "null".to_string(),
        }
    }
}

impl AssetLoader for MortarAssetLoader {
    type Asset = MortarAsset;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    /// Loads a Mortar asset.
    ///
    /// 加载 Mortar 资源。
    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let path = load_context.path();
            let asset_path = load_context.asset_path();

            let data = match path.extension().and_then(|s| s.to_str()) {
                Some("mortar") => {
                    let mortared_path = path.with_extension("mortared");

                    // Find the actual assets directory.
                    //
                    // 查找实际的 `assets` 目录。
                    let base_path =
                        Self::find_asset_base_path().ok_or("Cannot find assets directory")?;

                    let source_fs_path = base_path.join(path);
                    let mortared_fs_path = base_path.join(&mortared_path);

                    let recompile =
                        Self::should_recompile(&source_fs_path, &mortared_fs_path).unwrap_or(true);

                    if recompile {
                        Self::compile_mortar_source(reader, path, &mortared_path, &base_path)
                            .await?
                    } else {
                        Self::load_mortared_file(&mortared_fs_path)?
                    }
                }
                Some("mortared") => Self::load_mortared_direct(reader, path).await?,
                _ => return Err("Unsupported file extension".into()),
            };

            dev_info!(
                "Successfully loaded mortar asset: {:?} (nodes: {}, functions: {}, variables: {})",
                asset_path,
                data.nodes.len(),
                data.functions.len(),
                data.variables.len()
            );
            Self::log_public_constants(path, &data);

            Ok(MortarAsset { data })
        })
    }

    /// The extensions of the assets this loader supports.
    ///
    /// 此加载器支持的资源扩展名。
    fn extensions(&self) -> &[&str] {
        &["mortar", "mortared"]
    }
}
