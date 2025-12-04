use crate::MortarGameEvent;
use bevy::prelude::*;

/// Configures how Mortar handles `play_sound` events.
///
/// 配置 Mortar 如何处理 `play_sound` 事件。
#[derive(Resource, Clone, Copy)]
pub struct MortarAudioSettings {
    /// Whether the runtime should automatically spawn [`AudioPlayer`]s for `play_sound`.
    ///
    /// 是否自动为 `play_sound` 事件创建 [`AudioPlayer`]。
    pub auto_play_sound_events: bool,
    /// Playback configuration applied to auto-spawned audio players.
    ///
    /// 自动播放的音频所使用的播放配置。
    pub playback_settings: PlaybackSettings,
}

impl Default for MortarAudioSettings {
    fn default() -> Self {
        Self {
            auto_play_sound_events: true,
            playback_settings: PlaybackSettings::DESPAWN,
        }
    }
}

pub(crate) fn auto_play_sound_events(
    settings: Res<MortarAudioSettings>,
    mut events: MessageReader<MortarGameEvent>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if !settings.auto_play_sound_events {
        return;
    }

    for event in events.read() {
        if event.name != "play_sound" {
            continue;
        }

        if let Some(path) = event.args.first() {
            let audio_handle = asset_server.load::<AudioSource>(path.clone());
            commands.spawn((AudioPlayer::new(audio_handle), settings.playback_settings));
        }
    }
}
