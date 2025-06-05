use std::{collections::HashMap, fmt::Debug};

use bevy::{
    color::palettes,
    ecs::relationship::RelatedSpawnerCommands,
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    state::commands,
};

use crate::gameplay::{
    GamePhase,
    level::{GridCoord, ItemState, ObjectMap},
    setup::{self, Item},
};

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
    object_map: HashMap<GridCoord, (setup::Item, Entity)>,
    tick: u32,
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct RunningTimer(pub Timer);

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct BurningStack(Vec<(GridCoord, Item, Entity)>);

#[derive(Event, Debug, Clone, Copy, PartialEq)]
struct NextTick;

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

    let (fire_coord, fire_item) = object_map.fire.expect("Fire item not found in object map");
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
    trigger: Trigger<NextTick>,
    // mut commands: Commands,
    mut color: Query<(&mut Sprite, &mut ItemState)>,
    mut running_state: ResMut<RunningState>,
    mut burning_stack: ResMut<BurningStack>,
) {
    running_state.tick += 1;
    println!("Tick: {}", running_state.tick);
    for (coord, item, entity) in std::mem::take(&mut burning_stack.0) {
        if running_state.object_map.remove(&coord).is_none() {
            continue;
        }
        println!("Burning item: {:?} at {:?}", item, coord);
        // commands.entity(entity).insert(());

        if let Ok((mut sprite, mut item_state)) = color.get_mut(entity) {
            sprite.color = Color::Srgba(palettes::basic::RED);
            *item_state = ItemState::Burned;
        } else {
            println!("Failed to get sprite for entity: {:?}", entity);
        }

        for &(dx, dy) in item.impact_zone() {
            let new_coord = GridCoord {
                x: coord.x.wrapping_add(dx as u8),
                y: coord.y.wrapping_add(dy as u8),
            };

            if let Some((target_item, target_entity)) = running_state.object_map.get(&new_coord) {
                burning_stack
                    .0
                    .push((new_coord, *target_item, *target_entity));
            }
        }
    }
    println!("{:?}", &burning_stack);
}
