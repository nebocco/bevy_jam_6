//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

use crate::screens::Screen;

mod animation;
mod edit;
mod init_level;
mod result;
mod run;

pub use init_level::{CurrentLevel, LevelAssets};
use init_level::{GridCoord, Item, ItemAssets, ItemState};
pub use result::{ClearedLevels, move_to_level};

pub(super) fn plugin(app: &mut App) {
    app.add_sub_state::<GamePhase>().add_plugins((
        animation::plugin,
        edit::plugin,
        init_level::plugin,
        result::plugin,
        run::plugin,
    ));
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(Screen = Screen::Gameplay)]
#[states(scoped_entities)]
enum GamePhase {
    #[default]
    Init,
    Edit,
    Run,
    Result,
}
