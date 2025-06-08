use bevy::{audio::Volume, prelude::*};

use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MusicAssets>()
        .load_resource::<MusicAssets>();

    app.register_type::<Music>().register_type::<SoundEffect>();

    app.init_resource::<MusicVolume>()
        .init_resource::<SEVolume>();

    app.add_systems(
        Update,
        apply_volume_setting.run_if(resource_changed::<MusicVolume>),
    )
    .add_systems(Update, (fade_in, fade_out))
    .add_observer(spawn_music);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct MusicAssets {
    #[dependency]
    pub title_bgm: Handle<AudioSource>,
    #[dependency]
    pub level_select_bgm: Handle<AudioSource>,
    #[dependency]
    pub in_game_bgm: Handle<AudioSource>,
}

impl FromWorld for MusicAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource_mut::<AssetServer>();
        let title_bgm = asset_server.load("audio/music/dark_space.ogg");
        let level_select_bgm = asset_server.load("audio/music/dark_space.ogg");
        let in_game_bgm = asset_server.load("audio/music/dark_space.ogg");

        MusicAssets {
            title_bgm,
            level_select_bgm,
            in_game_bgm,
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Music;

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct MusicVolume {
    pub volume: Volume,
}

/// A music audio instance.
pub fn music(handle: Handle<AudioSource>) -> impl Bundle {
    (
        AudioPlayer(handle),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::SILENT, // Start silent, will fade in
            ..Default::default()
        },
        Music,
        FadeIn,
    )
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SoundEffect;

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct SEVolume {
    pub volume: Volume,
}

pub fn sound_effect(handle: Handle<AudioSource>, se_volume: &SEVolume) -> impl Bundle {
    (
        AudioPlayer(handle),
        PlaybackSettings::DESPAWN.with_volume(se_volume.volume),
        SoundEffect,
    )
}

const FADE_TIME: f32 = 2.0;

#[derive(Component)]
struct FadeIn;

#[derive(Component)]
struct FadeOut;

fn fade_in(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity), With<FadeIn>>,
    music_volume: Res<MusicVolume>,
    time: Res<Time>,
) {
    for (mut audio, entity) in audio_sink.iter_mut() {
        let current_volume = audio.volume();
        audio.set_volume(
            current_volume
                + Volume::Linear(music_volume.volume.to_linear() * time.delta_secs() / FADE_TIME),
        );
        if audio.volume().to_linear() >= music_volume.volume.to_linear() {
            audio.set_volume(music_volume.volume);
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}

fn fade_out(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity), With<FadeOut>>,
    time: Res<Time>,
    music_volume: Res<MusicVolume>,
) {
    for (mut audio, entity) in audio_sink.iter_mut() {
        let current_volume = audio.volume();
        audio.set_volume(
            current_volume
                - Volume::Linear(music_volume.volume.to_linear() * time.delta_secs() / FADE_TIME),
        );
        if audio.volume().to_linear() <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Event, Debug)]
pub struct SpawnMusic {
    handle: Handle<AudioSource>,
}

impl SpawnMusic {
    pub fn new(handle: Handle<AudioSource>) -> Self {
        Self { handle }
    }
}

fn spawn_music(
    trigger: Trigger<SpawnMusic>,
    mut commands: Commands,
    soundtrack: Query<Entity, (With<AudioSink>, With<Music>)>,
) {
    for track in soundtrack.iter() {
        commands.entity(track).insert(FadeOut);
    }
    println!("Changing track");
    commands.spawn(music(Handle::clone(&trigger.handle)));
}

fn apply_volume_setting(
    music_volume: Res<MusicVolume>,
    audio_query: Query<&mut AudioSink, With<Music>>,
) {
    for mut sink in audio_query {
        sink.set_volume(music_volume.volume);
    }
}
