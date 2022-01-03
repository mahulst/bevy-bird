pub use bevy::prelude::*;
pub use bevy_rapier3d::prelude::*;
pub use wasm_bindgen::prelude::*;

pub enum Actions {
    Start,
    Exit,
}

pub struct ActionButton(pub Actions);
pub struct MenuItem;
pub struct GameOver;
pub struct GameComponent;

pub struct Player;
pub struct Velocity(Vec2);

#[derive(Debug)]
pub struct CanKillPlayer;
pub struct Obstacle;
pub struct Camera;

pub struct Score(pub u32);
pub struct ScoreTrigger(pub bool);
pub struct ScoreText;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Menu,
    Playing,
    End,
}
impl AppState {
    pub fn new() -> Self {
        AppState::Menu
    }

    pub fn restart() -> Self {
        AppState::Playing
    }

    pub fn game_over() -> Self {
        AppState::End
    }
}
