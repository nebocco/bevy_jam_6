//! The settings menu.
//!
//! Additional settings and accessibility options should go here.

use bevy::{audio::Volume, input::common_conditions::input_just_pressed, prelude::*, ui::Val::*};

use crate::{
    audio::{MusicVolume, SEVolume},
    menus::Menu,
    screens::Screen,
    theme::{UiAssets, prelude::*},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Settings), spawn_settings_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Settings).and(input_just_pressed(KeyCode::Escape))),
    );

    app.register_type::<MusicVolumeLabel>()
        .register_type::<SEVolumeLabel>();

    app.add_systems(
        Update,
        (update_music_volume_label, update_se_volume_label).run_if(in_state(Menu::Settings)),
    );
}

fn spawn_settings_menu(mut commands: Commands, ui_assets: Res<UiAssets>) {
    commands.spawn((
        widget::ui_root("Settings Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Settings),
        children![
            widget::header("Settings", Handle::clone(&ui_assets.font)),
            settings_grid(&ui_assets),
            widget::text_button("Back", &ui_assets, go_back_on_click),
        ],
    ));
}

fn settings_grid(ui_assets: &UiAssets) -> impl Bundle {
    (
        Name::new("Settings Grid"),
        Node {
            display: Display::Grid,
            row_gap: Px(10.0),
            column_gap: Px(30.0),
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            ..default()
        },
        children![
            (
                widget::label("Music"),
                Node {
                    justify_self: JustifySelf::End,
                    ..default()
                }
            ),
            music_volume_widget(ui_assets),
            (
                widget::label("Sound Effects"),
                Node {
                    justify_self: JustifySelf::End,
                    ..default()
                }
            ),
            se_volume_widget(ui_assets),
        ],
    )
}

fn music_volume_widget(ui_assets: &UiAssets) -> impl Bundle {
    (
        Name::new("Music Volume Widget"),
        Node {
            justify_self: JustifySelf::Start,
            ..default()
        },
        children![
            widget::button_small("-", ui_assets, lower_music_volume),
            (
                Name::new("Current Volume"),
                Node {
                    padding: UiRect::horizontal(Px(10.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(widget::label(""), MusicVolumeLabel)],
            ),
            widget::button_small("+", ui_assets, raise_music_volume),
        ],
    )
}

fn se_volume_widget(ui_assets: &UiAssets) -> impl Bundle {
    (
        Name::new("SE Volume Widget"),
        Node {
            justify_self: JustifySelf::Start,
            ..default()
        },
        children![
            widget::button_small("-", ui_assets, lower_se_volume),
            (
                Name::new("Current SE Volume"),
                Node {
                    padding: UiRect::horizontal(Px(10.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(widget::label(""), SEVolumeLabel)],
            ),
            widget::button_small("+", ui_assets, raise_se_volume),
        ],
    )
}

const MIN_VOLUME: f32 = 0.0;
const MAX_VOLUME: f32 = 3.0;

fn lower_music_volume(_: Trigger<Pointer<Click>>, mut music_volume: ResMut<MusicVolume>) {
    let linear = (music_volume.volume.to_linear() - 0.1).max(MIN_VOLUME);
    music_volume.volume = Volume::Linear(linear);
}

fn raise_music_volume(_: Trigger<Pointer<Click>>, mut music_volume: ResMut<MusicVolume>) {
    let linear = (music_volume.volume.to_linear() + 0.1).min(MAX_VOLUME);
    music_volume.volume = Volume::Linear(linear);
}

fn lower_se_volume(_: Trigger<Pointer<Click>>, mut se_volume: ResMut<SEVolume>) {
    let linear = (se_volume.volume.to_linear() - 0.1).max(MIN_VOLUME);
    se_volume.volume = Volume::Linear(linear);
}

fn raise_se_volume(_: Trigger<Pointer<Click>>, mut se_volume: ResMut<SEVolume>) {
    let linear = (se_volume.volume.to_linear() + 0.1).min(MAX_VOLUME);
    se_volume.volume = Volume::Linear(linear);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct MusicVolumeLabel;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct SEVolumeLabel;

fn update_music_volume_label(
    music_volume: Res<MusicVolume>,
    mut label: Single<&mut Text, With<MusicVolumeLabel>>,
) {
    let percent = 100.0 * music_volume.volume.to_linear();
    label.0 = format!("{percent:3.0}%");
}

fn update_se_volume_label(
    se_volume: Res<SEVolume>,
    mut label: Single<&mut Text, With<SEVolumeLabel>>,
) {
    let percent = 100.0 * se_volume.volume.to_linear();
    label.0 = format!("{percent:3.0}%");
}

fn go_back_on_click(
    _: Trigger<Pointer<Click>>,
    screen: Res<State<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}

fn go_back(screen: Res<State<Screen>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}
