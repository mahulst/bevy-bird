#![warn(clippy::all, clippy::pedantic)]

use bevy::app::AppExit;
use bevy::prelude::*;

enum Actions {
    Start,
    Exit,
}

struct ActionButton(Actions);
struct MenuItem;
struct GameOver;
struct GameComponent;
struct Camera;

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Menu,
    Playing,
    End,
}

impl AppState {
    fn new() -> Self {
        AppState::Menu
    }

    fn restart() -> Self {
        AppState::Playing
    }

    fn game_over() -> Self {
        AppState::End
    }
}

struct Player;
struct Velocity(Vec2);

const GRAVITY: f32 = 3.1;
const FLY_SPEED: f32 = 1.;
const MAX_FLY_SPEED: f32 = 2.;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            vsync: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .init_resource::<ButtonMaterials>()
        .add_state(AppState::restart())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(menu.system())
        .add_startup_system(setup_camera.system())
        .add_system_set(
            SystemSet::on_enter(AppState::End)
                .with_system(setup_menu.system())
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
        .add_system_set(
            SystemSet::on_update(AppState::Playing)
                .with_system(move_player.system())
                .with_system(player_death.system())
                .with_system(gravity.system())
                .with_system(fly.system()),
        )
        .run();
}
fn player_death(mut state: ResMut<State<AppState>>, query: Query<(&Transform), (With<Player>)>) {
    if let Ok((tf)) = query.single() {
        if tf.translation.y < 0.5 {
            state.set(AppState::game_over()).unwrap();
        }
    }
}
fn fly(input: Res<Input<KeyCode>>, mut query: Query<(&mut Velocity), (With<Player>)>) {
    if input.pressed(KeyCode::Space) {
        for (mut velocity) in query.iter_mut() {
            velocity.0.y = MAX_FLY_SPEED.min(velocity.0.y + FLY_SPEED);
        }
    }
}

fn gravity(time: Res<Time>, mut query: Query<(&mut Velocity), (With<Player>)>) {
    let time_delta = time.delta_seconds();
    for (mut velocity) in query.iter_mut() {
        velocity.0.y -= time_delta * GRAVITY;
    }
}

fn move_player(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity), (With<Player>)>) {
    let time_delta = time.delta_seconds();
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += time_delta * velocity.0.x;
        transform.translation.y += time_delta * velocity.0.y;
    }
}

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(0.8, 0., 0.).into()),
            transform: Transform::from_xyz(-2.0, 5., 0.0),
            ..Default::default()
        })
        .insert(Player)
        .insert(GameComponent)
        .insert(Velocity(Vec2::new(0., 0.)));

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1. })),
            material: materials.add(Color::rgb(0., 0.8, 0.).into()),
            transform: Transform {
                translation: Vec3::new(0., 0., 0.),
                scale: Vec3::new(10., 0., 10.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(GameComponent);

    commands
        .spawn_bundle(LightBundle {
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..Default::default()
        })
        .insert(GameComponent);
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0., 5., 10.).looking_at(Vec3::new(0., 4., 2.), Vec3::Y),
            ..Default::default()
        })
        .insert(Camera);
}

fn close_menu(
    mut commands: Commands,
    menu_query: Query<(Entity), (Or<(With<MenuItem>, With<GameOver>)>)>,
) {
    for (entity) in menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn clean_up_previous_game(
    mut commands: Commands,
    menu_query: Query<(Entity), (With<GameComponent>)>,
) {
    for (entity) in menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn menu(
    button_materials: Res<ButtonMaterials>,
    mut state: ResMut<State<AppState>>,
    mut query: Query<
        (
            &Interaction,
            &mut Handle<ColorMaterial>,
            &Children,
            &ActionButton,
        ),
        (Changed<Interaction>, With<ActionButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (interaction, mut material, children, action) in query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).expect("Get button text");
        match *interaction {
            Interaction::Clicked => match *action {
                ActionButton(Actions::Start) => {
                    state.set(AppState::restart()).unwrap();
                }
                ActionButton(Actions::Exit) => {
                    app_exit_events.send(AppExit);
                }
            },
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

fn display_game_over(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(TextBundle {
            text: Text::with_section(
                "Game Over",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 80.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..Default::default()
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..Default::default()
                },
            ),
            style: Style {
                align_self: AlignSelf::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(GameOver);
}

fn setup_menu(
    mut commands: Commands,
    button_materials: Res<ButtonMaterials>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.), Val::Px(65.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Play",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .insert(ActionButton(Actions::Start))
        .insert(MenuItem);

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.), Val::Px(65.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Exit",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .insert(ActionButton(Actions::Exit))
        .insert(MenuItem);
}
