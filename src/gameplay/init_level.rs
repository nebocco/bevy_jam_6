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
    gameplay::{
        GamePhase, GridCoord, Item,
        edit::{CreateObject, SelectedItem, fire},
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
        .init_resource::<CurrentLevel>();
    app.add_systems(
        OnEnter(GamePhase::Init),
        (despawn_old_level, spawn_level).chain(),
    )
    .add_systems(
        PostUpdate,
        move_to_edit_phase.run_if(in_state(GamePhase::Init)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ItemAssets {
    #[dependency]
    pub sprite_sheet: Handle<Image>,
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
            texture_atlas_layout,
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct BgAssets {
    #[dependency]
    pub sprite_sheet: Handle<Image>,
    #[dependency]
    pub bg_image: Handle<Image>,
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
            bg_image: assets.load_with_settings(
                "images/bg_image.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            texture_atlas_layout,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct CurrentLevel {
    pub level: usize,
    pub layout: Handle<LevelLayout>,
}

#[derive(Asset, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct LevelLayout {
    pub board_size: (u8, u8),
    pub objects: HashMap<GridCoord, Item>,
    pub fire_coord: GridCoord,
    pub meta: LevelMetaData,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct LevelMetaData {
    pub name: Option<String>,
    pub description: Option<String>,
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

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GridTile {
    enable_interactions: bool,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GridTileTint;

#[derive(Resource, Asset, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    pub levels: Vec<Handle<LevelLayout>>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            levels: vec![
                assets.load("levels/level_01.ron"),
                assets.load("levels/level_02.ron"),
                assets.load("levels/level_03.ron"),
            ],
        }
    }
}

fn despawn_old_level(
    mut commands: Commands,
    query: Query<Entity, With<LevelBase>>,
    mut selected_item: ResMut<SelectedItem>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Reset the selected item when the level is despawned
    selected_item.0 = None;
}

/// A system that spawns the main level.
fn spawn_level(
    mut commands: Commands,
    bg_assets: Res<BgAssets>,
    item_assets: Res<ItemAssets>,
    current_level: Res<CurrentLevel>,
    level_layouts: Res<Assets<LevelLayout>>,
) {
    let level_layout = level_layouts
        .get(&current_level.layout)
        .expect("Level layout not found");

    commands
        .spawn((
            Name::new("Level"),
            LevelBase,
            Transform::default(),
            Visibility::default(),
            StateScoped(Screen::Gameplay),
        ))
        .with_children(|parent| spawn_grid(parent, bg_assets, item_assets, level_layout))
        .observe(reset_tint_colors_on_out);
}

const CELL_SIZE_BASE: f32 = 32.0;

fn spawn_grid(
    commands: &mut RelatedSpawnerCommands<'_, ChildOf>,
    bg_assets: Res<BgAssets>,
    item_assets: Res<ItemAssets>,
    level_layout: &LevelLayout,
) {
    commands
        .spawn((
            Name::new("Grid"),
            Transform::from_xyz(0.0, 0.0, 0.5),
            Visibility::default(),
            StateScoped(Screen::Gameplay),
        ))
        .with_children(move |parent| {
            (0..level_layout.board_size.0).for_each(|x| {
                (0..level_layout.board_size.1).for_each(|y| {
                    spawn_grid_cell(parent, level_layout, x, y, &bg_assets, &item_assets);
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
) {
    let scale_factor = 2.0;
    let cell_size = CELL_SIZE_BASE * scale_factor;
    let x_offset = (level_layout.board_size.0 as f32 - 1.0) * cell_size / 2.0;
    let y_offset = (level_layout.board_size.1 as f32 - 1.0) * cell_size / 2.0;

    let grid_coord = GridCoord { x, y };

    let mut entity_builder = builder.spawn((
        Name::new(format!("Tile ({}, {})", x, y)),
        GridTile {
            enable_interactions: true,
        },
        grid_coord,
        Transform::from_xyz(
            x as f32 * cell_size - x_offset,
            y as f32 * cell_size - y_offset,
            0.0,
        ),
        Visibility::default(),
        Pickable::default(),
        Sprite::from_atlas_image(
            bg_assets.sprite_sheet.clone(),
            TextureAtlas {
                layout: bg_assets.texture_atlas_layout.clone(),
                index: 0,
            },
        ),
    ));

    entity_builder.with_child((
        Name::new("Grid Tile Tint"),
        GridTileTint,
        grid_coord,
        Transform::from_xyz(0.0, 0.0, 3.0),
        Sprite::from_color(Color::NONE, Vec2::splat(cell_size)),
        StateScoped(Screen::Gameplay),
    ));

    if let Some(&item) = level_layout.objects.get(&grid_coord) {
        // if there is an item at the coordinate, disable interactions and spawn the item
        entity_builder.with_children(|parent| {
            let mut item_entity = parent.spawn((
                item,
                ItemState::None,
                grid_coord,
                Sprite::from_atlas_image(
                    item_assets.sprite_sheet.clone(),
                    TextureAtlas {
                        layout: item_assets.texture_atlas_layout.clone(),
                        index: item.to_sprite_index(),
                    },
                ),
                Visibility::default(),
                Transform::from_scale(Vec3::splat(2.0)).with_translation(Vec3::new(0.0, 0.0, 1.0)),
                StateScoped(Screen::Gameplay),
            ));

            if level_layout.fire_coord == grid_coord {
                // if the item is a fire, add a special component
                item_entity.with_child(fire(level_layout.fire_coord, item_assets));
            }
        });

        // gray out the tile sprite to indicate that interactions are disabled
        entity_builder
            .entry::<Sprite>()
            .and_modify(|mut sprite| sprite.color = CELL_COLOR_DISABLED);
        entity_builder.insert(GridTile {
            enable_interactions: false,
        });
    } else {
        // if there is no item at the coordinate, interactions are enabled
        entity_builder.observe(recolor_cells).observe(
            |out: Trigger<Pointer<Pressed>>,
             coord: Query<&GridCoord>,
             selected_item: Res<SelectedItem>,
             mut commands: Commands| {
                if out.button != PointerButton::Primary {
                    return;
                }
                let entity = out.target();
                let &coord = coord.get(entity).unwrap();

                let Some(item) = selected_item.0 else {
                    return;
                };
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

fn recolor_cells(
    over: Trigger<Pointer<Over>>,
    selected_item: Res<SelectedItem>,
    target_query: Query<(&GridCoord, &GridTile)>,
    mut tint_query: Query<(&mut Sprite, &GridCoord), With<GridTileTint>>,
) {
    let Ok((target_coord, target_grid_tile)) = target_query.get(over.target()) else {
        return;
    };
    let Some(item) = selected_item.0 else {
        return;
    };

    let affected_coords: Vec<GridCoord> = item
        .impact_zone()
        .iter()
        .map(|(dx, dy)| GridCoord {
            x: target_coord.x.wrapping_add(*dx as u8),
            y: target_coord.y.wrapping_add(*dy as u8),
        })
        .collect();

    tint_query.iter_mut().for_each(|(mut sprite, grid_coord)| {
        let color = if !target_grid_tile.enable_interactions {
            CELL_COLOR_NORMAL
        } else if grid_coord == target_coord {
            CELL_COLOR_HOVERED.with_alpha(0.3)
        } else if affected_coords.contains(grid_coord) {
            CELL_COLOR_AFFECTED.with_alpha(0.3)
        } else {
            CELL_COLOR_NORMAL
        };
        sprite.color = color;
    });
}

fn reset_tint_colors_on_out(
    _out: Trigger<Pointer<Out>>,
    tint_query: Query<&mut Sprite, With<GridTileTint>>,
) {
    reset_tint_colors(tint_query);
}

pub fn reset_tint_colors(mut tint_query: Query<&mut Sprite, With<GridTileTint>>) {
    tint_query.iter_mut().for_each(|mut sprite| {
        sprite.color = CELL_COLOR_NORMAL;
    });
}

const CELL_COLOR_NORMAL: Color = Color::NONE;
const CELL_COLOR_DISABLED: Color = Color::Srgba(palettes::css::LIGHT_GRAY);
const CELL_COLOR_HOVERED: Color = Color::Srgba(palettes::css::LIGHT_BLUE);
const CELL_COLOR_AFFECTED: Color = Color::Srgba(palettes::css::LIGHT_YELLOW);

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub enum ItemState {
    #[default]
    None,
    Burned,
}

fn move_to_edit_phase(mut next_state: ResMut<NextState<GamePhase>>) {
    next_state.set(GamePhase::Edit);
}
