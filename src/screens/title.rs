//! The title screen that appears after the splash screen.

use bevy::prelude::*;

use crate::{gameplay::BgAssets, menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), (open_main_menu, set_background));
    app.add_systems(OnExit(Screen::Title), close_menu);
}

fn open_main_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn set_background(mut commands: Commands, bg_assets: Res<BgAssets>) {
    commands.spawn((
        Sprite {
            image: Handle::clone(&bg_assets.bg_image),
            ..default()
        },
        Transform::from_scale(Vec2::splat(2.0).extend(1.0)),
        GlobalZIndex(0),
    ));
}
