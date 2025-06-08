use bevy::{audio::Volume, ecs::resource, prelude::*};

use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MusicAssets>()
        .load_resource::<MusicAssets>();
    app.register_type::<SoundEffectAssets>()
        .load_resource::<SoundEffectAssets>();

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
        let title_bgm = asset_server.load("audio/music/expansion.ogg");
        let level_select_bgm = asset_server.load("audio/music/industrial.ogg");
        let in_game_bgm = asset_server.load("audio/music/dark_space.ogg");

        MusicAssets {
            title_bgm,
            level_select_bgm,
            in_game_bgm,
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct SoundEffectAssets {
    pub explosion_1: Handle<AudioSource>,
    pub explosion_2: Handle<AudioSource>,
    pub explosion_3: Handle<AudioSource>,
    pub break_1: Handle<AudioSource>,
    pub break_2: Handle<AudioSource>,
    pub select_1: Handle<AudioSource>,
    pub select_2: Handle<AudioSource>,
    pub select_3: Handle<AudioSource>,
    pub select_4: Handle<AudioSource>,

    pub start_1: Handle<AudioSource>,
    pub clear: Handle<AudioSource>,
    pub failed: Handle<AudioSource>,
}

impl FromWorld for SoundEffectAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource_mut::<AssetServer>();
        let explosion_1 = asset_server.load("audio/sound_effects/explosion_1.wav");
        let explosion_2 = asset_server.load("audio/sound_effects/explosion_2.wav");
        let explosion_3 = asset_server.load("audio/sound_effects/explosion_3.wav");
        let break_1 = asset_server.load("audio/sound_effects/break_1.wav");
        let break_2 = asset_server.load("audio/sound_effects/break_2.wav");
        let select_1 = asset_server.load("audio/sound_effects/select_1.wav");
        let select_2 = asset_server.load("audio/sound_effects/select_2.wav");
        let select_3 = asset_server.load("audio/sound_effects/select_3.wav");
        let select_4 = asset_server.load("audio/sound_effects/select_4.wav");
        let start_1 = asset_server.load("audio/sound_effects/start_1.wav");
        let clear = asset_server.load("audio/sound_effects/clear.wav");
        let failed = asset_server.load("audio/sound_effects/failed.wav");

        SoundEffectAssets {
            explosion_1,
            explosion_2,
            explosion_3,
            break_1,
            break_2,
            select_1,
            select_2,
            select_3,
            select_4,
            start_1,
            clear,
            failed,
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
fn music(handle: Handle<AudioSource>) -> impl Bundle {
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

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct SEVolume {
    pub volume: Volume,
}

impl Default for SEVolume {
    fn default() -> Self {
        Self {
            volume: Volume::Linear(0.7), // Default sound effect volume
        }
    }
}

pub fn sound_effect(handle: Handle<AudioSource>, se_volume: &SEVolume) -> impl Bundle {
    println!("Spawning sound effect: {:?}", handle);
    (
        AudioPlayer(handle),
        PlaybackSettings::DESPAWN.with_volume(se_volume.volume),
        SoundEffect,
    )
}

const FADE_OUT_TIME: f32 = 0.5;
const FADE_IN_TIME: f32 = 2.0;

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
                + Volume::Linear(
                    music_volume.volume.to_linear() * time.delta_secs() / FADE_IN_TIME,
                ),
        );
        if audio.volume().to_linear() >= music_volume.volume.to_linear() {
            println!("Audio faded in: {:?}", entity);
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
                - Volume::Linear(
                    music_volume.volume.to_linear() * time.delta_secs() / FADE_OUT_TIME,
                ),
        );
        if audio.volume().to_linear() <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn stop_music(mut commands: Commands, audio_sink: Query<Entity, With<Music>>) {
    for entity in audio_sink.iter() {
        commands.entity(entity).remove::<FadeIn>().insert(FadeOut);
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
        commands.entity(track).remove::<FadeIn>().insert(FadeOut);
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
