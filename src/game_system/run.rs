use std::{collections::HashMap, fmt::Debug};

use bevy::{
    color::palettes,
    ecs::relationship::RelatedSpawnerCommands,
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    state::commands,
};

use crate::game_system::{
    GamePhase,
    level::{GridCoord, ObjectMap},
    setup::{self, Item},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RunningState>()
        .init_resource::<BurningStack>()
        .insert_resource(RunningTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
    app.add_systems(OnEnter(GamePhase::Run), init_run_state)
        .add_systems(Update, tick_timer.run_if(in_state(GamePhase::Run)));
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
    mut running_state: ResMut<RunningState>,
    object_map: Res<ObjectMap>,
    mut timer: ResMut<RunningTimer>,
) {
    running_state.object_map = object_map.objects.clone();
    running_state.tick = 0;

    timer.0.reset();
}

fn tick_timer(time: Res<Time>, mut running_timer: ResMut<RunningTimer>, mut commands: Commands) {
    if running_timer.0.tick(time.delta()).just_finished() {
        commands.trigger(NextTick);
    }
}

fn tick_simulation(
    trigger: Trigger<NextTick>,
    mut running_state: ResMut<RunningState>,
    mut burning_stack: ResMut<BurningStack>,
) {
    for (coord, item, entity) in std::mem::take(&mut burning_stack.0) {
        if running_state.object_map.remove(&coord).is_none() {
            continue;
        }
        println!("Burning item: {:?} at {:?}", item, coord);
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
