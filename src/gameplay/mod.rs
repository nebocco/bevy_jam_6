//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

use crate::screens::Screen;

mod level;
mod result;
mod run;
mod setup;

pub(super) fn plugin(app: &mut App) {
    app.add_sub_state::<GamePhase>().add_plugins((
        setup::plugin,
        level::plugin,
        run::plugin,
        result::plugin,
    ));
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(Screen = Screen::Gameplay)]
enum GamePhase {
    #[default]
    Setup,
    Run,
    Result,
}
