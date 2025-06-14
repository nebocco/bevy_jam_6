use std::{collections::HashMap, fmt::Debug};

use bevy::{color::palettes, prelude::*};

use crate::{
    audio::{SEVolume, SoundEffectAssets, sound_effect},
    gameplay::{
        CurrentLevel, GamePhase, GridCoord, Item, ItemState,
        animation::AffectedTileAnimation,
        edit::{CurrentPlacement, Fire, SelectedItem, fire},
        init_level::{GridTile, ItemAssets, LevelLayout, reset_tint_colors},
    },
    theme::{
        interaction::InteractionImagePalette,
        widget::{ItemButton, RunButton},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RunningState>()
        .init_resource::<BurningStack>()
        .insert_resource(RunningTimer(Timer::from_seconds(1.2, TimerMode::Repeating)));
    app.add_systems(
        OnEnter(GamePhase::Run),
        (
            disable_buttons,
            reset_tint_colors,
            (init_run_state, record_current_placement).chain(),
        ),
    )
    .add_systems(Update, tick_timer.run_if(in_state(GamePhase::Run)))
    .add_observer(tick_simulation);
}

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct RunningState {
    object_map: HashMap<GridCoord, (Item, Entity)>,
    tick: u32,
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct RunningTimer(pub Timer);

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct BurningStack(Vec<(GridCoord, Item, Entity)>);

#[derive(Event, Debug, Clone, Copy, PartialEq)]
struct NextTick;

#[derive(Event, Debug, Clone, Copy, PartialEq)]
pub struct Explode {
    pub parent_entity: Entity,
    pub item: Item,
}

fn init_run_state(
    mut timer: ResMut<RunningTimer>,
    mut running_state: ResMut<RunningState>,
    mut burning_stack: ResMut<BurningStack>,
    item_query: Query<(Entity, &Item, &GridCoord), Without<Fire>>,
    fire_query: Query<(Entity, &GridCoord), With<Fire>>,
) {
    timer.0.reset();

    running_state.object_map = item_query
        .iter()
        .map(|(entity, &item, &coord)| (coord, (item, entity)))
        .collect();
    running_state.tick = 0;

    let (_fire_entity, fire_coord) = fire_query
        .single()
        .expect("Fire item not found in object map");

    burning_stack.0 = running_state
        .object_map
        .iter()
        .filter_map(|(&coord, &(item, entity))| {
            if coord == *fire_coord {
                Some((coord, item, entity))
            } else {
                None
            }
        })
        .collect();
}

fn disable_buttons(
    mut commands: Commands,
    mut selected_item: ResMut<SelectedItem>,
    mut buttons: Query<(&mut ImageNode, Entity), Or<(With<ItemButton>, With<RunButton>)>>,
) {
    selected_item.0 = None; // Reset selected item

    for (mut image_node, entity) in buttons.iter_mut() {
        image_node
            .texture_atlas
            .iter_mut()
            .for_each(|texture_atlas| {
                texture_atlas.index = 1; // Set to gray
            });
        image_node.color = Color::WHITE;
        commands.entity(entity).remove::<InteractionImagePalette>();
    }
}

fn record_current_placement(
    running_state: Res<RunningState>,
    current_level: Res<CurrentLevel>,
    level_assets: Res<Assets<LevelLayout>>,
    mut current_placement: ResMut<CurrentPlacement>,
) {
    let Some(level_layout) = level_assets.get(&current_level.layout) else {
        warn!("Current level layout not found in assets");
        return;
    };

    *current_placement = CurrentPlacement::new(
        current_level.level,
        running_state
            .object_map
            .iter()
            .filter(|&(&coord, &(item, _entity))| level_layout.objects.get(&coord) != Some(&item))
            .map(|(&coord, &(item, _entity))| (coord, item))
            .collect(),
    );
}

fn tick_timer(
    time: Res<Time>,
    mut running_timer: ResMut<RunningTimer>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GamePhase>>,
    burning_stack: Res<BurningStack>,
) {
    if running_timer.0.tick(time.delta()).just_finished() {
        if burning_stack.0.is_empty() {
            next_state.set(GamePhase::Result);
        }
        commands.trigger(NextTick);
    }
}

fn tick_simulation(
    _trigger: Trigger<NextTick>,
    mut commands: Commands,
    item_assets: Res<ItemAssets>,
    mut running_state: ResMut<RunningState>,
    mut burning_stack: ResMut<BurningStack>,
    mut query: Query<&mut ItemState>,
    mut tile_query: Query<(Entity, &GridCoord), With<GridTile>>,
    se_assets: Option<Res<SoundEffectAssets>>,
    se_volume: Res<SEVolume>,
) {
    running_state.tick += 1;
    let mut filtered_burning_stack: Vec<_> = std::mem::take(&mut burning_stack.0)
        .into_iter()
        .filter(|(coord, _item, _entity)| running_state.object_map.remove(coord).is_some())
        .collect();
    filtered_burning_stack.sort_by_key(|(_, _, entity)| *entity);
    filtered_burning_stack.dedup_by_key(|(_, _, entity)| *entity);

    let affected_area = compute_affected_area(&filtered_burning_stack);

    // set affected tile animation
    for (tile_entity, coord) in &mut tile_query {
        if affected_area.iter().any(|&(c, _)| c == *coord) {
            commands.entity(tile_entity).with_children(|parent| {
                parent.spawn((
                    Name::new("Burning Tile Animation"),
                    Sprite::from_color(palettes::css::RED, Vec2::splat(60.0)),
                    AffectedTileAnimation::new(),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                ));
            });
        }
    }

    let affected_objects: Vec<_> = affected_area
        .iter()
        .filter_map(|&(coord, _count)| {
            running_state
                .object_map
                .get(&coord)
                .map(|&(item, entity)| (coord, item, entity))
        })
        .collect();

    let affected_items: Vec<_> = affected_objects
        .iter()
        .filter(|&&(_, item, _)| !item.is_bomb())
        .cloned()
        .collect();

    let affected_bombs: Vec<_> = affected_objects
        .into_iter()
        .filter(|&(_, item, _)| item.is_bomb())
        .collect();

    // remove affected items from the object map
    affected_items.iter().for_each(|(coord, _item, _entity)| {
        running_state.object_map.remove(coord);
    });

    // SE
    if let Some(se_assets) = se_assets.as_ref() {
        if !affected_items.is_empty() {
            commands.spawn(sound_effect(
                Handle::clone(&se_assets.explosion_1),
                &se_volume,
            ));
        }
    }

    // explode animation
    filtered_burning_stack
        .iter()
        .filter(|&(_, item, _)| item.is_bomb())
        .chain(affected_items.iter())
        .for_each(|&(_coord, item, entity)| {
            let mut item_state = query.get_mut(entity).expect("Entity not found in query");
            *item_state = ItemState::Burned;
            commands.trigger(Explode {
                item,
                parent_entity: entity,
            });
        });

    // SE
    if let Some(se_assets) = se_assets {
        if !filtered_burning_stack.is_empty() {
            commands.spawn(sound_effect(
                Handle::clone(&se_assets.explosion_2),
                &se_volume,
            ));
        }
    }

    // set fire animation for affected bombs
    affected_bombs.iter().for_each(|&(coord, _item, entity)| {
        commands
            .entity(entity)
            .with_child(fire(coord, &item_assets));
    });

    // preserve bombs in the burning stack
    burning_stack.0 = affected_bombs;
}

fn compute_affected_area(burning_stack: &[(GridCoord, Item, Entity)]) -> Vec<(GridCoord, usize)> {
    burning_stack
        .iter()
        .filter(|(_, item, _)| item.is_bomb())
        .flat_map(|(coord, item, _)| {
            item.impact_zone().iter().map(move |&(dx, dy)| {
                let new_coord = GridCoord {
                    x: coord.x.wrapping_add(dx as u8),
                    y: coord.y.wrapping_add(dy as u8),
                };
                (new_coord, 1) // Count each affected area once
            })
        })
        .fold(HashMap::new(), |mut acc, (coord, count)| {
            *acc.entry(coord).or_insert(0) += count;
            acc
        })
        .into_iter()
        .collect::<Vec<_>>()
}
