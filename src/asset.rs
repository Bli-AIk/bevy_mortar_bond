use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::log::{info, warn};
use bevy::prelude;
use bevy::prelude::TypePath;
use bevy::tasks::ConditionalSendFuture;
use mortar_compiler::{Deserializer, Language, MortaredData, ParseHandler, Serializer};

#[derive(Asset, TypePath, Debug)]
pub struct MortarAsset {
    pub data: MortaredData,
}

#[derive(Default)]
pub struct MortarAssetLoader;

impl AssetLoader for MortarAssetLoader {
    type Asset = MortarAsset;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> impl ConditionalSendFuture<Output = prelude::Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let path = load_context.path();
            let asset_path = load_context.asset_path();

            info!("Loading mortar file: {:?}", path);

            let data = if path.extension().and_then(|s| s.to_str()) == Some("mortar") {
                // 处理 .mortar 文件
                let mortared_path = path.with_extension("mortared");

                // 检查是否需要编译
                let should_compile = if let Ok(source_meta) = std::fs::metadata(path) {
                    if let Ok(compiled_meta) = std::fs::metadata(&mortared_path) {
                        // 如果源文件修改时间晚于编译文件，需要重新编译
                        if let (Ok(source_time), Ok(compiled_time)) =
                            (source_meta.modified(), compiled_meta.modified())
                        {
                            source_time > compiled_time
                        } else {
                            true // 无法获取时间戳，重新编译
                        }
                    } else {
                        true // .mortared 文件不存在，需要编译
                    }
                } else {
                    return Err("Cannot access .mortar file metadata".into());
                };

                if should_compile {
                    info!("Compiling .mortar file: {:?}", path);

                    // 读取源文件内容
                    let mut bytes = Vec::new();
                    reader.read_to_end(&mut bytes).await?;
                    let source_content = std::str::from_utf8(&bytes)?;

                    // 解析源代码
                    let locale = std::env::var("LANG")
                        .or_else(|_| std::env::var("LANGUAGE"))
                        .unwrap_or_default()
                        .to_lowercase();
                    let language = if locale.starts_with("zh") {
                        Language::Chinese
                    } else {
                        Language::English
                    };

                    let (parse_result, diagnostics) =
                        ParseHandler::parse_source_code_with_diagnostics_and_language(
                            source_content,
                            path.to_string_lossy().to_string(),
                            false, // verbose_lexer
                            language,
                        );

                    // 检查诊断错误
                    if diagnostics.has_errors() {
                        diagnostics.print_diagnostics(source_content);
                        return Err("Mortar compilation failed with errors".into());
                    }

                    let program = parse_result?;

                    // 序列化为 JSON
                    let json = Serializer::serialize_to_json(
                        &program, true, // pretty print
                    )?;

                    // 保存到 .mortared 文件
                    if let Err(e) = std::fs::write(&mortared_path, json.as_bytes()) {
                        warn!("Failed to write .mortared file: {}", e);
                    }

                    // 反序列化刚刚生成的 JSON
                    Deserializer::from_json(&json)?
                } else {
                    info!("Loading existing .mortared file: {:?}", mortared_path);

                    // 读取已编译的 .mortared 文件
                    let json = std::fs::read_to_string(&mortared_path)?;
                    Deserializer::from_json(&json)?
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("mortared") {
                // 直接处理 .mortared 文件
                info!("Loading .mortared file: {:?}", path);

                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).await?;
                let json = std::str::from_utf8(&bytes)?;

                Deserializer::from_json(json)?
            } else {
                return Err("Unsupported file extension".into());
            };

            info!(
                "Successfully loaded mortar asset: {:?} (nodes: {}, functions: {}, variables: {})",
                asset_path,
                data.nodes.len(),
                data.functions.len(),
                data.variables.len()
            );

            // TODO: 具体的逻辑处理（例如：节点验证、函数绑定等）

            Ok(MortarAsset { data })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mortar", "mortared"]
    }
}
