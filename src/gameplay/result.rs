use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::{
    audio::{SEVolume, SoundEffectAssets, sound_effect, stop_music},
    gameplay::{
        CurrentLevel, GamePhase, GridCoord, Item, ItemState, LevelAssets, LevelLayout,
        init_level::{MissionsSection, missions_section},
    },
    screens::Screen,
    theme::{UiAssets, widget},
};
use bevy::prelude::*;
use bevy_persistent::prelude::*;
use serde::{Deserialize, Serialize};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameResult>();

    app.add_systems(Startup, insert_persistent_resources);
    app.add_systems(
        OnEnter(GamePhase::Result),
        (
            (
                compute_game_result,
                record_cleared_levels,
                (init_result_state, update_mission_status),
            )
                .chain(),
            stop_music,
        ),
    );
}

#[derive(Resource, Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ClearedLevels(pub HashMap<usize, GameResult>);

#[derive(Resource, Reflect, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct GameResult {
    pub level: usize,
    pub is_cleared: bool,
    pub used_bomb_count: u8,
    pub affected_cell_count: u8,
    pub mission_status: [bool; 3], // clear, min_bombs, min_affected_cells
}

fn compute_game_result(
    current_level: Res<CurrentLevel>,
    level_assets: Res<Assets<LevelLayout>>,
    query: Query<(&Item, &ItemState, &GridCoord)>,
    mut result: ResMut<GameResult>,
) {
    // reset to default values
    *result = GameResult {
        level: current_level.level,
        is_cleared: false,
        used_bomb_count: u8::MAX,
        affected_cell_count: u8::MAX,
        mission_status: [false; 3],
    };

    let Some(level_layout) = level_assets.get(&current_level.layout) else {
        warn!("Current level layout not found in assets");
        return;
    };

    // Check if:
    // - All bombs are burned
    // - All rocks are destroyed
    // - All jewels are saved
    // - All enemies are defeated
    let is_cleared = query.iter().all(|(&item, &state, _)| match item {
        Item::BombSmall
        | Item::BombMedium
        | Item::BombLarge
        | Item::BombHorizontal
        | Item::BombVertical => state == ItemState::Burned,
        Item::Rock => state == ItemState::Burned,
        Item::Jewel => state == ItemState::None,
        Item::Enemy => state == ItemState::Burned,
        _ => true, // Other items are not relevant for the result
    });

    let bombs_list = query
        .iter()
        .filter(|(item, _state, _)| {
            matches!(
                item,
                Item::BombSmall
                    | Item::BombMedium
                    | Item::BombLarge
                    | Item::BombHorizontal
                    | Item::BombVertical
            )
        })
        .map(|(&item, &state, &coord)| (item, state, coord))
        .collect::<Vec<_>>();

    let used_bomb_count = if is_cleared {
        let total_bomb_count = bombs_list.len() as u8;

        let level_bomb_count = level_layout
            .objects
            .iter()
            .filter(|(_coord, item)| {
                matches!(
                    item,
                    Item::BombSmall
                        | Item::BombMedium
                        | Item::BombLarge
                        | Item::BombHorizontal
                        | Item::BombVertical
                )
            })
            .count() as u8;

        total_bomb_count - level_bomb_count
    } else {
        u8::MAX
    };

    let affected_cell_count = if is_cleared {
        bombs_list
            .iter()
            .flat_map(|(item, _state, coord)| {
                item.impact_zone()
                    .iter()
                    .map(move |&(dx, dy)| GridCoord {
                        x: (coord.x as i8 + dx) as u8,
                        y: (coord.y as i8 + dy) as u8,
                    })
                    .filter(|coord| {
                        // Ensure the coordinate is within the level bounds
                        coord.x < level_layout.board_size.0 && coord.y < level_layout.board_size.1
                    })
            })
            .collect::<HashSet<GridCoord>>()
            .len() as u8
    } else {
        u8::MAX
    };

    let mission_status = [
        is_cleared,
        used_bomb_count <= level_layout.meta.min_bombs,
        affected_cell_count <= level_layout.meta.min_affected_cells,
    ];

    *result = GameResult {
        level: current_level.level,
        is_cleared,
        used_bomb_count,
        affected_cell_count,
        mission_status,
    };
}

