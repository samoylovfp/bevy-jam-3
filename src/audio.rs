use bevy::prelude::{
    info, warn, AssetServer, EventWriter, Handle, IntoSystemAppConfig, IntoSystemConfig, OnExit,
    OnUpdate, Plugin, Res, ResMut, Resource,
};
use bevy_kira_audio::{AudioApp, AudioChannel, AudioControl, AudioSource};

use crate::game::GameState;
use crate::hud::SubtitleTrigger;
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
            .add_system(dialogue.in_set(OnUpdate(AppState::InGame)))
            .add_system(stop_all_dialogue.in_schedule(OnExit(AppState::InGame)))
            .add_startup_system(start_music)
            .insert_resource(DialoguePlaying::None);
        // .add_system(adjust_volume_on_distance);
    }
}

// enter the test chamber
static FILES: &[&str] = &[
    "01-doc1.ogg",
    "02-bvj.ogg",
    "03-doc1.ogg",
    "04-doc1.ogg",
    "05-bvj.ogg",
    "06-doc2.ogg",
    "07-doc1.ogg",
    "08-bvj.ogg",
    "09-doc1.ogg",
    "10-doc2.ogg",
    "11-doc1.ogg",
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
    for f in FILES {
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

static SPEED: f64 = 5.0;

static SUBTITLES: &str = include_str!("../assets/text/subtitles.txt");

fn dialogue(
    asset_server: Res<AssetServer>,
    mut playing: ResMut<DialoguePlaying>,
    audio_channel: Res<AudioChannel<SpawnRoomSpeaker>>,
    mut events: EventWriter<SubtitleTrigger>,
    mut game_state: ResMut<GameState>,
) {
    audio_channel.set_playback_rate(SPEED);
    let play_dialogue_file = |n: usize| String::from("sounds/dialogues/") + FILES[n];
    let play_first_phase_dialog =
        |n: usize| audio_channel.play(asset_server.load(play_dialogue_file(n)));
    let phase_end = match *game_state {
        GameState::JustSpawned => 2,
        GameState::InTestingRoom | GameState::TurnOnLaser1 => 3,
        _ => {
            warn!("TODO");
            1
        }
    };

    let new_state = match *playing {
        DialoguePlaying::None => {
            info!("Started");
            play_first_phase_dialog(0);
            DialoguePlaying::StartedButNotPlaying(0)
        }
        DialoguePlaying::StartedButNotPlaying(n) => {
            let subtit = SUBTITLES.lines().nth(n).unwrap();
            events.send(SubtitleTrigger(subtit.to_string()));
            if audio_channel.is_playing_sound() {
                DialoguePlaying::Playing(n)
            } else {
                DialoguePlaying::StartedButNotPlaying(n)
            }
        }
        DialoguePlaying::Playing(n) => {
            if !audio_channel.is_playing_sound() {
                if n == 3 && matches!(*game_state, GameState::InTestingRoom) {
                    info!("TUrning on the laser!!");
                    *game_state = GameState::TurnOnLaser1;
                }
            }
            if audio_channel.is_playing_sound() {
                DialoguePlaying::Playing(n)
            } else if n < phase_end {
                info!("Continuing to {n}+1");
                play_first_phase_dialog(n + 1);
                DialoguePlaying::StartedButNotPlaying(n + 1)
            } else {
                events.send(SubtitleTrigger(String::new()));
                DialoguePlaying::Playing(n)
            }
        }
    };
    *playing = new_state;
}

fn stop_all_dialogue(audio: Res<AudioChannel<SpawnRoomSpeaker>>) {
    audio.stop();
}
