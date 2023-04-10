use bevy::prelude::{
    info, AssetServer, Handle, IntoSystemAppConfig, IntoSystemConfig, OnExit, OnUpdate, Plugin,
    Res, ResMut, Resource,
};
use bevy_kira_audio::{AudioApp, AudioChannel, AudioControl, AudioSource};

use crate::AppState;

pub(crate) struct AudioPlugin;

#[derive(Resource)]
struct BgMusic;

#[derive(Resource)]
struct SpawnRoomSpeaker;

#[derive(Resource)]
struct ProtagonistVoice;

#[derive(Resource)]
struct FirstRoomSpeaker;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(bevy_kira_audio::AudioPlugin)
            .add_audio_channel::<BgMusic>()
            .add_audio_channel::<SpawnRoomSpeaker>()
            .add_system(first_dialogue.in_set(OnUpdate(AppState::InGame)))
            .add_system(stop_all_dialogue.in_schedule(OnExit(AppState::InGame)))
            .add_startup_system(start_music)
            .insert_resource(DialoguePlaying::None);
        // .add_system(adjust_volume_on_distance);
    }
}

// spawn
static FIRST_PHASE: &[&str] = &["01-doc1.ogg", "02-bvj.ogg", "03-doc1.ogg"];
// static FIRST_PHASE: &[&str] = &["01-doc1.ogg"];

// enter the test chamber
static SECOND_PHASE: &[&str] = &[
    "04-doc1.ogg",
    "05-bvj.ogg",
    "06-doc2.ogg",
    "07-doc1.ogg",
    "08-bvj.ogg",
    "09-doc1.ogg",
    "10-doc2.ogg",
    "11-doc1.ogg",
];

static REMAINING_FILES: &[&str] = &[
    "12-bvj.ogg",
    "13-doc1.ogg",
    "14-doc2.ogg",
    "15-doc1.ogg",
    "16-bvj.ogg",
    "17-doc2.ogg",
    "18-doc2.ogg",
    "19-doc1.ogg",
];

fn start_music(asset_server: Res<AssetServer>, audio: Res<AudioChannel<BgMusic>>) {
    for f in FIRST_PHASE.iter().chain(SECOND_PHASE) {
        let _h: Handle<AudioSource> = asset_server.load(*f);
    }

    audio
        .play(asset_server.load("sounds/bvj-3-space-lab.ogg"))
        .looped();
    audio.set_volume(0.2);
}

#[derive(Resource)]
enum DialoguePlaying {
    None,
    StartedButNotPlaying(usize),
    Playing(usize),
}

fn first_dialogue(
    asset_server: Res<AssetServer>,
    mut playing: ResMut<DialoguePlaying>,
    audio_channel: Res<AudioChannel<SpawnRoomSpeaker>>,
) {
    let first_phase_dialogue_file = |n: usize| String::from("sounds/dialogues/") + FIRST_PHASE[n];
    let play_first_phase_dialog =
        |n: usize| audio_channel.play(asset_server.load(first_phase_dialogue_file(n)));

    // info!("playing: {}", audio_channel.is_playing_sound());

    let new_state = match *playing {
        DialoguePlaying::None => {
            info!("Started");
            play_first_phase_dialog(0);
            DialoguePlaying::StartedButNotPlaying(0)
        }
        DialoguePlaying::StartedButNotPlaying(n) => {
            if audio_channel.is_playing_sound() {
                DialoguePlaying::Playing(n)
            } else {
                DialoguePlaying::StartedButNotPlaying(n)
            }
        }
        DialoguePlaying::Playing(n) => {
            if audio_channel.is_playing_sound() {
                DialoguePlaying::Playing(n)
            } else if n < FIRST_PHASE.len() - 1 {
                info!("Continuing to {n}+1");
                play_first_phase_dialog(n + 1);
                DialoguePlaying::StartedButNotPlaying(n + 1)
            } else {
                DialoguePlaying::Playing(n)
            }
        }
    };
    *playing = new_state;
}

fn stop_all_dialogue(audio: Res<AudioChannel<SpawnRoomSpeaker>>) {
    audio.stop();
}

// fn adjust_positional_audio(
//     start: Query<&PlayerSpawn>,
//     body: Query<
//     audio: Res<AudioChannel<SpawnRoomSpeaker>>,
// ) {

// }
