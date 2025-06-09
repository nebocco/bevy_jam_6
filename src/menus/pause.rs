//! The pause menu.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    menus::Menu,
    screens::Screen,
    theme::{UiAssets, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Pause).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_pause_menu(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    current_screen: Res<State<Screen>>,
) {
    if current_screen.get() != &Screen::LevelSelect {
        // If the current screen is not LevelSelect, we don't spawn the pause menu.
        commands.spawn((
            widget::ui_root("Pause Menu"),
            GlobalZIndex(2),
            StateScoped(Menu::Pause),
            children![
                widget::header("Game paused", Handle::clone(&ui_assets.font)),
                widget::text_button("Resume", &ui_assets, close_menu),
                widget::text_button("Settings", &ui_assets, open_settings_menu),
                widget::text_button("Select Level", &ui_assets, back_to_level_select),
            ],
        ));
    } else {
        // If the current screen is LevelSelect, we spawn a different pause menu.
        commands.spawn((
            widget::ui_root("Pause Menu"),
            GlobalZIndex(2),
            StateScoped(Menu::Pause),
            children![
                widget::header("Game paused", Handle::clone(&ui_assets.font)),
                widget::text_button("Resume", &ui_assets, close_menu),
                widget::text_button("Settings", &ui_assets, open_settings_menu),
                widget::text_button("Quit to title", &ui_assets, quit_to_title),
            ],
        ));
    }
}

fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn close_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn back_to_level_select(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::LevelSelect);
}

fn quit_to_title(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
