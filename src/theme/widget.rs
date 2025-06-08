//! Helper functions for creating common widgets.

use std::borrow::Cow;

use bevy::{
    color::palettes,
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
    ui::Val::*,
};

use crate::{
    gameplay::Item,
    screens::LevelStatus,
    theme::{UiAssets, interaction::InteractionImagePalette, palette::*},
};

/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: Percent(100.0),
            height: Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Px(20.0),
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

/// A simple header label. Bigger than [`label`].
pub fn header(text: impl Into<String>, font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        TextFont::from_font(font).with_font_size(48.0),
        TextColor(HEADER_TEXT),
        Pickable::IGNORE,
    )
}

/// A simple text label.
pub fn label(text: impl Into<String>, font: Option<Handle<Font>>) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        if let Some(font) = font {
            TextFont::from_font(font).with_font_size(32.0)
        } else {
            TextFont::from_font_size(32.0)
        },
        TextColor(LABEL_TEXT),
    )
}

/// A simple text.
pub fn text(text: impl Into<String>, font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        TextFont::from_font(font).with_font_size(32.0),
        TextColor(TEXT),
        Pickable::IGNORE,
    )
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct ItemButton;

pub fn item_button<E, B, M, I>(
    image_handle: Handle<Image>,
    ui_assets: &UiAssets,
    layout: Handle<TextureAtlasLayout>,
    item: Item,
    action: I,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let action = IntoObserverSystem::into_system(action);
    let texture_handle = Handle::clone(&ui_assets.ui_texture);
    let button_texture_layout = Handle::clone(&ui_assets.texture_atlas_layout);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    ItemButton,
                    item,
                    Node {
                        width: Px(80.0),
                        height: Px(80.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ImageNode::from_atlas_image(
                        texture_handle,
                        TextureAtlas {
                            layout: button_texture_layout,
                            index: 1,
                        },
                    )
                    .with_mode(NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect::all(8.0),
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 2.0,
                    })),
                    InteractionImagePalette {
                        none: Color::Srgba(palettes::css::WHITE),
                        hovered: Color::Srgba(palettes::css::THISTLE),
                        pressed: Color::Srgba(palettes::css::PLUM.with_alpha(0.5)),
                    },
                    children![(
                        Name::new("Button Image"),
                        ImageNode::from_atlas_image(
                            image_handle,
                            TextureAtlas {
                                layout,
                                index: item.to_sprite_index(),
                            },
                        ),
                        Transform::from_xyz(0.0, 0.0, 0.1).with_scale(Vec2::splat(2.0).extend(1.0)),
                        Pickable::IGNORE,
                    )],
                ))
                .observe(action);
        })),
    )
}

pub fn level_button(index: usize, ui_assets: &UiAssets, level_status: LevelStatus) -> impl Bundle {
    let text = index.to_string();
    let texture_handle = Handle::clone(&ui_assets.ui_texture);
    let layout = Handle::clone(&ui_assets.texture_atlas_layout);
    let font_handle = Handle::clone(&ui_assets.font);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            let mut entity_bundle = parent.spawn((
                Name::new("Button Inner"),
                Button,
                Node {
                    width: Px(96.0),
                    height: Px(96.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ImageNode::from_atlas_image(
                    texture_handle,
                    TextureAtlas {
                        layout,
                        index: if level_status.is_cleared { 0 } else { 1 },
                    },
                )
                .with_mode(NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(8.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 4.0,
                }))
                .with_color(if level_status.is_locked {
                    Color::Srgba(palettes::css::GRAY.with_alpha(0.5))
                } else {
                    Color::Srgba(palettes::css::WHITE)
                }),
                children![(
                    Name::new("Button Text"),
                    Text(text),
                    // TextFont::from_font_size(40.0),
                    TextFont::from_font(font_handle).with_font_size(48.0),
                    TextColor(if level_status.is_locked {
                        BUTTON_TEXT_DISABLED
                    } else {
                        BUTTON_TEXT
                    }),
                    // Don't bubble picking events from the text up to the button.
                    Pickable::IGNORE,
                )],
            ));

            if !level_status.is_locked {
                entity_bundle.insert(InteractionImagePalette {
                    none: Color::Srgba(palettes::css::WHITE),
                    hovered: Color::Srgba(palettes::css::THISTLE),
                    pressed: Color::Srgba(palettes::css::PLUM.with_alpha(0.5)),
                });
            }
        })),
    )
}

