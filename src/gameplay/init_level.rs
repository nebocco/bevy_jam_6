//! Spawn the main level.

use std::{collections::HashMap, fmt::Debug};

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    color::palettes,
    ecs::relationship::RelatedSpawnerCommands,
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    gameplay::{
        GamePhase,
        setup::{CreateObject, SelectedItem},
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>()
        .init_asset::<LevelLayout>()
        .init_asset_loader::<LevelLayoutLoader>();
    app.load_resource::<BgAssets>()
        .load_resource::<ItemAssets>()
        .load_resource::<LevelAssets>()
        .init_resource::<CurrentLevel>()
        .init_resource::<ObjectMap>();
    app.add_systems(
        OnEnter(GamePhase::Setup),
        (despawn_old_level, spawn_level).chain(),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ItemAssets {
    #[dependency]
    pub sprite_sheet: Handle<Image>,
    // #[dependency]
    // pub steps: Vec<Handle<AudioSource>>,
    pub texture_atlas_layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for ItemAssets {
    fn from_world(world: &mut World) -> Self {
        let texture_atlas_layout = {
            let mut texture_atlas = world.resource_mut::<Assets<TextureAtlasLayout>>();
            texture_atlas.add(TextureAtlasLayout::from_grid(
                UVec2::splat(32),
                4,
                8,
                None,
                None,
            ))
        };
        let assets = world.resource::<AssetServer>();
        Self {
            sprite_sheet: assets.load_with_settings(
                "images/item_sprite_sheet.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            // steps: vec![
            //     assets.load("audio/sound_effects/step1.ogg"),
            //     assets.load("audio/sound_effects/step2.ogg"),
            //     assets.load("audio/sound_effects/step3.ogg"),
            //     assets.load("audio/sound_effects/step4.ogg"),
            // ],
            texture_atlas_layout,
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct BgAssets {
    #[dependency]
    pub sprite_sheet: Handle<Image>,
    // #[dependency]
    // pub steps: Vec<Handle<AudioSource>>,
    pub texture_atlas_layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for BgAssets {
    fn from_world(world: &mut World) -> Self {
        let texture_atlas_layout = {
            let mut texture_atlas = world.resource_mut::<Assets<TextureAtlasLayout>>();
            texture_atlas.add(TextureAtlasLayout::from_grid(
                UVec2::splat(64),
                4,
                4,
                None,
                None,
            ))
        };
        let assets = world.resource::<AssetServer>();
        Self {
            sprite_sheet: assets.load_with_settings(
                "images/bg_sprite_sheet.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            // steps: vec![
            //     assets.load("audio/sound_effects/step1.ogg"),
            //     assets.load("audio/sound_effects/step2.ogg"),
            //     assets.load("audio/sound_effects/step3.ogg"),
            //     assets.load("audio/sound_effects/step4.ogg"),
            // ],
            texture_atlas_layout,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CurrentLevel(pub usize);

#[derive(Asset, Debug, Clone, Reflect, Serialize, Deserialize)]
struct LevelLayout {
    board_size: (u8, u8),
    objects: HashMap<GridCoord, Item>,
}

#[derive(Default)]
struct LevelLayoutLoader;

impl AssetLoader for LevelLayoutLoader {
    type Asset = LevelLayout;
    type Settings = ();
    type Error = anyhow::Error;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let custom_asset = ron::de::from_bytes::<LevelLayout>(&bytes)?;
        Ok(custom_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct LevelBase;

#[derive(Resource, Asset, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    levels: Vec<Handle<LevelLayout>>,
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
            levels: vec![
                assets.load("levels/level_01.ron"),
                assets.load("levels/level_02.ron"),
                assets.load("levels/level_03.ron"),
            ],
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Reflect)]
pub struct GridCoord {
    pub x: u8,
    pub y: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Reflect)]
pub enum Item {
    BombSmall,
    BombMedium,
    BombLarge,
    BombHorizontal,
    BombVertical,
    Rock,
    Gem,
    Eraser,
    Fire,
}

impl Item {
    pub fn is_bomb(&self) -> bool {
        matches!(
            self,
            Item::BombSmall
                | Item::BombMedium
                | Item::BombLarge
                | Item::BombHorizontal
                | Item::BombVertical
        )
    }

    pub const fn to_sprite_index(&self) -> usize {
        match self {
            Item::BombSmall => 0,
            Item::BombMedium => 1,
            Item::BombLarge => 2,
            Item::BombHorizontal => 3,
            Item::BombVertical => 4,
            Item::Fire => 5,
            Item::Eraser => 6,
            Item::Rock => 8,
            Item::Gem => 10,
        }
    }
}

impl Item {
    pub fn impact_zone(&self) -> &'static [(i8, i8)] {
        match self {
            // . . . . .
            // . x x x .
            // . x # x .
            // . x x x.
            // . . . . .
            Item::BombSmall => &[
                (-1, 1),
                (0, 1),
                (1, 1),
                (-1, 0),
                (0, 0),
                (1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
            ],

            // . . x . .
            // . x x x .
            // x x # x x
            // . x x x .
            // . . x . .
            Item::BombMedium => &[
                (0, 2),
                (-1, 1),
                (0, 1),
                (1, 1),
                (-2, 0),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (0, -2),
            ],

            // . . . x . . .
            // . . x x x . .
            // . x x x x x .
            // x x x # x x x
            // . x x x x x .
            // . . x x x . .
            // . . . x . . .
            Item::BombLarge => &[
                (0, 3),
                (-1, 2),
                (0, 2),
                (1, 2),
                (-2, 1),
                (-1, 1),
                (0, 1),
                (1, 1),
                (2, 1),
                (-3, 0),
                (-2, 0),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (-2, -1),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (-1, -2),
                (0, -2),
                (1, -2),
                (0, -3),
            ],

            // . . x . .
            // . . x . .
            // . . # . .
            // . . x . .
            // . . x . .
            Item::BombVertical => &[
                (0, 5),
                (0, 4),
                (0, 3),
                (0, 2),
                (0, 1),
                (0, 0),
                (0, -1),
                (0, -2),
                (0, -3),
                (0, -4),
                (0, -5),
            ],

            // . . . . .
            // . . . . .
            // x x # x x
            // . . . . .
            // . . . . .
            Item::BombHorizontal => &[
                (5, 0),
                (4, 0),
                (3, 0),
                (2, 0),
                (1, 0),
                (0, 0),
                (-1, 0),
                (-2, 0),
                (-3, 0),
                (-4, 0),
                (-5, 0),
            ],

            Item::Fire => &[(0, 0)],

            // Eraser does not have an impact zone.
            Item::Rock | Item::Gem | Item::Eraser => &[],
        }
    }
}

impl From<u8> for Item {
    fn from(value: u8) -> Self {
        match value {
            0 => Item::BombSmall,
            1 => Item::BombMedium,
            2 => Item::BombLarge,
            255 => Item::Eraser,
            _ => panic!("Invalid item index"),
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ObjectMap {
    pub objects: std::collections::HashMap<GridCoord, (Item, Entity)>,
    pub fire: Option<(GridCoord, Entity)>,
}

fn despawn_old_level(mut commands: Commands, query: Query<Entity, With<LevelBase>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// A system that spawns the main level.
fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    bg_assets: Res<BgAssets>,
    item_assets: Res<ItemAssets>,
    current_level: Res<CurrentLevel>,
    level_layouts: Res<Assets<LevelLayout>>,
    object_map: ResMut<ObjectMap>,
) {
    let current_level = current_level.0;
    let level_layout_handle = level_assets
        .levels
        .get(current_level)
        .expect("Current level handle not found");
    let level_layout = level_layouts
        .get(level_layout_handle)
        .expect("Level layout not found");

    commands
        .spawn((
            Name::new("Level"),
            LevelBase,
            Transform::default(),
            Visibility::default(),
            StateScoped(Screen::Gameplay),
            children![(
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            )],
        ))
        .with_children(|parent| {
            spawn_grid(parent, bg_assets, item_assets, level_layout, object_map)
        });
}

const CELL_SIZE_BASE: f32 = 32.0;

fn spawn_grid(
    commands: &mut RelatedSpawnerCommands<'_, ChildOf>,
    bg_assets: Res<BgAssets>,
    item_assets: Res<ItemAssets>,
    level_layout: &LevelLayout,
    mut object_map: ResMut<ObjectMap>,
) {
    commands
        .spawn((
            Name::new("Grid"),
            Transform::from_xyz(-200., 0., 0.),
            Visibility::default(),
            StateScoped(Screen::Gameplay),
        ))
        .with_children(move |parent| {
            (0..level_layout.board_size.0).for_each(|x| {
                (0..level_layout.board_size.1).for_each(|y| {
                    spawn_grid_cell(
                        parent,
                        level_layout,
                        x,
                        y,
                        &bg_assets,
                        &item_assets,
                        &mut object_map,
                    );
                });
            });
        });
}

fn spawn_grid_cell(
    builder: &mut RelatedSpawnerCommands<'_, ChildOf>,
    level_layout: &LevelLayout,
    x: u8,
    y: u8,
    bg_assets: &Res<BgAssets>,
    item_assets: &Res<ItemAssets>,
    object_map: &mut ResMut<ObjectMap>,
) {
    let scale_factor = 2.0;
    let cell_size = CELL_SIZE_BASE * scale_factor;
    let x_offset = (level_layout.board_size.0 as f32 - 1.0) * cell_size / 2.0;
    let y_offset = (level_layout.board_size.1 as f32 - 1.0) * cell_size / 2.0;

    let mut entity_builder = builder.spawn((
        Name::new(format!("Tile ({}, {})", x, y)),
        GridCoord { x, y },
        Transform::from_xyz(
            x as f32 * cell_size - x_offset,
            y as f32 * cell_size - y_offset,
            0.0,
        ),
        Pickable::default(),
        Sprite::from_atlas_image(
            bg_assets.sprite_sheet.clone(),
            TextureAtlas {
                layout: bg_assets.texture_atlas_layout.clone(),
                index: 0,
            },
        ),
    ));

    if let Some(&item) = level_layout.objects.get(&GridCoord { x, y }) {
        // if there is an item at the coordinate, disable interactions and spawn the item
        println!("Spawning item {:?} at ({}, {})", item, x, y);

        entity_builder.with_children(|parent| {
            let item_entity = parent
                .spawn((
                    item,
                    ItemState::None,
                    GridCoord { x, y },
                    Sprite::from_atlas_image(
                        item_assets.sprite_sheet.clone(),
                        TextureAtlas {
                            layout: item_assets.texture_atlas_layout.clone(),
                            index: item.to_sprite_index(),
                        },
                    ),
                    Transform::from_scale(Vec3::splat(2.0))
                        .with_translation(Vec3::new(0.0, 0.0, 1.0)),
                    StateScoped(Screen::Gameplay),
                ))
                .id();
            object_map
                .objects
                .insert(GridCoord { x, y }, (item, item_entity));
        });
    } else {
        // if there is no item at the coordinate, interactions are enabled
        entity_builder
            .observe(
                move |over: Trigger<Pointer<Over>>, mut sprite: Query<&mut Sprite>| {
                    let mut sprite = sprite.get_mut(over.target()).unwrap();
                    sprite.color = Color::Srgba(palettes::basic::BLUE);
                },
            )
            .observe(
                move |out: Trigger<Pointer<Out>>, mut sprite: Query<&mut Sprite>| {
                    let mut sprite = sprite.get_mut(out.target()).unwrap();
                    sprite.color = Default::default();
                },
            )
            .observe(
                |out: Trigger<Pointer<Pressed>>,
                 coord: Query<&GridCoord>,
                 selected_item: Res<SelectedItem>,
                 mut commands: Commands| {
                    let entity = out.target();
                    let &coord = coord.get(entity).unwrap();
                    println!("Creating object at coord: {:?}", coord);
                    let item = match out.button {
                        PointerButton::Primary => {
                            let Some(item) = selected_item.0 else {
                                println!("No item selected, skipping object creation.");
                                return;
                            };
                            item
                        }
                        PointerButton::Secondary | _ => {
                            println!("Using eraser, removing object at coord: {:?}", coord);
                            Item::Fire
                        }
                    };
                    println!("Creating object with item: {:?}", item);
                    commands.trigger(CreateObject {
                        parent_grid: entity,
                        coord,
                        item,
                    });
                },
            );
    }
}

// An observer listener that changes the target entity's color.
fn _recolor_on<E: Debug + Clone + Reflect>(
    color: Color,
) -> impl Fn(Trigger<E>, Query<&mut Sprite>) {
    move |ev, mut sprites| {
        let Ok(mut sprite) = sprites.get_mut(ev.target()) else {
            return;
        };
        sprite.color = color;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub enum ItemState {
    #[default]
    None,
    Burned,
}
