//! The title screen that appears after the splash screen.

use bevy::prelude::*;

use crate::{gameplay::BgAssets, menus::Menu, screens::Screen, theme::palette::*};

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

fn set_background(mut commands: Commands, _bg_assets: Res<BgAssets>) {
    commands.spawn((
        // Sprite {
        //     image: Handle::clone(&bg_assets.bg_image),
        //     ..default()
        // },
        Sprite::from_color(MAIN_COLOR, Vec2::splat(900.0)),
        Transform::from_scale(Vec2::splat(2.0).extend(1.0))
            .with_translation(Vec3::new(0.0, 0.0, 0.1)),
        GlobalZIndex(0),
    ));
}