fn record_cleared_levels(
    current_level: Res<CurrentLevel>,
    game_result: Res<GameResult>,
    mut cleared_levels: ResMut<Persistent<ClearedLevels>>,
) {
    assert_eq!(
        current_level.level, game_result.level,
        "Current level and game result level must match",
    );

    if game_result.is_cleared {
        cleared_levels
            .update(|cleared_levels| {
                let current_best =
                    cleared_levels
                        .0
                        .entry(current_level.level)
                        .or_insert_with(|| GameResult {
                            level: current_level.level,
                            is_cleared: false,
                            used_bomb_count: u8::MAX,
                            affected_cell_count: u8::MAX,
                            mission_status: [false; 3],
                        });

                current_best.is_cleared |= game_result.is_cleared;
                current_best.used_bomb_count = current_best
                    .used_bomb_count
                    .min(game_result.used_bomb_count);
                current_best.affected_cell_count = current_best
                    .affected_cell_count
                    .min(game_result.affected_cell_count);

                current_best
                    .mission_status
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, status)| {
                        *status |= game_result.mission_status[i];
                    });
            })
            .ok();
    }
}

fn init_result_state(
    mut commands: Commands,
    result: Res<GameResult>,
    ui_assets: Res<UiAssets>,
    se_assets: Option<Res<SoundEffectAssets>>,
    se_volume: Res<SEVolume>,
) {
    // create UI node for graying out the whole screen

    let mut entity = commands.spawn((
        widget::ui_root("Result Screen"),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        StateScoped(GamePhase::Result),
        GlobalZIndex(2),
    ));

    if result.is_cleared {
        entity.insert(children![
            widget::header("Level Cleared!", Handle::clone(&ui_assets.font)),
            widget::text(
                format!("Used Bombs: {}", result.used_bomb_count),
                Handle::clone(&ui_assets.font)
            ),
            widget::text(
                format!("Affected Cells: {}", result.affected_cell_count),
                Handle::clone(&ui_assets.font)
            ),
            widget::text_button("Select Level", &ui_assets, go_level_select),
            widget::text_button("Retry", &ui_assets, retry_level),
            widget::text_button("Next Level", &ui_assets, next_level),
        ]);

        if let Some(se_assets) = se_assets {
            commands.spawn(sound_effect(se_assets.clear.clone(), &se_volume));
        }
    } else {
        entity.insert(children![
            widget::header("Level Failed...", Handle::clone(&ui_assets.font)),
            widget::text_button("Select Level", &ui_assets, go_level_select),
            widget::text_button("Retry", &ui_assets, retry_level),
        ]);

        if let Some(se_assets) = se_assets {
            commands.spawn(sound_effect(se_assets.failed.clone(), &se_volume));
        }
    }
}

fn update_mission_status(
    mut commands: Commands,
    cleared_levels: Res<Persistent<ClearedLevels>>,
    ui_assets: Res<UiAssets>,
    current_level: Res<CurrentLevel>,
    level_layouts: Res<Assets<LevelLayout>>,
    missions_section_query: Query<(Entity, &ChildOf), With<MissionsSection>>,
) {
    let Some(level_layout) = level_layouts.get(&current_level.layout) else {
        warn!("Current level layout not found in assets");
        return;
    };

    let game_result = cleared_levels.0.get(&current_level.level);
    for (entity, ChildOf(parent)) in missions_section_query.iter() {
        commands.entity(entity).despawn();

        commands.entity(*parent).with_child(missions_section(
            &ui_assets,
            level_layout,
            game_result,
        ));
    }
}

fn retry_level(_: Trigger<Pointer<Click>>, mut next_phase: ResMut<NextState<GamePhase>>) {
    next_phase.set(GamePhase::Init);
}

fn go_level_select(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::LevelSelect);
}

fn next_level(
    _: Trigger<Pointer<Click>>,
    level_assets: Res<LevelAssets>,
    current_level: ResMut<CurrentLevel>,
    next_phase: ResMut<NextState<GamePhase>>,
    next_screen: ResMut<NextState<Screen>>,
) {
    move_to_level(
        current_level.level + 1,
        level_assets,
        current_level,
        next_phase,
        next_screen,
    );
}

pub fn move_to_level(
    next_level: usize,
    level_assets: Res<LevelAssets>,
    mut current_level: ResMut<CurrentLevel>,
    mut next_phase: ResMut<NextState<GamePhase>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if let Some(level_handle) = level_assets.levels.get(next_level) {
        current_level.level = next_level;
        current_level.layout = Handle::clone(level_handle);
        next_screen.set(Screen::Gameplay);
        next_phase.set(GamePhase::Init);
    } else {
        warn!("Level {} does not exist", next_level);
        next_screen.set(Screen::LevelSelect);
    }
}

fn insert_persistent_resources(mut commands: Commands) {
    let data_dir = dirs::data_dir()
        .map(|native_state_dir| native_state_dir.join("bevy-persistent"))
        .unwrap_or(Path::new("local").join("data"))
        .join("bombombo");

    commands.insert_resource(
        Persistent::<ClearedLevels>::builder()
            .name("cleared levels")
            .format(StorageFormat::Ron)
            .path(data_dir.join("cleared_levels.ron"))
            .default(ClearedLevels::default())
            .build()
            .expect("failed to initialize cleared levels persistent resource"),
    )
}
