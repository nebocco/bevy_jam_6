use bevy::{
    color::palettes,
    ecs::component::Mutable,
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use std::time::Duration;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    gameplay::{Item, edit::Fire, run::Explode},
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
            update_animation_timer::<ExplodeAnimation>.in_set(AppSystems::TickTimers),
            update_animation_atlas::<ExplodeAnimation>.in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    )
    .add_systems(
        Update,
        (
            update_animation_timer::<FireAnimation>.in_set(AppSystems::TickTimers),
            update_animation_atlas::<FireAnimation>.in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

pub fn explosion(
    trigger: Trigger<Explode>,
    mut commands: Commands,
    asset: Res<ExplosionAssets>,
    fire_query: Query<(Entity, &ChildOf), With<Fire>>,
) {
    if trigger.item.is_bomb() {
        explode_bomb(&mut commands, trigger.parent_entity, &asset, fire_query);
    } else if trigger.item == Item::Rock || trigger.item == Item::Gem {
        let mut entity_builder = commands.entity(trigger.parent_entity);
        explode_object(&mut entity_builder, trigger.item, &asset);
    } else {
        warn!("Unexpected item type for explosion: {:?}", trigger.item);
        return;
    }
}

fn explode_bomb(
    commands: &mut Commands,
    bomb_entity: Entity,
    asset: &ExplosionAssets,
    fire_query: Query<(Entity, &ChildOf), With<Fire>>,
) {
    // remove the fire
    fire_query
        .iter()
        .filter(|(_, child)| child.parent() == bomb_entity)
        .for_each(|(fire_entity, _)| {
            commands.entity(fire_entity).despawn();
        });

    let mut entity_builder = commands.entity(bomb_entity);

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
}

fn explode_object(entity_builder: &mut EntityCommands, item: Item, asset: &ExplosionAssets) {
    // change the sprite to a destroyed item
    entity_builder
        .entry::<Sprite>()
        .and_modify(move |mut sprite| {
            sprite.texture_atlas.iter_mut().for_each(|atlas| {
                atlas.index = match item {
                    Item::Rock => 9, // index for destroyed rock
                    Item::Gem => 11, // index for destroyed gem
                    _ => unreachable!(),
                };
            });
        });

    // create destroyed item animation as a child
    entity_builder.with_children(|parent| {
        parent.spawn((
            Name::new("Destroyed Item Animation"),
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

/// Update the animation timer.
fn update_animation_timer<D>(time: Res<Time>, mut query: Query<&mut D>)
where
    D: SpriteAnimation + Component<Mutability = Mutable>,
{
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas<D>(mut query: Query<(&D, &mut Sprite, &mut Visibility)>)
where
    D: SpriteAnimation + Component<Mutability = Mutable>,
{
    for (animation, mut sprite, mut visibility) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.changed() {
            if animation.finished() {
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

trait SpriteAnimation {
    const FRAMES: usize;
    const INTERVAL: Duration;

    fn update_timer(&mut self, delta: Duration);
    fn changed(&self) -> bool;
    fn finished(&self) -> bool;
    fn get_atlas_index(&self) -> usize;
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ExplodeAnimation {
    timer: Timer,
    frame: usize,
    finished: bool,
}

impl ExplodeAnimation {
    pub fn new() -> Self {
        Self {
            timer: Timer::new(Self::INTERVAL, TimerMode::Once),
            frame: 0,
            finished: false,
        }
    }
}

impl SpriteAnimation for ExplodeAnimation {
    const FRAMES: usize = 9;
    const INTERVAL: Duration = Duration::from_millis(100);

    fn update_timer(&mut self, delta: Duration) {
        if self.finished {
            return;
        }
        self.timer.tick(delta);
        if !self.timer.finished() {
            return;
        }
        self.frame += 1;
        if self.frame >= Self::FRAMES {
            self.finished = true;
        }
    }

    fn changed(&self) -> bool {
        self.timer.finished()
    }

    fn finished(&self) -> bool {
        self.finished
    }

    fn get_atlas_index(&self) -> usize {
        self.frame
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FireAnimation {
    timer: Timer,
    frame: usize,
}

impl FireAnimation {
    pub fn new() -> Self {
        Self {
            timer: Timer::new(Self::INTERVAL, TimerMode::Repeating),
            frame: 0,
        }
    }
}

impl Default for FireAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl SpriteAnimation for FireAnimation {
    const FRAMES: usize = 2;
    const INTERVAL: Duration = Duration::from_millis(400);

    fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.finished() {
            return;
        }
        self.frame = (self.frame + 1) % Self::FRAMES;
    }

    fn changed(&self) -> bool {
        self.timer.just_finished()
    }

    fn finished(&self) -> bool {
        false
    }

    fn get_atlas_index(&self) -> usize {
        self.frame + 5 // offset
    }
}
