use bevy::{ecs::spawn::SpawnWith, input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    Pause,
    audio::{MusicAssets, SpawnMusic},
    gameplay::{ClearedLevels, CurrentLevel, GamePhase, LevelAssets, move_to_level},
    menus::Menu,
    screens::Screen,
    theme::{UiAssets, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::LevelSelect),
        (spawn_level_select_screen, spawn_music),
    );

    app.add_systems(
        Update,
        (
            (pause, spawn_pause_overlay, open_pause_menu).run_if(
                in_state(Screen::LevelSelect)
                    .and(in_state(Menu::None))
                    .and(input_just_pressed(KeyCode::KeyP).or(input_just_pressed(KeyCode::Escape))),
            ),
            close_menu.run_if(
                in_state(Screen::LevelSelect)
                    .and(not(in_state(Menu::None)))
                    .and(input_just_pressed(KeyCode::KeyP)),
            ),
        ),
    );
    app.add_systems(OnExit(Screen::LevelSelect), (close_menu, unpause));
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(Screen::LevelSelect)),
    );
}

fn unpause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause(false));
}

fn pause(mut next_pause: ResMut<NextState<Pause>>) {
    println!("Pausing game");
    next_pause.set(Pause(true));
}

fn spawn_pause_overlay(mut commands: Commands) {
    commands.spawn((
        Name::new("Pause Overlay"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        GlobalZIndex(1),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        StateScoped(Pause(true)),
    ));
}

fn open_pause_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Pause);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn spawn_level_select_screen(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    cleared_levels: Res<ClearedLevels>,
    level_assets: Res<LevelAssets>,
) {
    let mut entity = commands.spawn((
        widget::ui_root("Level Select Screen"),
        StateScoped(Screen::LevelSelect),
        GlobalZIndex(0),
        children![
            widget::header("Select Level"),
            stage_select_button_grid(ui_assets, &cleared_levels, &level_assets)
        ],
    ));

    // TODO: REMOVE !!!
    if cleared_levels.0.len() == level_assets.levels.len() || cleared_levels.0.len() < 3 {
        entity.with_child(widget::footer("All Levels Cleared!"));
    }
}

fn spawn_music(mut commands: Commands, music_assets: Res<MusicAssets>) {
    commands.trigger(SpawnMusic::new(Handle::clone(
        &music_assets.level_select_bgm,
    )));
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LevelStatus {
    pub is_cleared: bool,
    pub is_locked: bool,
}

fn stage_select_button_grid(
    ui_assets: Res<UiAssets>,
    cleared_levels: &ClearedLevels,
    level_assets: &LevelAssets,
) -> impl Bundle {
    let ui_assets = ui_assets.clone();
    let level_status_list = level_assets
        .levels
        .iter()
        .enumerate()
        .map(|(index, _)| LevelStatus {
            is_cleared: cleared_levels.0.contains_key(&index),
            // is_locked: (0..index).any(|i| !cleared_levels.0.contains_key(&i)) ,
            is_locked: false, // TODO: REMOVE !!!
        })
        .collect::<Vec<_>>();

    println!("Level Status List: {:?}", level_status_list);

    (
        Node {
            display: Display::Grid,
            width: Val::Percent(60.0),
            height: Val::Percent(70.0),
            grid_template_columns: vec![GridTrack::auto(); 4],
            grid_template_rows: vec![GridTrack::auto(); 4],
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            for (index, status) in level_status_list.into_iter().enumerate() {
                let mut entity_bundle =
                    parent.spawn(widget::level_button(index, &ui_assets, status));
                if !status.is_locked {
                    entity_bundle.observe(
                        move |_out: Trigger<Pointer<Click>>,
                              level_assets: Res<LevelAssets>,
                              current_level: ResMut<CurrentLevel>,
                              next_phase: ResMut<NextState<GamePhase>>,
                              next_screen: ResMut<NextState<Screen>>| {
                            move_to_level(
                                index,
                                level_assets,
                                current_level,
                                next_phase,
                                next_screen,
                            );
                        },
                    );
                }
            }
        })),
    )
}
