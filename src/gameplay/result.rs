use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    gameplay::{CurrentLevel, GamePhase, Item, ItemState, LevelAssets, LevelLayout},
    screens::Screen,
    theme::{UiAssets, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameResult>()
        .init_resource::<ClearedLevels>();
    app.add_systems(
        OnEnter(GamePhase::Result),
        (
            compute_game_result,
            record_cleated_levels,
            init_result_state,
        )
            .chain(),
    );
}

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct ClearedLevels(pub HashMap<usize, GameResult>);

#[derive(Resource, Reflect, Debug, Default, Clone, PartialEq)]
#[reflect(Resource)]
pub struct GameResult {
    level: usize,
    is_cleared: bool,
    used_bomb_count: u32,
}

fn compute_game_result(
    current_level: Res<CurrentLevel>,
    level_assets: Res<Assets<LevelLayout>>,
    query: Query<(&Item, &ItemState)>,
    mut result: ResMut<GameResult>,
) {
    *result = GameResult {
        level: current_level.level,
        is_cleared: false,
        used_bomb_count: 0,
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
    let is_cleared = query.iter().all(|(&item, &state)| match item {
        Item::BombSmall
        | Item::BombMedium
        | Item::BombLarge
        | Item::BombHorizontal
        | Item::BombVertical => state == ItemState::Burned,
        Item::Rock => state == ItemState::Burned,
        Item::Gem => state == ItemState::None,
        Item::Enemy => state == ItemState::Burned,
        _ => true, // Other items are not relevant for the result
    });

    let total_bomb_count = query
        .iter()
        .filter(|(item, _state)| {
            matches!(
                item,
                Item::BombSmall
                    | Item::BombMedium
                    | Item::BombLarge
                    | Item::BombHorizontal
                    | Item::BombVertical
            )
        })
        .count() as u32;

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
        .count() as u32;

    let used_bomb_count = total_bomb_count - level_bomb_count;
    *result = GameResult {
        level: current_level.level,
        is_cleared,
        used_bomb_count,
    };
}

fn record_cleated_levels(
    current_level: Res<CurrentLevel>,
    game_result: Res<GameResult>,
    mut cleared_levels: ResMut<ClearedLevels>,
) {
    assert_eq!(
        current_level.level, game_result.level,
        "Current level and game result level must match",
    );

    if game_result.is_cleared {
        cleared_levels
            .0
            .insert(current_level.level, game_result.clone());
        info!(
            "Level {} cleared! Used {} bombs.",
            current_level.level, game_result.used_bomb_count
        );
    } else {
        info!("Level {} failed.", current_level.level);
    };
}

fn init_result_state(mut commands: Commands, result: Res<GameResult>, ui_assets: Res<UiAssets>) {
    // create UI node for graying out the whole screen

    let mut entity = commands.spawn((
        widget::ui_root("Result Screen"),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        StateScoped(GamePhase::Result),
        GlobalZIndex(2),
    ));

    if result.is_cleared {
        entity.insert(children![
            widget::header("Level Cleared!"),
            widget::text_button("Select Level", &ui_assets, go_level_select),
            widget::text_button("Next Level", &ui_assets, next_level),
        ]);
    } else {
        entity.insert(children![
            widget::header("Level Failed..."),
            widget::text_button("Home", &ui_assets, go_level_select),
            widget::text_button("Retry", &ui_assets, retry_level),
        ]);
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
    info!("Moving to Level: {}", next_level);
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
