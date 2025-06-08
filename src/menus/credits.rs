//! The credits menu.

use bevy::{
    ecs::spawn::SpawnIter, input::common_conditions::input_just_pressed, prelude::*, ui::Val::*,
};

use crate::{
    menus::Menu,
    theme::{UiAssets, prelude::*},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Credits).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_credits_menu(mut commands: Commands, ui_assets: Res<UiAssets>) {
    commands.spawn((
        widget::ui_root("Credits Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Credits),
        children![
            widget::header("Created by", Handle::clone(&ui_assets.font)),
            // created_by(&ui_assets),
            widget::text("nebocco", Handle::clone(&ui_assets.font)),
            widget::header("Assets", Handle::clone(&ui_assets.font)),
            assets(&ui_assets),
            widget::text_button("Back", &ui_assets, go_back_on_click),
        ],
    ));
}

fn assets(ui_assets: &UiAssets) -> impl Bundle {
    grid(
        vec![
            ["Music", "by ansimuz"],
            ["SFX", "created with jsfxr, by Chris McCormick"],
            ["Sprite Animation", "by Bdragon1727"],
            ["Fonts", "by Daniel Linssen"],
        ],
        ui_assets,
    )
}

fn grid(content: Vec<[&'static str; 2]>, ui_assets: &UiAssets) -> impl Bundle {
    let font_handle = Handle::clone(&ui_assets.font);
    (
        Name::new("Grid"),
        Node {
            display: Display::Grid,
            row_gap: Px(10.0),
            column_gap: Px(30.0),
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            ..default()
        },
        Children::spawn(SpawnIter(content.into_iter().flatten().enumerate().map(
            move |(i, text)| {
                (
                    widget::text(text, Handle::clone(&font_handle)),
                    Node {
                        justify_self: if i % 2 == 0 {
                            JustifySelf::End
                        } else {
                            JustifySelf::Start
                        },
                        ..default()
                    },
                )
            },
        ))),
    )
}

fn go_back_on_click(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}
