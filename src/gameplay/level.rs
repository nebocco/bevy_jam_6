//! Spawn the main level.

use std::fmt::Debug;

use bevy::{
    color::palettes,
    ecs::relationship::RelatedSpawnerCommands,
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
};

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    gameplay::{GamePhase, setup},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>()
        .load_resource::<LevelAssets>()
        .init_resource::<ObjectMap>()
        .init_resource::<CurrentLevel>();
    app.add_plugins(MeshPickingPlugin)
        .add_event::<CreateObject>();
    app.add_systems(OnEnter(Screen::Gameplay), spawn_level)
        .add_observer(create_object)
        .add_systems(
            Update,
            (reset_all_object_placements, run_simulation).run_if(in_state(Screen::Gameplay)),
        );
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CurrentLevel(pub usize);

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn((
            Name::new("Level"),
            Transform::default(),
            Visibility::default(),
            StateScoped(Screen::Gameplay),
            children![(
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            )],
        ))
        .with_children(|parent| spawn_grid(parent, level_assets, &mut meshes, &mut materials));
}

pub fn spawn_grid(
    commands: &mut RelatedSpawnerCommands<'_, ChildOf>,
    level_assets: Res<LevelAssets>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let rect_handle = meshes.add(Rectangle::new(32.0, 32.0));
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
            (0..10).for_each(|x| {
                (0..10).for_each(|y| {
                    let color_handle = Handle::clone(&color_handle);
                    let hovered_color_handle = Handle::clone(&hovered_color_handle);
                    spawn_grid_cell(
                        parent,
                        x,
                        y,
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
    x: u8,
    y: u8,
    rect_handle: Handle<Mesh>,
    color_handle: Handle<ColorMaterial>,
    hovered_color_handle: Handle<ColorMaterial>,
) {
    builder
        .spawn((
            Name::new(format!("Tile ({}, {})", x, y)),
            GridCoord { x, y },
            Transform::from_xyz(x as f32 * 32.0 - 144.0, y as f32 * 32.0 - 144.0, 0.0),
            Pickable::default(),
            Mesh2d(rect_handle),
            MeshMaterial2d(Handle::clone(&color_handle)),
        ))
        .observe(
            move |over: Trigger<Pointer<Over>>,
                  mut color: Query<&mut MeshMaterial2d<ColorMaterial>>| {
                let mut color = color.get_mut(over.target()).unwrap();
                color.0 = Handle::clone(&hovered_color_handle);
            },
        )
        .observe(
            move |out: Trigger<Pointer<Out>>,
                  mut color: Query<&mut MeshMaterial2d<ColorMaterial>>| {
                let mut color = color.get_mut(out.target()).unwrap();
                color.0 = Handle::clone(&color_handle);
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
                Transform::from_xyz(8.0, 8.0, 3.0),
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
                StateScoped(Screen::Gameplay),
            ))
            .id();

        commands.entity(event.parent_grid).add_child(entity);
        object_map.objects.insert(event.coord, (event.item, entity));
    }
}