/// A large rounded button with text and an action defined as an [`Observer`].
pub fn text_button<E, B, M, I>(
    text: impl Into<String>,
    ui_assets: &UiAssets,
    action: I,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let texture_handle = Handle::clone(&ui_assets.ui_texture);
    let layout = Handle::clone(&ui_assets.texture_atlas_layout);
    let action = IntoObserverSystem::into_system(action);
    let font_handle = Handle::clone(&ui_assets.font);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    Node {
                        width: Px(380.0),
                        height: Px(90.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::top(Val::Px(6.0)),
                        ..default()
                    },
                    ImageNode::from_atlas_image(texture_handle, TextureAtlas { layout, index: 2 })
                        .with_mode(NodeImageMode::Sliced(TextureSlicer {
                            border: BorderRect::all(12.0),
                            center_scale_mode: SliceScaleMode::Stretch,
                            sides_scale_mode: SliceScaleMode::Stretch,
                            max_corner_scale: 4.0,
                        })),
                    InteractionImagePalette {
                        none: Color::Srgba(palettes::css::WHITE),
                        hovered: Color::Srgba(palettes::css::THISTLE),
                        pressed: Color::Srgba(palettes::css::PLUM.with_alpha(0.5)),
                    },
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextFont::from_font(font_handle).with_font_size(48.0),
                        TextColor(BUTTON_TEXT),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .observe(action);
        })),
    )
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct RunButton;

pub fn run_button<E, B, M, I>(ui_assets: &UiAssets, action: I) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let texture_handle = Handle::clone(&ui_assets.ui_texture);
    let layout = Handle::clone(&ui_assets.texture_atlas_layout);
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    RunButton,
                    Node {
                        width: Px(96.0),
                        height: Px(96.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ImageNode::from_atlas_image(
                        Handle::clone(&texture_handle),
                        TextureAtlas {
                            layout: Handle::clone(&layout),
                            index: 0,
                        },
                    )
                    .with_mode(NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect::all(8.0),
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 4.0,
                    })),
                    InteractionImagePalette {
                        none: Color::Srgba(palettes::css::WHITE),
                        hovered: Color::Srgba(palettes::css::THISTLE),
                        pressed: Color::Srgba(palettes::css::PLUM.with_alpha(0.5)),
                    },
                    children![(
                        Name::new("Button Image"),
                        ImageNode::from_atlas_image(
                            texture_handle,
                            TextureAtlas { layout, index: 4 },
                        ),
                        Transform::from_xyz(0.0, 0.0, 0.1).with_scale(Vec2::splat(2.0).extend(1.0)),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .observe(action);
        })),
    )
}

pub fn menu_button(ui_assets: &UiAssets) -> impl Bundle {
    let texture_handle = Handle::clone(&ui_assets.ui_texture);
    let layout = Handle::clone(&ui_assets.texture_atlas_layout);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn((
                Name::new("Button Inner"),
                Button,
                Node {
                    width: Px(96.0),
                    height: Px(96.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ImageNode::from_atlas_image(
                    Handle::clone(&texture_handle),
                    TextureAtlas {
                        layout: Handle::clone(&layout),
                        index: 0,
                    },
                )
                .with_mode(NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(8.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 4.0,
                })),
                children![(
                    Name::new("Button Image"),
                    ImageNode::from_atlas_image(texture_handle, TextureAtlas { layout, index: 5 },),
                    Transform::from_xyz(0.0, 0.0, 0.1).with_scale(Vec2::splat(2.0).extend(1.0)),
                    // Don't bubble picking events from the text up to the button.
                    Pickable::IGNORE,
                )],
            ));
        })),
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub fn button_small<E, B, M, I>(
    text: impl Into<String>,
    ui_assets: &UiAssets,
    action: I,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let texture_handle = Handle::clone(&ui_assets.ui_texture);
    let layout = Handle::clone(&ui_assets.texture_atlas_layout);
    let action = IntoObserverSystem::into_system(action);
    let font_handle = Handle::clone(&ui_assets.font);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    Node {
                        width: Px(32.0),
                        height: Px(32.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::top(Val::Px(2.0)),
                        ..default()
                    },
                    ImageNode::from_atlas_image(texture_handle, TextureAtlas { layout, index: 1 })
                        .with_mode(NodeImageMode::Sliced(TextureSlicer {
                            border: BorderRect::all(12.0),
                            center_scale_mode: SliceScaleMode::Stretch,
                            sides_scale_mode: SliceScaleMode::Stretch,
                            max_corner_scale: 4.0,
                        })),
                    InteractionImagePalette {
                        none: Color::Srgba(palettes::css::WHITE),
                        hovered: Color::Srgba(palettes::css::THISTLE),
                        pressed: Color::Srgba(palettes::css::PLUM.with_alpha(0.5)),
                    },
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        // TextFont::from_font_size(40.0),
                        TextFont::from_font(font_handle).with_font_size(24.0),
                        TextColor(BUTTON_TEXT),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .observe(action);
        })),
    )
}
