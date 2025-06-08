//! Reusable UI widgets & theming.

// Unused utilities may trigger this lints undesirably.
#![allow(dead_code)]

pub mod interaction;
pub mod palette;
pub mod widget;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::{interaction::InteractionPalette, palette as ui_palette, widget};
}

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<UiAssets>();
    app.add_plugins(interaction::plugin);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct UiAssets {
    #[dependency]
    ui_texture: Handle<Image>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    #[dependency]
    pub font: Handle<Font>,
}

impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let texture_atlas_layout = {
            let mut texture_atlas = world.resource_mut::<Assets<TextureAtlasLayout>>();
            texture_atlas.add(TextureAtlasLayout::from_grid(
                UVec2::splat(32),
                4,
                8,
                None,
                None,
            ))
        };

        let assets = world.resource::<AssetServer>();

        Self {
            ui_texture: assets.load_with_settings(
                "images/ui_sprite_sheet.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            texture_atlas_layout,
            font: assets.load("fonts/m6x11.ttf"),
        }
    }
}
