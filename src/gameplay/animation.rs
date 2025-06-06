use bevy::{
    color::palettes,
    ecs::system::command,
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
    render::view::visibility,
};
use std::time::Duration;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    gameplay::{init_level::Item, run::Explode},
    theme::palette,
};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    // app.register_type::<AffectAnimation>();
    app.register_type::<ExplosionAssets>()
        .load_resource::<ExplosionAssets>();
    app.register_type::<ExplodeAnimation>();
    app.add_observer(explosion);
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            update_animation_atlas.in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

pub fn explosion(trigger: Trigger<Explode>, mut commands: Commands, asset: Res<ExplosionAssets>) {
    let mut entity_builder = commands.entity(trigger.parent_entity);

    if trigger.item.is_bomb() {
        // Make the exploded bomb red
        entity_builder.entry::<Sprite>().and_modify(|mut sprite| {
            sprite.color = Color::Srgba(palettes::css::RED.with_alpha(0.3));
        });

        // create explosion animation as a child
        entity_builder.with_children(|parent| {
            parent.spawn((
                Name::new("Explosion Animation"),
                Sprite::from_atlas_image(
                    Handle::clone(&asset.explosion_image),
                    TextureAtlas {
                        layout: Handle::clone(&asset.texture_atlas_layout),
                        index: 9, // dummy empty index
                    },
                ),
                Transform::from_xyz(0.0, 0.0, 4.0).with_scale(Vec2::splat(3.0).extend(1.0)),
                ExplodeAnimation::new(),
            ));
        });
    } else if trigger.item == Item::Rock {
        // change the sprite to a destroyed rock
        entity_builder.entry::<Sprite>().and_modify(|mut sprite| {
            sprite.texture_atlas.iter_mut().for_each(|atlas| {
                atlas.index = 9;
            });
        });

        // create destroyed rock animation as a child
        entity_builder.with_children(|parent| {
            parent.spawn((
                Name::new("Destroyed Rock Animation"),
                Sprite::from_atlas_image(
                    Handle::clone(&asset.destroyed_image),
                    TextureAtlas {
                        layout: Handle::clone(&asset.texture_atlas_layout),
                        index: 9, // dummy empty index
                    },
                ),
                Transform::from_xyz(0.0, 0.0, 4.0).with_scale(Vec2::splat(2.0).extend(1.0)),
                ExplodeAnimation::new(),
            ));
        });
    } else if trigger.item == Item::Gem {
        // change the sprite to a destroyed gem
        entity_builder.entry::<Sprite>().and_modify(|mut sprite| {
            sprite.texture_atlas.iter_mut().for_each(|atlas| {
                atlas.index = 11;
            });
        });

        // create destroyed gem animation as a child
        entity_builder.with_children(|parent| {
            parent.spawn((
                Name::new("Destroyed Gem Animation"),
                Sprite::from_atlas_image(
                    Handle::clone(&asset.destroyed_image),
                    TextureAtlas {
                        layout: Handle::clone(&asset.texture_atlas_layout),
                        index: 9, // dummy empty index
                    },
                ),
                Transform::from_xyz(0.0, 0.0, 4.0).with_scale(Vec2::splat(2.0).extend(1.0)),
                ExplodeAnimation::new(),
            ));
        });
    }
}

/// Update the animation timer.
fn update_animation_timer(time: Res<Time>, mut query: Query<&mut ExplodeAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas(mut query: Query<(&ExplodeAnimation, &mut Sprite, &mut Visibility)>) {
    for (animation, mut sprite, mut visibility) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.changed() {
            if animation.finished {
                *visibility = Visibility::Hidden; // Hide the sprite when the animation is finished.
            } else {
                atlas.index = animation.get_atlas_index();
            }
        }
    }
}

/// If the player is moving, play a step sound effect synchronized with the
/// animation.
// fn trigger_explode_sound_effect(
//     mut commands: Commands,
//     player_assets: Res<ItemAssets>,
//     mut step_query: Query<&PlayerAnimation>,
// ) {
//     for animation in &mut step_query {
//         if animation.state == PlayerAnimationState::Walking
//             && animation.changed()
//             && (animation.frame == 2 || animation.frame == 5)
//         {
//             let rng = &mut rand::thread_rng();
//             let random_step = player_assets.steps.choose(rng).unwrap().clone();
//             commands.spawn(sound_effect(random_step));
//         }
//     }
// }

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ExplosionAssets {
    #[dependency]
    explosion_image: Handle<Image>,
    #[dependency]
    destroyed_image: Handle<Image>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    #[dependency]
    sound_effect: Handle<AudioSource>,
}

impl FromWorld for ExplosionAssets {
    fn from_world(world: &mut World) -> Self {
        let texture_atlas_layout = {
            let mut texture_atlas = world.resource_mut::<Assets<TextureAtlasLayout>>();
            texture_atlas.add(TextureAtlasLayout::from_grid(
                UVec2::splat(32),
                6,
                2,
                None,
                None,
            ))
        };

        let assets = world.resource::<AssetServer>();

        Self {
            explosion_image: assets.load_with_settings(
                "images/explosion_01.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            destroyed_image: assets.load_with_settings(
                "images/explosion_26.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            texture_atlas_layout,
            sound_effect: assets.load("audio/sound_effects/step1.ogg"),
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ExplodeAnimation {
    timer: Timer,
    frame: usize,
    finished: bool,
}

impl ExplodeAnimation {
    const FRAMES: usize = 9;
    const INTERVAL: Duration = Duration::from_millis(100);

    pub fn new() -> Self {
        Self {
            timer: Timer::new(Self::INTERVAL, TimerMode::Once),
            frame: 0,
            finished: false,
        }
    }

    pub fn update_timer(&mut self, delta: Duration) {
        if self.finished {
            return;
        }
        self.timer.tick(delta);
        if !self.timer.finished() {
            return;
        }
        self.frame = self.frame + 1;
        if self.frame >= Self::FRAMES {
            self.finished = true;
            return;
        }
    }

    pub fn changed(&self) -> bool {
        self.timer.finished()
    }

    pub fn get_atlas_index(&self) -> usize {
        self.frame
    }
}
