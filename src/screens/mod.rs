//! The game's main screen states and transitions between them.

mod gameplay;
mod level_select;
mod loading;
mod splash;
mod title;

use bevy::prelude::*;
pub use level_select::LevelStatus;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((
        gameplay::plugin,
        loading::plugin,
        splash::plugin,
        level_select::plugin,
        title::plugin,
    ));
}

/// The game's main screen states.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum Screen {
    #[default]
    Splash,
    Title,
    LevelSelect,
    Loading,
    Gameplay,
}
