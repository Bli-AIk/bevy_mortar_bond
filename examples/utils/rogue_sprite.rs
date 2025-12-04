use bevy::{prelude::*, ui::widget::ImageNode};

/// Path to the rogue sprite sheet asset.
pub const ROGUE_SPRITESHEET_PATH: &str = "sprites/rogue_spritesheet_calciumtrice.png";
const ROGUE_COLUMNS: usize = 10;
const ROGUE_ROWS: usize = 10;
const ROGUE_FRAMES_PER_ROW: usize = 10;
const ROGUE_FRAME_TIME: f32 = 0.12;

/// Plugin that installs shared resources and animation systems for the rogue sprite.
pub struct RogueSpritePlugin;

impl Plugin for RogueSpritePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RogueSpritesheet>().add_systems(
            Update,
            (sync_rogue_sprite_on_change, advance_rogue_sprite_frames),
        );
    }
}

/// Resource that stores handles to the sprite texture and atlas layout.
#[derive(Resource, Clone)]
pub struct RogueSpritesheet {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for RogueSpritesheet {
    fn from_world(world: &mut World) -> Self {
        let texture = {
            let asset_server = world.resource::<AssetServer>();
            asset_server.load(ROGUE_SPRITESHEET_PATH)
        };
        let layout = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .expect("TextureAtlasLayout assets should exist")
            .add(TextureAtlasLayout::from_grid(
                UVec2::new(32, 32),
                ROGUE_COLUMNS as u32,
                ROGUE_ROWS as u32,
                None,
                None,
            ));

        Self { texture, layout }
    }
}

impl RogueSpritesheet {
    /// Build an [`ImageNode`] configured to render the provided sprite state.
    pub fn image_node(&self, sprite: &RogueSprite) -> ImageNode {
        ImageNode::from_atlas_image(
            self.texture.clone(),
            TextureAtlas {
                layout: self.layout.clone(),
                index: sprite.atlas_index(0),
            },
        )
    }
}

/// Gender describes which block of atlas rows should be used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RogueGender {
    Male,
    Female,
}

impl RogueGender {
    const fn row_offset(self) -> usize {
        match self {
            Self::Male => 0,
            Self::Female => 5,
        }
    }
}

/// Supported animation tracks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RogueAnimation {
    Idle,
    Gesture,
    Walk,
    Attack,
    Death,
}

impl RogueAnimation {
    const fn row_delta(self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Gesture => 1,
            Self::Walk => 2,
            Self::Attack => 3,
            Self::Death => 4,
        }
    }
}

/// Component attached to any UI image that is playing a rogue animation.
#[derive(Component, Clone, Copy)]
pub struct RogueSprite {
    pub gender: RogueGender,
    pub animation: RogueAnimation,
}

impl RogueSprite {
    const fn atlas_row(&self) -> usize {
        self.gender.row_offset() + self.animation.row_delta()
    }

    pub const fn new(gender: RogueGender, animation: RogueAnimation) -> Self {
        Self { gender, animation }
    }

    pub fn atlas_index(&self, frame: usize) -> usize {
        self.atlas_row() * ROGUE_COLUMNS + (frame % ROGUE_FRAMES_PER_ROW)
    }
}

/// Tracks playback for a sprite animation.
#[derive(Component)]
pub struct RogueAnimationState {
    timer: Timer,
    frame: usize,
}

impl Default for RogueAnimationState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(ROGUE_FRAME_TIME, TimerMode::Repeating),
            frame: 0,
        }
    }
}

fn sync_rogue_sprite_on_change(
    mut query: Query<
        (&RogueSprite, &mut RogueAnimationState, &mut ImageNode),
        Changed<RogueSprite>,
    >,
) {
    for (sprite, mut state, mut image) in &mut query {
        state.frame = 0;
        state.timer.reset();
        set_image_index(&mut image, sprite.atlas_index(state.frame));
    }
}

fn advance_rogue_sprite_frames(
    time: Res<Time>,
    mut query: Query<(&RogueSprite, &mut RogueAnimationState, &mut ImageNode)>,
) {
    for (sprite, mut state, mut image) in &mut query {
        if state.timer.tick(time.delta()).just_finished() {
            state.frame = (state.frame + 1) % ROGUE_FRAMES_PER_ROW;
            set_image_index(&mut image, sprite.atlas_index(state.frame));
        }
    }
}

fn set_image_index(image: &mut ImageNode, index: usize) {
    if let Some(atlas) = image.texture_atlas.as_mut() {
        atlas.index = index;
    }
}
