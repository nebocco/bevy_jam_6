//! Helper functions for creating common widgets.

use std::borrow::Cow;

use bevy::{
    color::palettes,
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
    ui::Val::*,
};

use crate::{
    demo::level,
    screens::LevelStatus,
    theme::{
        UiAssets,
        interaction::{InteractionImagePalette, InteractionPalette},
        palette::*,
    },
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
pub fn header(text: impl Into<String>) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        TextFont::from_font_size(40.0),
        TextColor(HEADER_TEXT),
    )
}

/// A simple text label.
pub fn label(text: impl Into<String>) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        TextFont::from_font_size(24.0),
        TextColor(LABEL_TEXT),
    )
}

pub fn item_button<E, B, M, I>(
    image_handle: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    index: usize,
    action: I,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    BackgroundColor(BUTTON_BACKGROUND),
                    InteractionPalette {
                        none: BUTTON_BACKGROUND,
                        hovered: BUTTON_HOVERED_BACKGROUND,
                        pressed: BUTTON_PRESSED_BACKGROUND,
                    },
                    Node {
                        width: Px(80.0),
                        height: Px(80.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![(
                        Name::new("Button Image"),
                        ImageNode::from_atlas_image(image_handle, TextureAtlas { layout, index },),
                        Transform::from_scale(Vec2::splat(2.0).extend(1.0))
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
                .with_color(if level_status.is_locked {
                    Color::Srgba(palettes::css::GRAY.with_alpha(0.5))
                } else {
                    Color::Srgba(palettes::css::WHITE)
                }),
                children![(
                    Name::new("Button Text"),
                    Text(text),
                    TextFont::from_font_size(40.0),
                    TextColor(BUTTON_TEXT),
                    // Don't bubble picking events from the text up to the button.
                    Pickable::IGNORE,
                )],
            ));

            if !level_status.is_locked {
                entity_bundle.insert(InteractionImagePalette {
                    none: Color::Srgba(palettes::css::WHITE),
                    hovered: Color::Srgba(palettes::css::LIGHT_BLUE),
                    pressed: Color::Srgba(palettes::css::LIGHT_BLUE.with_alpha(0.5)),
                });
            }
        })),
    )
}

/// A large rounded button with text and an action defined as an [`Observer`].
pub fn button<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        action,
        (
            Node {
                width: Px(380.0),
                height: Px(80.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderRadius::MAX,
        ),
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub fn button_small<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        action,
        Node {
            width: Px(30.0),
            height: Px(30.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )
}

/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_bundle`.
fn button_base<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    button_bundle: impl Bundle,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    BackgroundColor(BUTTON_BACKGROUND),
                    InteractionPalette {
                        none: BUTTON_BACKGROUND,
                        hovered: BUTTON_HOVERED_BACKGROUND,
                        pressed: BUTTON_PRESSED_BACKGROUND,
                    },
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextFont::from_font_size(40.0),
                        TextColor(BUTTON_TEXT),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .insert(button_bundle)
                .observe(action);
        })),
    )
}
