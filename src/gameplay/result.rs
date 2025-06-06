use bevy::prelude::*;

use crate::{
    gameplay::{CurrentLevel, GamePhase},
    screens::Screen,
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GamePhase::Result), init_result_state);
}

fn init_result_state(mut commands: Commands) {
    // create UI node for graying out the whole screen

    commands.spawn((
        widget::ui_root("Result Screen"),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        StateScoped(GamePhase::Result),
        GlobalZIndex(2),
        children![
            widget::button("Retry", retry_level),
            widget::button("Home", go_home),
            widget::button("Next Level", next_level),
        ],
    ));
}

fn retry_level(_: Trigger<Pointer<Click>>, mut next_phase: ResMut<NextState<GamePhase>>) {
    next_phase.set(GamePhase::Setup);
}

fn go_home(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn next_level(
    _: Trigger<Pointer<Click>>,
    mut next_phase: ResMut<NextState<GamePhase>>,
    mut current_level: ResMut<CurrentLevel>,
) {
    current_level.0 += 1; // Increment the current level
    next_phase.set(GamePhase::Setup);
}
