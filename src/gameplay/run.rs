use std::{collections::HashMap, fmt::Debug};

use bevy::prelude::*;

use crate::gameplay::{GamePhase, GridCoord, Item, ItemState, ObjectMap};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RunningState>()
        .init_resource::<BurningStack>()
        .insert_resource(RunningTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
    app.add_systems(OnEnter(GamePhase::Run), init_run_state)
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
    object_map: Res<ObjectMap>,
) {
    println!("Initializing run state...");
    timer.0.reset();

    running_state.object_map = object_map.objects.clone();
    running_state.tick = 0;

    let (fire_coord, _fire_entity) = object_map.fire.expect("Fire item not found in object map");
    let (burning_item, burning_item_entity) = object_map
        .objects
        .get(&fire_coord)
        .cloned()
        .expect("Fire item not found in object map");
    burning_stack.0 = vec![(fire_coord, burning_item, burning_item_entity)];
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
            println!("No items to burn, skipping tick.");
            next_state.set(GamePhase::Result);
        }
        commands.trigger(NextTick);
    }
}

fn tick_simulation(
    _trigger: Trigger<NextTick>,
    mut commands: Commands,
    mut running_state: ResMut<RunningState>,
    mut burning_stack: ResMut<BurningStack>,
    mut query: Query<&mut ItemState>,
) {
    running_state.tick += 1;
    println!("Tick: {}", running_state.tick);
    println!("Burning stack: {:?}", &burning_stack);
    let mut filtered_burning_stack: Vec<_> = std::mem::take(&mut burning_stack.0)
        .into_iter()
        .filter(|(coord, _item, _entity)| running_state.object_map.remove(&coord).is_some())
        .collect();
    filtered_burning_stack.sort_by_key(|(_, _, entity)| *entity);
    filtered_burning_stack.dedup_by_key(|(_, _, entity)| *entity);

    // explode animation
    filtered_burning_stack
        .iter()
        .for_each(|&(coord, item, entity)| {
            let mut item_state = query.get_mut(entity).expect("Entity not found in query");
            *item_state = ItemState::Burned;
            commands.trigger(Explode {
                item,
                parent_entity: entity,
            });
        });

    let affected_area = compute_affected_area(&filtered_burning_stack);

    let affected_items: Vec<_> = affected_area
        .iter()
        .filter_map(|&(coord, _count)| {
            running_state
                .object_map
                .get(&coord)
                .map(|&(item, entity)| (coord, item, entity))
        })
        .collect();
    burning_stack.0 = affected_items;

    println!("next stack: {:?}", &burning_stack);
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
