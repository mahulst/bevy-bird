pub use bevy::prelude::*;
pub use bevy_rapier3d::prelude::*;

pub enum Actions {
    Start,
    Exit,
}

#[derive(Component)]
pub struct ActionButton(pub Actions);
#[derive(Component)]
pub struct MenuItem;
#[derive(Component)]
pub struct GameOver;
#[derive(Component)]
pub struct GameComponent;

#[derive(Component)]
pub struct Player;
#[derive(Component)]
pub struct Velocity(Vec2);

#[derive(Debug, Component)]
pub struct CanKillPlayer;
#[derive(Component)]
pub struct Obstacle;
#[derive(Component)]
pub struct Camera;

#[derive(Component)]
pub struct Score(pub u32);
#[derive(Component)]
pub struct ScoreTrigger(pub bool);
#[derive(Component)]
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
