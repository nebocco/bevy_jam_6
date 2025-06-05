//! Spawn the main level.

use std::{collections::HashMap, fmt::Debug};

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    color::palettes,
    ecs::relationship::RelatedSpawnerCommands,
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    demo::level,
    gameplay::{
        GamePhase,
        setup::{self, BgAssets},
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>()
        .init_asset::<LevelLayout>()
        .init_asset_loader::<LevelLayoutLoader>()
        .load_resource::<LevelAssets>()
        .init_resource::<ObjectMap>()
        .init_resource::<CurrentLevel>();
    app.add_plugins(MeshPickingPlugin)
        .add_event::<CreateObject>();
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (despawn_old_level, spawn_level).chain(),
    )
    .add_observer(create_object)
    .add_systems(
        Update,
        (reset_all_object_placements, run_simulation).run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CurrentLevel(pub usize);

#[derive(Asset, Debug, Clone, Reflect, Serialize, Deserialize)]
struct LevelLayout {
    board_size: (u8, u8),
    objects: HashMap<GridCoord, setup::Item>,
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

#[derive(Debug, Clone, Copy, Event)]
struct CreateObject {
    parent_grid: Entity,
    pub coord: GridCoord,
    item: setup::Item,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ObjectMap {
    pub objects: std::collections::HashMap<GridCoord, (setup::Item, Entity)>,
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
    current_level: Res<CurrentLevel>,
    level_layouts: Res<Assets<LevelLayout>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
            spawn_grid(
                parent,
                level_assets,
                bg_assets,
                level_layout,
                &mut meshes,
                &mut materials,
            )
        });
}

const CELL_SIZE_BASE: f32 = 32.0;

fn spawn_grid(
    commands: &mut RelatedSpawnerCommands<'_, ChildOf>,
    level_assets: Res<LevelAssets>,
    bg_assets: Res<BgAssets>,
    level_layout: &LevelLayout,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let rect_handle = meshes.add(Rectangle::new(CELL_SIZE_BASE, CELL_SIZE_BASE));
    let color_handle = materials.add(Color::Srgba(palettes::basic::BLUE));
    let hovered_color_handle = materials.add(Color::Srgba(palettes::basic::RED));

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
                    let color_handle = Handle::clone(&color_handle);
                    let hovered_color_handle = Handle::clone(&hovered_color_handle);
                    spawn_grid_cell(
                        parent,
                        level_layout,
                        x,
                        y,
                        &bg_assets,
                        rect_handle.clone(),
                        color_handle.clone(),
                        hovered_color_handle.clone(),
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
    rect_handle: Handle<Mesh>,
    color_handle: Handle<ColorMaterial>,
    hovered_color_handle: Handle<ColorMaterial>,
) {
    let scale_factor = 2.0;
    let cell_size = CELL_SIZE_BASE * scale_factor;
    let x_offset = (level_layout.board_size.0 as f32 - 1.0) * cell_size / 2.0;
    let y_offset = (level_layout.board_size.1 as f32 - 1.0) * cell_size / 2.0;

    builder
        .spawn((
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
        ))
        .observe(
            move |over: Trigger<Pointer<Over>>, mut sprite: Query<&mut Sprite>| {
                let mut sprite = sprite.get_mut(over.target()).unwrap();
                sprite.color = Color::Srgba(palettes::basic::BLUE);
            },
        )
        .observe(
            move |out: Trigger<Pointer<Out>>, mut sprite: Query<&mut Sprite>| {
                let mut sprite = sprite.get_mut(out.target()).unwrap();
                sprite.color = Color::Srgba(palettes::basic::BLACK);
            },
        )
        .observe(
            |out: Trigger<Pointer<Pressed>>,
             coord: Query<&GridCoord>,
             selected_item: Res<setup::SelectedItem>,
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
                        setup::Item::Fire
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

// An observer listener that changes the target entity's color.
fn recolor_on<E: Debug + Clone + Reflect>(color: Color) -> impl Fn(Trigger<E>, Query<&mut Sprite>) {
    move |ev, mut sprites| {
        let Ok(mut sprite) = sprites.get_mut(ev.target()) else {
            return;
        };
        sprite.color = color;
    }
}

fn reset_all_object_placements(
    button_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut object_map: ResMut<ObjectMap>,
) {
    if button_input.just_pressed(KeyCode::KeyR) {
        for (_key, (_item, entity)) in object_map.objects.drain() {
            commands.entity(entity).despawn();
        }
    }
}

fn run_simulation(
    button_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    if button_input.just_pressed(KeyCode::Space) {
        println!("Running simulation...");
        next_state.set(GamePhase::Run);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub enum ItemState {
    #[default]
    None,
    Burned,
}

// create item on grid click
fn create_object(
    trigger: Trigger<CreateObject>,
    mut commands: Commands,
    item_assets: Res<setup::ItemAssets>,
    mut object_map: ResMut<ObjectMap>,
) {
    let event = trigger.event();
    println!("Creating object at coord: {:?}", event.coord);
    println!("{:?}", &object_map.objects);

    if event.item == setup::Item::Fire {
        if let Some((fire_coord, fire_entity)) = object_map.fire.take() {
            commands.entity(fire_entity).despawn();
            if fire_coord == event.coord {
                println!("Fire already exists at this coordinate, skipping creation.");
                return;
            }
        }
        let entity = commands
            .spawn((
                Name::new("Fire Object"),
                GridCoord::clone(&event.coord),
                setup::Item::Fire,
                Sprite::from_color(palettes::basic::RED, Vec2::splat(8.0)),
                Transform::from_xyz(8.0, 8.0, 3.0).with_scale(Vec3::splat(2.0)),
                StateScoped(Screen::Gameplay),
            ))
            .id();
        commands.entity(event.parent_grid).add_child(entity);
        object_map.fire = Some((event.coord, entity));
        return;
    }

    if let Some((_, existing_entity)) = object_map.objects.remove(&event.coord) {
        commands.entity(existing_entity).despawn();
    }

    if event.item != setup::Item::Eraser {
        let entity = commands
            .spawn((
                Name::new("Item Object"),
                GridCoord::clone(&event.coord),
                setup::Item::clone(&event.item),
                ItemState::None,
                Sprite::from_atlas_image(
                    item_assets.sprite_sheet.clone(),
                    TextureAtlas {
                        layout: item_assets.texture_atlas_layout.clone(),
                        index: event.item as usize,
                    },
                ),
                Transform::from_scale(Vec3::splat(2.0)),
                StateScoped(Screen::Gameplay),
            ))
            .id();

        commands.entity(event.parent_grid).add_child(entity);
        object_map.objects.insert(event.coord, (event.item, entity));
    }
}
