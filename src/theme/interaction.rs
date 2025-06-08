use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::{SEVolume, SoundEffectAssets, sound_effect},
    theme::widget::{ItemButton, RunButton},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<InteractionPalette>();
    app.add_systems(
        Update,
        (apply_interaction_palette, apply_interaction_image_palette),
    );

    app.add_observer(play_on_hover_sound_effect);
    app.add_observer(play_on_click_sound_effect);
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
    interaction_query: Query<Option<&ItemButton>, With<Interaction>>,
    se_volume: Res<SEVolume>,
) {
    let Some(se_assets) = se_assets else {
        return;
    };

    if let Ok(option) = interaction_query.get(trigger.target()) {
        match option {
            Some(_) => {
                commands.spawn(sound_effect(se_assets.select_4.clone(), &se_volume));
            }
            None => {
                commands.spawn(sound_effect(se_assets.select_3.clone(), &se_volume));
            }
        }
    }
}

fn play_on_click_sound_effect(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    se_assets: Option<Res<SoundEffectAssets>>,
    interaction_query: Query<(Option<&ItemButton>, Option<&RunButton>), With<Interaction>>,
    se_volume: Res<SEVolume>,
) {
    let Some(se_assets) = se_assets else {
        return;
    };

    if let Ok((item, run)) = interaction_query.get(trigger.target()) {
        match (item, run) {
            (Some(_), _) => {
                // Item button clicked
                commands.spawn(sound_effect(se_assets.select_1.clone(), &se_volume));
            }
            (_, Some(_)) => {
                // Run button clicked
                commands.spawn(sound_effect(se_assets.start_1.clone(), &se_volume));
            }
            _ => {
                // Default click sound effect
                commands.spawn(sound_effect(se_assets.select_2.clone(), &se_volume));
            }
        }
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
