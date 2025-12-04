//! Minimal ECS typewriter implementation tailored for the dialogue example.
//! Copied from `bevy_ecs_typewriter` so we can avoid depending on the crate.

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum TypewriterState {
    #[default]
    Idle,
    Playing,
    Paused,
    Finished,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Typewriter {
    pub source_text: String,
    pub current_text: String,
    pub timer: Timer,
    pub state: TypewriterState,
    pub current_char_index: usize,
}

impl Typewriter {
    pub fn new(text: impl Into<String>, char_duration: f32) -> Self {
        Self {
            source_text: text.into(),
            current_text: String::new(),
            timer: Timer::from_seconds(char_duration, TimerMode::Repeating),
            state: TypewriterState::Idle,
            current_char_index: 0,
        }
    }

    pub fn play(&mut self) {
        if self.state == TypewriterState::Idle {
            self.current_char_index = 0;
            self.current_text.clear();
        }
        self.state = TypewriterState::Playing;
        self.timer.reset();
    }

    #[allow(dead_code)]
    pub fn pause(&mut self) {
        if self.state == TypewriterState::Playing {
            self.state = TypewriterState::Paused;
        }
    }

    #[allow(dead_code)]
    pub fn resume(&mut self) {
        if self.state == TypewriterState::Paused {
            self.state = TypewriterState::Playing;
        }
    }

    #[allow(dead_code)]
    pub fn stop(&mut self) {
        self.state = TypewriterState::Idle;
        self.current_char_index = 0;
        self.current_text.clear();
        self.timer.reset();
    }

    #[allow(dead_code)]
    pub fn restart(&mut self) {
        self.stop();
        self.play();
    }

    #[allow(dead_code)]
    pub fn is_finished(&self) -> bool {
        self.state == TypewriterState::Finished
    }

    pub fn is_playing(&self) -> bool {
        self.state == TypewriterState::Playing
    }

    #[allow(dead_code)]
    pub fn progress(&self) -> f32 {
        let total = self.source_text.chars().count();
        if total == 0 {
            return 1.0;
        }
        self.current_char_index as f32 / total as f32
    }
}

fn typewriter_system(time: Res<Time>, mut query: Query<&mut Typewriter>) {
    for mut typewriter in &mut query {
        if typewriter.state != TypewriterState::Playing {
            continue;
        }

        typewriter.timer.tick(time.delta());

        if typewriter.timer.is_finished() {
            let total_chars = typewriter.source_text.chars().count();
            if typewriter.current_char_index >= total_chars {
                typewriter.state = TypewriterState::Finished;
                continue;
            }

            let char_indices: Vec<_> = typewriter.source_text.char_indices().collect();
            let source_len = typewriter.source_text.len();

            if let Some(&(byte_index, _)) = char_indices.get(typewriter.current_char_index) {
                let next_byte_index = char_indices
                    .get(typewriter.current_char_index + 1)
                    .map(|&(i, _)| i)
                    .unwrap_or(source_len);

                let char_str = typewriter.source_text[byte_index..next_byte_index].to_string();
                typewriter.current_text.push_str(&char_str);
                typewriter.current_char_index += 1;
            }
        }
    }
}

pub struct TypewriterPlugin;

impl Plugin for TypewriterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Typewriter>()
            .register_type::<TypewriterState>()
            .add_systems(Update, typewriter_system);
    }
}
