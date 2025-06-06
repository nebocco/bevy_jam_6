use std::fmt::Debug;

use bevy::prelude::*;

use crate::{
    gameplay::{ClearedLevels, CurrentLevel, LevelAssets, move_to_level},
    screens::Screen,
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::LevelSelect), spawn_level_select_screen);
}

fn spawn_level_select_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Level Select Screen"),
        StateScoped(Screen::LevelSelect),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        GlobalZIndex(2),
        children![
            widget::label("Select Level"),
            // Add buttons for each level here
            stage_select_button_grid()
        ],
    ));
}

fn stage_select_button_grid() -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            width: Val::Percent(80.0),
            height: Val::Percent(70.0),
            grid_template_columns: vec![GridTrack::auto(); 4],
            grid_template_rows: vec![GridTrack::auto(); 4],
            ..default()
        },
        children![
            widget::button("Level 1", select_level::<0>),
            widget::button("Level 2", select_level::<1>),
            widget::button("Level 3", select_level::<2>),
        ],
    )
}

fn select_level<const L: usize>(
    _out: Trigger<Pointer<Click>>,
    level_assets: Res<LevelAssets>,
    current_level: ResMut<CurrentLevel>,
    next_screen: ResMut<NextState<Screen>>,
) {
    info!("Selecting Level: {}", L);
    move_to_level(L, level_assets, current_level, next_screen);
}
