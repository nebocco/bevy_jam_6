use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    AppSystems, PausableSystems, asset_tracking::LoadResource, menus::Menu, screens::Screen,
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    // app.register_type::<Item>();

    // app.register_type::<ItemAssets>();
    app.load_resource::<ItemAssets>();
    app.init_resource::<SelectedItem>();
    app.add_systems(OnEnter(Screen::Gameplay), spawn_item_buttons);
}

fn spawn_item_buttons(
    mut commands: Commands,
    item_assets: Res<ItemAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 8, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands
        .spawn((
            widget::ui_root("Item Buttons"),
            GlobalZIndex(2),
            StateScoped(Menu::None),
            children![
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    Handle::clone(&texture_atlas_layout),
                    0,
                    select_item::<0>
                ),
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    Handle::clone(&texture_atlas_layout),
                    1,
                    select_item::<1>
                ),
                widget::item_button(
                    Handle::clone(&item_assets.sprite_sheet),
                    Handle::clone(&texture_atlas_layout),
                    2,
                    select_item::<2>
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
            row_gap: Val::Px(32.0),
            left: Val::Percent(60.0),
            ..Default::default()
        });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Item {
    Item1,
    Item2,
    Item3,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
struct SelectedItem(Option<Item>);

impl From<u8> for Item {
    fn from(value: u8) -> Self {
        match value {
            0 => Item::Item1,
            1 => Item::Item2,
            2 => Item::Item3,
            _ => panic!("Invalid item index"),
        }
    }
}

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

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ItemAssets {
    #[dependency]
    sprite_sheet: Handle<Image>,
    // #[dependency]
    // pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for ItemAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            sprite_sheet: assets.load_with_settings(
                "images/item_sprite_sheet.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            // steps: vec![
            //     assets.load("audio/sound_effects/step1.ogg"),
            //     assets.load("audio/sound_effects/step2.ogg"),
            //     assets.load("audio/sound_effects/step3.ogg"),
            //     assets.load("audio/sound_effects/step4.ogg"),
            // ],
        }
    }
}
