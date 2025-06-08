use bevy::prelude::*;

mod color_animation;
mod sprite_animation;

pub use color_animation::AffectedTileAnimation;
pub use sprite_animation::FireAnimation;

pub fn plugin(app: &mut App) {
    app.add_plugins((sprite_animation::plugin, color_animation::plugin));
}
