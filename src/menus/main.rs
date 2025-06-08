//! The main menu (seen on the title screen).

use bevy::prelude::*;

use crate::{
    asset_tracking::ResourceHandles,
    menus::Menu,
    screens::Screen,
    theme::{UiAssets, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands, ui_assets: Res<UiAssets>) {
    commands.spawn((
        widget::ui_root("Main Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Main),
        children![(
            Name::new("Container"),
            Node {
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            children![title_logo(&ui_assets), menu_buttons(&ui_assets),],
        )],
    ));
}

fn title_logo(ui_assets: &UiAssets) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Row,
            row_gap: Val::Px(10.0),
            ..default()
        },
        children![
            widget::title("Bombombo", Handle::clone(&ui_assets.font)),
            widget::title_logo(ui_assets)
        ],
    )
}

fn menu_buttons(ui_assets: &UiAssets) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        #[cfg(not(target_family = "wasm"))]
        children![
            widget::text_button("Play", &ui_assets, enter_loading_or_gameplay_screen),
            widget::text_button("Settings", &ui_assets, open_settings_menu),
            widget::text_button("Credits", &ui_assets, open_credits_menu),
            widget::text_button("Exit", &ui_assets, exit_app),
        ],
        #[cfg(target_family = "wasm")]
        children![
            widget::text_button("Play", &ui_assets, enter_loading_or_gameplay_screen),
            widget::text_button("Settings", &ui_assets, open_settings_menu),
            widget::text_button("Credits", &ui_assets, open_credits_menu),
        ],
    )
}

fn enter_loading_or_gameplay_screen(
    _: Trigger<Pointer<Click>>,
    resource_handles: Res<ResourceHandles>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if resource_handles.is_all_done() {
        next_screen.set(Screen::LevelSelect);
    } else {
        next_screen.set(Screen::Loading);
    }
}

fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: Trigger<Pointer<Click>>, mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
