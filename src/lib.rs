#![warn(clippy::all, clippy::pedantic)]

use wasm_bindgen::prelude::wasm_bindgen;
use crate::obstacles::{clean, spawn, stop};
use crate::player::{
    clamp_player_y, clean_up_previous_game, count_score, fly, player_death, rotate_player_body,
    setup_camera, setup_game,
};
use crate::prelude::*;
use crate::ui::{close_menu, display_game_over, menu, setup_menu, update_score};

pub mod obstacles;
pub mod player;
pub mod prelude;
pub mod ui;

const GRAVITY: f32 = 12.8;

#[wasm_bindgen]
pub fn run() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);


    app.insert_resource(Score(0))
        .insert_resource(WindowDescriptor {
            vsync: false,
            ..Default::default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(RapierConfiguration {
            gravity: -Vector::y() * GRAVITY,
            ..Default::default()
        })
        .add_plugin(RapierRenderPlugin)
        .add_state_to_stage(CoreStage::Update, AppState::restart())
        .add_state_to_stage(CoreStage::PostUpdate, AppState::restart())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(menu.system())
        .add_startup_system(setup_camera.system())
        .add_system_set(
            SystemSet::on_enter(AppState::End)
                .with_system(setup_menu.system())
                .with_system(stop.system())
                .with_system(display_game_over.system()),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::End).with_system(clean_up_previous_game.system()),
        )
        .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(setup_menu.system()))
        .add_system_set(
            SystemSet::on_enter(AppState::Playing)
                .with_system(setup_game.system())
                .with_system(close_menu.system()),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::on_update(AppState::Playing).with_system(spawn.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Playing)
                .with_system(clean.system())
                .with_system(count_score.system())
                .with_system(update_score.system())
                .with_system(clamp_player_y.system())
                .with_system(rotate_player_body.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Playing)
                .with_system(player_death.system())
                .with_system(fly.system()),
        );

    app.run();
}
