use bevy::{color::palettes, ecs::component::Mutable, prelude::*};
use std::time::Duration;

use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<AffectedTileAnimation>().add_systems(
        Update,
        (
            update_animation_timer::<AffectedTileAnimation>.in_set(AppSystems::TickTimers),
            update_animation_color::<AffectedTileAnimation>.in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

/// Update the animation timer.
fn update_animation_timer<D>(time: Res<Time>, mut query: Query<&mut D>)
where
    D: ColorAnimation + Component<Mutability = Mutable>,
{
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_color<D>(mut commands: Commands, mut query: Query<(&D, &mut Sprite, Entity)>)
where
    D: ColorAnimation + Component<Mutability = Mutable>,
{
    for (animation, mut sprite, entity) in &mut query {
        if animation.finished() {
            commands.entity(entity).despawn();
        } else {
            sprite.color = animation.get_color();
        }
    }
}

trait ColorAnimation {
    fn update_timer(&mut self, delta: Duration);
    fn finished(&self) -> bool;
    fn get_color(&self) -> Color;
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct AffectedTileAnimation {
    timer: Timer,
}

impl AffectedTileAnimation {
    const DURATION: Duration = Duration::from_millis(800);

    pub fn new() -> Self {
        Self {
            timer: Timer::new(Self::DURATION, TimerMode::Once),
        }
    }
}

impl ColorAnimation for AffectedTileAnimation {
    fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
    }

    fn finished(&self) -> bool {
        self.timer.finished()
    }

    fn get_color(&self) -> Color {
        // RED -> WHITE -> TRANSPARENT
        // 0.0 -> 0.2 -> 0.8
        let red = palettes::css::RED.with_alpha(0.7);
        let white = palettes::css::WHITE.with_alpha(0.7);
        let transparent = white.with_alpha(0.0);

        let elapsed_secs = self.timer.elapsed_secs();

        if elapsed_secs < 0.2 {
            let ratio = elapsed_secs / 0.2;
            red.mix(&white, ratio).into()
        } else if !self.finished() {
            let ratio = (elapsed_secs - 0.2) / (Self::DURATION.as_secs_f32() - 0.2);
            white.mix(&transparent, ratio).into()
        } else {
            transparent.into()
        }
    }
}
