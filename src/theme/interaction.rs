use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::{SEVolume, SoundEffectAssets, sound_effect},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<InteractionPalette>();
    app.add_systems(
        Update,
        (apply_interaction_palette, apply_interaction_image_palette),
    );
}

/// Palette for widget interactions. Add this to an entity that supports
/// [`Interaction`]s, such as a button, to change its [`BackgroundColor`] based
/// on the current interaction state.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InteractionPalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
}

fn apply_interaction_palette(
    mut palette_query: Query<
        (&Interaction, &InteractionPalette, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (interaction, palette, mut background) in &mut palette_query {
        *background = match interaction {
            Interaction::None => palette.none,
            Interaction::Hovered => palette.hovered,
            Interaction::Pressed => palette.pressed,
        }
        .into();
    }
}

fn play_on_hover_sound_effect(
    trigger: Trigger<Pointer<Over>>,
    mut commands: Commands,
    se_assets: Option<Res<SoundEffectAssets>>,
    interaction_query: Query<(), With<Interaction>>,
    se_volume: Res<SEVolume>,
) {
    let Some(se_assets) = se_assets else {
        return;
    };

    if interaction_query.contains(trigger.target()) {
        commands.spawn(sound_effect(se_assets.select_3.clone(), &se_volume));
    }
}

fn play_on_click_sound_effect(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    se_assets: Option<Res<SoundEffectAssets>>,
    interaction_query: Query<(), With<Interaction>>,
    se_volume: Res<SEVolume>,
) {
    let Some(se_assets) = se_assets else {
        return;
    };

    if interaction_query.contains(trigger.target()) {
        commands.spawn(sound_effect(se_assets.select_2.clone(), &se_volume));
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InteractionImagePalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
}

fn apply_interaction_image_palette(
    mut palette_query: Query<
        (&Interaction, &InteractionImagePalette, &mut ImageNode),
        Changed<Interaction>,
    >,
) {
    for (interaction, palette, mut image_node) in &mut palette_query {
        image_node.color = match interaction {
            Interaction::None => palette.none,
            Interaction::Hovered => palette.hovered,
            Interaction::Pressed => palette.pressed,
        };
    }
}
