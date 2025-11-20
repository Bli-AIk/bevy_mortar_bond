use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::log::{info, warn};
use bevy::prelude::TypePath;
use bevy::tasks::ConditionalSendFuture;
use mortar_compiler::{Deserializer, Language, MortaredData, ParseHandler, Serializer};
use std::path::Path;

#[derive(Asset, TypePath, Debug)]
pub struct MortarAsset {
    pub data: MortaredData,
}

#[derive(Default)]
pub struct MortarAssetLoader;

impl MortarAssetLoader {
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

    fn should_recompile(
        source_path: &Path,
        mortared_path: &Path,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let source_meta =
            std::fs::metadata(source_path).map_err(|_| "Cannot access .mortar file metadata")?;

        let Ok(compiled_meta) = std::fs::metadata(mortared_path) else {
            return Ok(true);
        };

        if let (Ok(source_time), Ok(compiled_time)) =
            (source_meta.modified(), compiled_meta.modified())
        {
            Ok(source_time > compiled_time)
        } else {
            Ok(true)
        }
    }

    async fn compile_mortar_source(
        reader: &mut dyn Reader,
        source_path: &Path,
        mortared_path: &Path,
    ) -> Result<MortaredData, Box<dyn std::error::Error + Send + Sync>> {
        info!("Compiling .mortar file: {:?}", source_path);

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

        // Prepend the "assets" directory to the path for writing
        let write_path = Path::new("assets").join(mortared_path);
        if let Err(e) = std::fs::write(&write_path, json.as_bytes()) {
            warn!(
                "Failed to write .mortared file to {:?}: {}",
                write_path, e
            );
        }

        Deserializer::from_json(&json).map_err(Into::into)
    }

    fn load_mortared_file(
        mortared_path: &Path,
    ) -> Result<MortaredData, Box<dyn std::error::Error + Send + Sync>> {
        // Prepend the "assets" directory to the path for reading
        let read_path = Path::new("assets").join(mortared_path);
        info!("Loading existing .mortared file: {:?}", read_path);
        let json = std::fs::read_to_string(&read_path)?;
        Deserializer::from_json(&json).map_err(Into::into)
    }

    async fn load_mortared_direct(
        reader: &mut dyn Reader,
        path: &Path,
    ) -> Result<MortaredData, Box<dyn std::error::Error + Send + Sync>> {
        info!("Loading .mortared file: {:?}", path);

        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let json = std::str::from_utf8(&bytes)?;

        Deserializer::from_json(json).map_err(Into::into)
    }
}

impl AssetLoader for MortarAssetLoader {
    type Asset = MortarAsset;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

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

                    // We need to construct the full path to check metadata
                    let source_fs_path = Path::new("assets").join(path);
                    let mortared_fs_path = Path::new("assets").join(&mortared_path);

                    let recompile =
                        Self::should_recompile(&source_fs_path, &mortared_fs_path).unwrap_or(true);

                    if recompile {
                        Self::compile_mortar_source(reader, path, &mortared_path).await?
                    } else {
                        Self::load_mortared_file(&mortared_path)?
                    }
                }
                Some("mortared") => Self::load_mortared_direct(reader, path).await?,
                _ => return Err("Unsupported file extension".into()),
            };

            info!(
                "Successfully loaded mortar asset: {:?} (nodes: {}, functions: {}, variables: {})",
                asset_path,
                data.nodes.len(),
                data.functions.len(),
                data.variables.len()
            );

            Ok(MortarAsset { data })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mortar", "mortared"]
    }
}
