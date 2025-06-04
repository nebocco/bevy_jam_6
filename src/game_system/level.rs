//! Spawn the main level.

use std::fmt::Debug;

use bevy::{color::palettes, ecs::relationship::RelatedSpawnerCommands, prelude::*};

use crate::{asset_tracking::LoadResource, audio::music, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
    app.add_plugins(MeshPickingPlugin);
    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);
}

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
    for x in 0..10 {
        for y in 0..10 {
            let color_handle = Handle::clone(&color_handle);
            let hovered_color_handle = Handle::clone(&hovered_color_handle);
            parent
                        .spawn((
                            Name::new(format!("Tile ({}, {})", x, y)),
                            Transform::from_xyz(x as f32 * 32.0 - 144.0, y as f32 * 32.0 - 144.0, 0.0),
                            Pickable::default(),
                            Mesh2d(Handle::clone(&rect_handle)),
                            MeshMaterial2d(Handle::clone(&color_handle)),
                        ))
                        .observe(
                            move |over: Trigger<Pointer<Over>>,
                             mut color: Query<&mut MeshMaterial2d<ColorMaterial>>,|{
                                println!("Hovered over tile ({}, {})", x, y);
                                let mut color = color.get_mut(over.target()).unwrap();
                                color.0 = Handle::clone(&hovered_color_handle);
                            },
                        ).observe(
                            move |out: Trigger<Pointer<Out>>,
                             mut color: Query<&mut MeshMaterial2d<ColorMaterial>>,|{
                                println!("Pointer left tile ({}, {})", x, y);
                                let mut color = color.get_mut(out.target()).unwrap();
                                color.0 = Handle::clone(&color_handle);
                            },
                        );

        }
    }
    });
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
