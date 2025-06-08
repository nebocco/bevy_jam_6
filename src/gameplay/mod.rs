//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::screens::Screen;

mod animation;
mod edit;
mod init_level;
mod result;
mod run;

use animation::FireAnimation;
pub use init_level::{BgAssets, CurrentLevel, LevelAssets};
use init_level::{ItemAssets, ItemState, LevelLayout};
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

#[derive(SubStates, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[source(Screen = Screen::Gameplay)]
#[states(scoped_entities)]
pub enum GamePhase {
    #[default]
    Init,
    Edit,
    Run,
    Result,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Reflect)]
pub struct GridCoord {
    pub x: u8,
    pub y: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Reflect)]
pub enum Item {
    BombSmall,
    BombMedium,
    BombLarge,
    BombHorizontal,
    BombVertical,
    Null,
    Rock,
    Gem,
    Eraser,
    Enemy,
}

impl Item {
    pub fn is_bomb(&self) -> bool {
        matches!(
            self,
            Item::BombSmall
                | Item::BombMedium
                | Item::BombLarge
                | Item::BombHorizontal
                | Item::BombVertical
        )
    }

    pub const fn to_sprite_index(&self) -> usize {
        match self {
            Item::BombSmall => 0,
            Item::BombMedium => 1,
            Item::BombLarge => 2,
            Item::BombHorizontal => 3,
            Item::BombVertical => 4,
            Item::Null => 7,
            Item::Rock => 8,
            Item::Gem => 10,
            Item::Enemy => 11,
            Item::Eraser => 12,
        }
    }
}

impl Item {
    pub fn impact_zone(&self) -> &'static [(i8, i8)] {
        match self {
            // . . . . .
            // . x x x .
            // . x # x .
            // . x x x.
            // . . . . .
            Item::BombSmall => &[
                (-1, 1),
                (0, 1),
                (1, 1),
                (-1, 0),
                (0, 0),
                (1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
            ],

            // . . x . .
            // . x x x .
            // x x # x x
            // . x x x .
            // . . x . .
            Item::BombMedium => &[
                (0, 2),
                (-1, 1),
                (0, 1),
                (1, 1),
                (-2, 0),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (0, -2),
            ],

            // . . . x . . .
            // . . x x x . .
            // . x x x x x .
            // x x x # x x x
            // . x x x x x .
            // . . x x x . .
            // . . . x . . .
            Item::BombLarge => &[
                (0, 3),
                (-1, 2),
                (0, 2),
                (1, 2),
                (-2, 1),
                (-1, 1),
                (0, 1),
                (1, 1),
                (2, 1),
                (-3, 0),
                (-2, 0),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (-2, -1),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (-1, -2),
                (0, -2),
                (1, -2),
                (0, -3),
            ],

            // . . x . .
            // . . x . .
            // . . # . .
            // . . x . .
            // . . x . .
            Item::BombVertical => &[
                (0, 10),
                (0, 9),
                (0, 8),
                (0, 7),
                (0, 6),
                (0, 5),
                (0, 4),
                (0, 3),
                (0, 2),
                (0, 1),
                (0, 0),
                (0, -1),
                (0, -2),
                (0, -3),
                (0, -4),
                (0, -5),
                (0, -6),
                (0, -7),
                (0, -8),
                (0, -9),
                (0, -10),
            ],

            // . . . . .
            // . . . . .
            // x x # x x
            // . . . . .
            // . . . . .
            Item::BombHorizontal => &[
                (10, 0),
                (9, 0),
                (8, 0),
                (7, 0),
                (6, 0),
                (5, 0),
                (4, 0),
                (3, 0),
                (2, 0),
                (1, 0),
                (0, 0),
                (-1, 0),
                (-2, 0),
                (-3, 0),
                (-4, 0),
                (-5, 0),
                (-6, 0),
                (-7, 0),
                (-8, 0),
                (-9, 0),
                (-10, 0),
            ],

            Item::Eraser => &[(0, 0)],

            Item::Rock | Item::Gem | Item::Enemy | Item::Null => &[],
        }
    }
}

impl From<u8> for Item {
    fn from(value: u8) -> Self {
        match value {
            0 => Item::BombSmall,
            1 => Item::BombMedium,
            2 => Item::BombLarge,
            3 => Item::BombHorizontal,
            4 => Item::BombVertical,
            255 => Item::Eraser,
            _ => panic!("Invalid item index"),
        }
    }
}
