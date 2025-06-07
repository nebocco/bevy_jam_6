use bevy::{ecs::spawn::SpawnWith, prelude::*};

use crate::{
    gameplay::{ClearedLevels, CurrentLevel, GamePhase, LevelAssets, move_to_level},
    screens::Screen,
    theme::{UiAssets, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::LevelSelect), spawn_level_select_screen);
}

fn spawn_level_select_screen(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    cleared_levels: Res<ClearedLevels>,
    level_assets: Res<LevelAssets>,
) {
    commands.spawn((
        widget::ui_root("Level Select Screen"),
        StateScoped(Screen::LevelSelect),
        GlobalZIndex(2),
        children![
            widget::label("Select Level"),
            stage_select_button_grid(ui_assets, cleared_levels, level_assets)
        ],
    ));
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LevelStatus {
    pub is_cleared: bool,
    pub is_locked: bool,
}

fn stage_select_button_grid(
    ui_assets: Res<UiAssets>,
    cleared_levels: Res<ClearedLevels>,
    level_assets: Res<LevelAssets>,
) -> impl Bundle {
    let ui_assets = ui_assets.clone();
    let level_status_list = level_assets
        .levels
        .iter()
        .enumerate()
        .map(|(index, _)| LevelStatus {
            is_cleared: cleared_levels.0.contains_key(&index),
            is_locked: (0..index).any(|i| !cleared_levels.0.contains_key(&i)) && false, // TODO: REMOVE !!!
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
