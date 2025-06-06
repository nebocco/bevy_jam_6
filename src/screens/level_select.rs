use std::fmt::Debug;

use bevy::prelude::*;

use crate::{gameplay::CurrentLevel, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ClearedLevels>()
        .add_systems(OnEnter(Screen::LevelSelect), spawn_level_select_screen);
}

#[derive(Resource, Debug, Clone, PartialEq, Default)]
struct ClearedLevels(Vec<usize>);

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
    mut current_level: ResMut<CurrentLevel>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    // Here you would typically load the level data and transition to the gameplay screen.
    // For now, we just log the selected level.
    info!("Selected Level: {}", L);
    current_level.0 = L; // Set the current level to the selected one
    next_state.set(Screen::Gameplay); // Transition to gameplay screen
}
