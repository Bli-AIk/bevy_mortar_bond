//! This module defines the `MortarAsset` and `MortarAssetLoader`.
//!
//! 本模块定义了 `MortarAsset` 和 `MortarAssetLoader`。

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::TypePath;
use bevy::tasks::ConditionalSendFuture;
use mortar_compiler::{Deserializer, Language, MortaredData, ParseHandler, Serializer};
use std::path::Path;

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
#[derive(Default, bevy::prelude::TypePath)]
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

    /// Compiles a `.mortar` source file into `MortaredData`.
    ///
    /// 将 `.mortar` 源文件编译为 `MortaredData`。
    async fn compile_mortar_source(
        reader: &mut dyn Reader,
        source_path: &Path,
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

        Deserializer::from_json(&json).map_err(Into::into)
    }

    /// Loads a `.mortared` file directly from the asset reader.
    ///
    /// 直接从资源读取器加载 `.mortared` 文件。
    async fn load_mortared_direct(
        reader: &mut dyn Reader,
        path: &Path,
    ) -> Result<MortaredData, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(not(feature = "dev-logs"))]
        let _ = path;
        dev_info!("Loading .mortared file: {:?}", path);

        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let json = std::str::from_utf8(&bytes)?;

        Deserializer::from_json(json).map_err(Into::into)
    }

    /// Logs all public constants contained within a Mortar program.
    ///
    /// 记录 Mortar 程序中的所有公开常量。
    #[cfg_attr(not(feature = "dev-logs"), allow(unused_variables))]
    fn log_public_constants(path: &Path, data: &MortaredData) {
        let public_constants: Vec<_> = data
            .constants
            .iter()
            .filter(|constant| constant.public)
            .collect();
        if public_constants.is_empty() {
            return;
        }

        #[cfg(not(feature = "dev-logs"))]
        let _ = path;

        dev_info!("Public constants exported by {}:", path.display());

        for constant in public_constants {
            #[cfg(not(feature = "dev-logs"))]
            let _ = constant;
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
    #[allow(dead_code)]
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
            // In Bevy 0.18, path() returns &AssetPath
            let asset_path = load_context.path().clone();
            let path = asset_path.path().to_path_buf();

            let data = match path.extension().and_then(std::ffi::OsStr::to_str) {
                Some("mortar") => {
                    // Always compile from source to ensure hot reloading gets the latest changes.
                    //
                    // 始终从源代码编译以确保热重载获取最新更改。
                    Self::compile_mortar_source(reader, &path).await?
                }
                Some("mortared") => Self::load_mortared_direct(reader, &path).await?,
                _ => return Err("Unsupported file extension".into()),
            };

            dev_info!(
                "Successfully loaded mortar asset: {:?} (nodes: {}, functions: {}, variables: {})",
                asset_path,
                data.nodes.len(),
                data.functions.len(),
                data.variables.len()
            );
            Self::log_public_constants(&path, &data);

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
