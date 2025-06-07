use bevy::prelude::*;

use crate::{
    PausableSystems,
    gameplay::{FireAnimation, GamePhase, GridCoord, Item, ItemAssets, ItemState},
    screens::Screen,
    theme::{UiAssets, widget},
};

pub(super) fn plugin(app: &mut App) {
    // app.register_type::<Item>();

    // app.register_type::<ItemAssets>();

    app.init_resource::<SelectedItem>();

    app.add_observer(create_object)
        .add_observer(try_create_single_fire);

    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_item_buttons, spawn_controlflow_buttons),
    )
    .add_systems(OnEnter(GamePhase::Edit), init_edit_state)
    .add_systems(
        Update,
        (reset_all_object_placements, run_simulation)
            .run_if(in_state(GamePhase::Edit))
            .in_set(PausableSystems),
    );
}

fn spawn_item_buttons(
    mut commands: Commands,
    item_assets: Res<ItemAssets>,
    ui_assets: Res<UiAssets>,
) {
    let ui_assets = ui_assets.clone();
    commands
        .spawn((
            widget::ui_root("Item Buttons"),
            GlobalZIndex(0),
            StateScoped(Screen::Gameplay),
            children![
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    &ui_assets,
                    Handle::clone(&item_assets.texture_atlas_layout),
                    0,
                    select_item::<0>
                ),
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    &ui_assets,
                    Handle::clone(&item_assets.texture_atlas_layout),
                    1,
                    select_item::<1>
                ),
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    &ui_assets,
                    Handle::clone(&item_assets.texture_atlas_layout),
                    2,
                    select_item::<2>
                ),
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    &ui_assets,
                    Handle::clone(&item_assets.texture_atlas_layout),
                    3,
                    select_item::<3>
                ),
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    &ui_assets,
                    Handle::clone(&item_assets.texture_atlas_layout),
                    4,
                    select_item::<4>
                ),
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    &ui_assets,
                    Handle::clone(&item_assets.texture_atlas_layout),
                    12,
                    select_item::<255> // Eraser
                ),
            ],
        ))
        .insert(Node {
            position_type: PositionType::Absolute,
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            left: Val::Percent(80.0),
            ..Default::default()
        });
}

fn spawn_controlflow_buttons(mut commands: Commands, ui_assets: Res<UiAssets>) {
    commands
        .spawn((
            widget::ui_root("Control Flow Buttons"),
            GlobalZIndex(0),
            StateScoped(Screen::Gameplay),
        ))
        .insert(Node {
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            height: Val::Percent(80.0),
            top: Val::Percent(10.0),
            left: Val::Percent(10.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(widget::menu_button(&ui_assets));
            parent.spawn(widget::run_button(&ui_assets));
        });
}

fn init_edit_state(mut selected_item: ResMut<SelectedItem>) {
    selected_item.0 = None; // Reset selected item
}

#[derive(Debug, Clone, Event)]
pub struct CreateObject {
    pub parent_grid: Entity,
    pub coord: GridCoord,
    pub item: Item,
}

#[derive(Event, Debug, Clone)]
pub struct CreateFire {
    pub _parent_grid: Entity,
    pub coord: GridCoord,
}

#[derive(Component, Debug)]
#[require(FireAnimation)]
pub struct Fire;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub(super) struct SelectedItem(pub Option<Item>);

fn select_item<const I: u8>(
    _: Trigger<Pointer<Click>>,
    mut selected_item: ResMut<SelectedItem>,
    game_phase: Res<State<GamePhase>>,
) {
    let item = Item::from(I);
    selected_item.0 = if selected_item.0 == Some(item) || *game_phase.get() != GamePhase::Edit {
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
    query: Query<(Entity, &Item, &GridCoord)>,
) {
    let event = trigger.event();
    println!("Creating object at coord: {:?}", event.coord);
    println!("{:?}", query.iter().collect::<Vec<_>>());

    if let Some((existing_entity, _item, _coord)) =
        query.iter().find(|&(_, _, coord)| coord == &event.coord)
    {
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
            Transform::from_scale(Vec3::splat(2.0)).with_translation(Vec3::new(0.0, 0.0, 1.0)),
            StateScoped(Screen::Gameplay),
        ))
        .id();

    commands.entity(event.parent_grid).add_child(entity);
}

fn try_create_single_fire(
    trigger: Trigger<CreateFire>,
    mut commands: Commands,
    item_query: Query<(Entity, &Item, &GridCoord), Without<Fire>>,
    fire_query: Query<(Entity, &GridCoord), With<Fire>>,
    item_assets: Res<ItemAssets>,
) {
    println!("Attempting to create fire at coord: {:?}", trigger.coord);
    // if there is no bomb at the coordinate, do nothing
    let Some((parent_entity, _item, _coord)) = item_query
        .iter()
        .find(|&(_, item, coord)| item.is_bomb() && *coord == trigger.coord)
    else {
        println!("No bomb object found at the coordinate.");
        return;
    };

    if let Ok((fire_entity, &fire_coord)) = fire_query.single() {
        commands.entity(fire_entity).despawn();
        if fire_coord == trigger.coord {
            println!("Fire already exists at this coordinate, skipping creation.");
            return;
        }
    }

    commands
        .entity(parent_entity)
        .with_child(fire(trigger.coord, &item_assets));
}

pub fn fire(coord: GridCoord, item_assets: &ItemAssets) -> impl Bundle {
    (
        Name::new("Fire Object"),
        coord,
        Fire,
        Sprite::from_atlas_image(
            item_assets.sprite_sheet.clone(),
            TextureAtlas {
                layout: item_assets.texture_atlas_layout.clone(),
                index: 5,
            },
        ),
        Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
        StateScoped(Screen::Gameplay),
    )
}

fn reset_all_object_placements(
    button_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    if button_input.just_pressed(KeyCode::KeyR) {
        next_state.set(GamePhase::Init);
    }
}

fn run_simulation(
    button_input: Res<ButtonInput<KeyCode>>,
    fire_query: Query<Entity, With<Fire>>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    if button_input.just_pressed(KeyCode::Space) {
        if fire_query.is_empty() {
            println!("No fire objects found, cannot run simulation.");
            return;
        } else {
            println!("Fire objects found, proceeding with simulation.");
            next_state.set(GamePhase::Run);
        }
    }
}
