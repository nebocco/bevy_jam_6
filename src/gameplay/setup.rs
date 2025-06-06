use bevy::prelude::*;

use crate::{
    PausableSystems,
    gameplay::{GamePhase, GridCoord, Item, ItemAssets, ItemState},
    screens::Screen,
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    // app.register_type::<Item>();

    // app.register_type::<ItemAssets>();

    app.init_resource::<SelectedItem>();
    app.add_systems(OnEnter(Screen::Gameplay), spawn_item_buttons)
        .add_plugins(MeshPickingPlugin)
        .init_resource::<ObjectMap>()
        .add_event::<CreateObject>()
        .add_observer(create_object)
        .add_systems(
            Update,
            (reset_all_object_placements, run_simulation)
                .run_if(in_state(GamePhase::Setup))
                .in_set(PausableSystems),
        );
}

fn spawn_item_buttons(mut commands: Commands, item_assets: Res<ItemAssets>) {
    commands
        .spawn((
            widget::ui_root("Item Buttons"),
            GlobalZIndex(2),
            StateScoped(Screen::Gameplay),
            children![
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    Handle::clone(&item_assets.texture_atlas_layout),
                    0,
                    select_item::<0>
                ),
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    Handle::clone(&item_assets.texture_atlas_layout),
                    1,
                    select_item::<1>
                ),
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    Handle::clone(&item_assets.texture_atlas_layout),
                    2,
                    select_item::<2>
                ),
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    Handle::clone(&item_assets.texture_atlas_layout),
                    8,
                    select_item::<255> // Eraser
                ),
            ],
            BackgroundColor(Color::WHITE),
        ))
        .insert(Node {
            position_type: PositionType::Absolute,
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            left: Val::Percent(60.0),
            ..Default::default()
        });
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ObjectMap {
    pub objects: std::collections::HashMap<GridCoord, (Item, Entity)>,
    pub fire: Option<(GridCoord, Entity)>,
}

#[derive(Debug, Clone, Copy, Event)]
pub struct CreateObject {
    pub parent_grid: Entity,
    pub coord: GridCoord,
    pub item: Item,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub(super) struct SelectedItem(pub Option<Item>);

fn select_item<const I: u8>(_: Trigger<Pointer<Click>>, mut selected_item: ResMut<SelectedItem>) {
    let item = Item::from(I);
    selected_item.0 = if selected_item.0 == Some(item) {
        println!("Deselecting item: {:?}", item);
        None
    } else {
        println!("Selected item: {:?}", item);
        Some(item)
    }
}

// create item on grid click
fn create_object(
    trigger: Trigger<CreateObject>,
    mut commands: Commands,
    item_assets: Res<ItemAssets>,
    mut object_map: ResMut<ObjectMap>,
) {
    let event = trigger.event();
    println!("Creating object at coord: {:?}", event.coord);
    println!("{:?}", &object_map.objects);

    if event.item == Item::Fire {
        try_create_fire(event, &mut commands, item_assets, &mut object_map);
        return;
    }

    if let Some((_, existing_entity)) = object_map.objects.remove(&event.coord) {
        commands.entity(existing_entity).despawn();
    }

    if event.item == Item::Eraser {
        return;
    }

    let entity = commands
        .spawn((
            Name::new("Item Object"),
            GridCoord::clone(&event.coord),
            Item::clone(&event.item),
            ItemState::None,
            Sprite::from_atlas_image(
                item_assets.sprite_sheet.clone(),
                TextureAtlas {
                    layout: item_assets.texture_atlas_layout.clone(),
                    index: event.item as usize,
                },
            ),
            Transform::from_scale(Vec3::splat(2.0)),
            StateScoped(Screen::Gameplay),
        ))
        .id();

    commands.entity(event.parent_grid).add_child(entity);
    object_map.objects.insert(event.coord, (event.item, entity));
}

fn try_create_fire(
    event: &CreateObject,
    commands: &mut Commands,
    item_assets: Res<ItemAssets>,
    object_map: &mut ResMut<ObjectMap>,
) {
    // if there is no bomb at the coordinate, do nothing
    match object_map.objects.get(&event.coord) {
        Some((item, _)) if item.is_bomb() => {}
        _ => {
            return;
        }
    }

    if let Some((fire_coord, fire_entity)) = object_map.fire.take() {
        commands.entity(fire_entity).despawn();
        if fire_coord == event.coord {
            println!("Fire already exists at this coordinate, skipping creation.");
            return;
        }
    }
    let entity = commands
        .spawn((
            Name::new("Fire Object"),
            GridCoord::clone(&event.coord),
            Item::Fire,
            Sprite::from_atlas_image(
                item_assets.sprite_sheet.clone(),
                TextureAtlas {
                    layout: item_assets.texture_atlas_layout.clone(),
                    index: 5,
                },
            ),
            Transform::from_scale(Vec3::splat(2.0)),
            StateScoped(Screen::Gameplay),
        ))
        .id();
    commands.entity(event.parent_grid).add_child(entity);
    object_map.fire = Some((event.coord, entity));
}

fn reset_all_object_placements(
    button_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut object_map: ResMut<ObjectMap>,
) {
    if button_input.just_pressed(KeyCode::KeyR) {
        for (_key, (_item, entity)) in object_map.objects.drain() {
            commands.entity(entity).despawn();
        }
    }
}

fn run_simulation(
    button_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    if button_input.just_pressed(KeyCode::Space) {
        println!("Running simulation...");
        next_state.set(GamePhase::Run);
    }
}
