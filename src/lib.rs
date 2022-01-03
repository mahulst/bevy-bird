#![warn(clippy::all, clippy::pedantic)]
use wasm_bindgen::prelude::*;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_rapier3d::na::{Quaternion, UnitQuaternion};
use bevy_rapier3d::physics;
use bevy_rapier3d::physics::PhysicsStages;
use bevy_rapier3d::prelude::*;
use rand::{thread_rng, Rng};

enum Actions {
    Start,
    Exit,
}
struct ActionButton(Actions);
struct MenuItem;
struct GameOver;
struct GameComponent;

#[derive(Debug)]
struct CanKillPlayer;
struct Obstacle;
struct Camera;

struct Score(u32);
struct ScoreTrigger(bool);
struct ScoreText;

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

const GRAVITY: f32 = 12.8;
const FLY_SPEED: f32 = 8.;
const OBSTACLE_MOVEMENT_SPEED: f32 = -4.;
const OBSTACLE_DISTANCE: f32 = 4.;
const OBSTACLE_SPAWN_X: f32 = 8.;
const MAX_FLY_SPEED: f32 = 3.8;

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();

    app.add_plugins(DefaultPlugins);

    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
        app.add_plugin(bevy_webgl2::WebGL2Plugin);


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
        .init_resource::<ButtonMaterials>()
        .add_plugin(RapierRenderPlugin)
        .add_state_to_stage(CoreStage::Update, AppState::restart())
        .add_state_to_stage(CoreStage::PostUpdate, AppState::restart())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(menu.system())
        .add_startup_system(setup_camera.system())
        .add_system_set(
            SystemSet::on_enter(AppState::End)
                .with_system(setup_menu.system())
                .with_system(stop_obstacles.system())
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
            SystemSet::on_update(AppState::Playing).with_system(spawn_obstacles.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Playing)
                .with_system(clean_obstacles.system())
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



fn pause_physics(mut conf: ResMut<RapierConfiguration>) {
    conf.physics_pipeline_active = false;
}

fn rotate_player_body(
    mut query: Query<(&RigidBodyVelocity, &mut RigidBodyPosition), (With<Player>)>,
) {
    for (vel, mut pos) in query.iter_mut() {
        let rotation_percentage = lerp(-3., 3., vel.linvel.y) * 2.0 - 1.;

        pos.position.rotation = Quat::from_rotation_z(rotation_percentage).into();
    }
}

fn clamp_player_y(mut query: Query<(&mut RigidBodyVelocity, &Transform), (With<Player>)>) {
    if let Ok((mut rb, tf)) = query.single_mut() {
        if tf.translation.y > 7. {
            rb.linvel.y = -0.1;
        }
    }
}

fn count_score(mut query: Query<(&Transform, &mut ScoreTrigger)>, mut score: ResMut<Score>) {
    // println!("{}",  score.0);
    for ((tf, mut score_trigger)) in query.iter_mut() {
        if score_trigger.0 && tf.translation.x < -2.5 {
            score.0 += 1;
            score_trigger.0 = false;
        }
    }
}

fn clean_obstacles(mut commands: Commands, query: Query<(Entity, &Transform), (With<Obstacle>)>) {
    for (entity, tf) in query.iter() {
        if tf.translation.x < -8. {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Transform), (With<ScoreTrigger>)>,
) {
    let closest_obstacle: f32 = query
        .iter()
        .fold(f32::MIN, |acc, tf| acc.max(tf.translation.x));
    if (closest_obstacle - OBSTACLE_SPAWN_X).abs() > OBSTACLE_DISTANCE {
        let mut rng = thread_rng();
        let y = rng.gen_range(0.0..4.0) as f32;
        commands
            .spawn()
            .insert(Obstacle)
            .insert(ScoreTrigger(true))
            .insert(CanKillPlayer)
            .insert_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(bevy::prelude::shape::Cube { size: 1. })),
                material: materials.add(Color::rgb(0., 0., 0.1).into()),
                transform: Transform {
                    scale: Vec3::new(1., 8., 1.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(GameComponent)
            .insert_bundle(RigidBodyBundle {
                body_type: RigidBodyType::Dynamic,
                position: Vec3::new(OBSTACLE_SPAWN_X, 7. + y, 0.).into(),
                mass_properties: (RigidBodyMassPropsFlags::ROTATION_LOCKED_X
                    | RigidBodyMassPropsFlags::ROTATION_LOCKED_Y)
                    .into(),
                velocity: RigidBodyVelocity {
                    linvel: Vec3::new(OBSTACLE_MOVEMENT_SPEED, 0., 0.).into(),
                    ..Default::default()
                },
                forces: RigidBodyForces {
                    gravity_scale: 0.,
                    ..Default::default()
                },
                damping: RigidBodyDamping {
                    linear_damping: 0.,
                    angular_damping: 0.,
                },
                ..Default::default()
            })
            .insert_bundle(ColliderBundle {
                shape: ColliderShape::cuboid(0.5, 4., 0.5),
                flags: ColliderFlags {
                    active_events: ActiveEvents::CONTACT_EVENTS,
                    collision_groups: InteractionGroups {
                        memberships: 8,
                        filter: 4,
                    },
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(ColliderPositionSync::Discrete);

        commands
            .spawn()
            .insert(Obstacle)
            .insert(CanKillPlayer)
            .insert_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(bevy::prelude::shape::Cube { size: 1. })),
                material: materials.add(Color::rgb(0., 0., 1.).into()),
                transform: Transform {
                    scale: Vec3::new(1., 6., 1.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert_bundle(RigidBodyBundle {
                body_type: RigidBodyType::Dynamic,
                position: Vec3::new(OBSTACLE_SPAWN_X, -1. + y, 0.).into(),
                velocity: RigidBodyVelocity {
                    linvel: Vec3::new(OBSTACLE_MOVEMENT_SPEED, 0., 0.).into(),
                    ..Default::default()
                },
                forces: RigidBodyForces {
                    gravity_scale: 0.,
                    ..Default::default()
                },
                damping: RigidBodyDamping {
                    linear_damping: 0.,
                    angular_damping: 0.,
                },
                ..Default::default()
            })
            .insert(GameComponent)
            .insert_bundle(ColliderBundle {
                shape: ColliderShape::cuboid(0.5, 3., 0.5),
                flags: ColliderFlags {
                    active_events: ActiveEvents::CONTACT_EVENTS,
                    collision_groups: InteractionGroups {
                        memberships: 8,
                        filter: 4,
                    },
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(ColliderPositionSync::Discrete);
    }
}

fn fly(input: Res<Input<KeyCode>>, mut query: Query<(&mut RigidBodyVelocity), (With<Player>)>) {
    if input.just_pressed(KeyCode::Space) {
        for (mut velocity) in query.iter_mut() {
            velocity.linvel.y = MAX_FLY_SPEED.min(velocity.linvel.y.max(0.) + FLY_SPEED);
        }
    }
}

fn update_score(mut query: Query<(&mut Text), (With<ScoreText>)>, score: Res<Score>) {
    if let Ok(mut text) = query.single_mut() {
        text.sections[0].value = score.0.to_string();
    }
}

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut score: ResMut<Score>,
) {
    score.0 = 0;

    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(TextBundle {
            text: Text::with_section(
                "0",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 80.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..Default::default()
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Right,
                    ..Default::default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    right: Val::Percent(5.),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(GameComponent)
        .insert(ScoreText);

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(0.8, 0., 0.).into()),
            ..Default::default()
        })
        .insert(Player)
        .insert(GameComponent)
        .insert_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Dynamic,
            position: (Vec3::new(-2.0, 5., 0.0)).into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            flags: ColliderFlags {
                active_events: ActiveEvents::CONTACT_EVENTS,
                collision_groups: InteractionGroups {
                    memberships: 5,
                    filter: 10,
                },
                ..Default::default()
            },
            shape: ColliderShape::cuboid(0.05, 0.05, 0.05),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(Velocity(Vec2::new(0., 0.)));

    commands
        .spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Plane { size: 128. })),
            material: materials.add(Color::rgb(0., 0.8, 0.).into()),
            transform: Transform {
                scale: Vec3::new(10., 0., 10.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Static,
            position: Vec3::new(-0.0, -0.5, 0.0).into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(64., 0.5, 64.),
            flags: ColliderFlags {
                collision_groups: InteractionGroups {
                    memberships: 2,
                    filter: 1,
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(CanKillPlayer)
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
            transform: Transform::from_xyz(-3., 5., 10.).looking_at(Vec3::new(0., 4., 2.), Vec3::Y),
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
                    dbg!("play again");
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
    commands.spawn_bundle(UiCameraBundle::default());
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
                    horizontal: HorizontalAlign::Right,
                    ..Default::default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Percent(50.),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(GameOver);
}

fn stop_obstacles(mut query: Query<(&mut RigidBodyVelocity), (With<Obstacle>)>) {
    for (mut velocity) in query.iter_mut() {
        velocity.linvel.x = 0.;
    }
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


fn player_death(
    mut state: ResMut<State<AppState>>,
    mut contact_events: EventReader<ContactEvent>,
    mut query: Query<(&CanKillPlayer)>,
    mut player_query: Query<(&mut RigidBodyMassProps), (With<Player>)>,
) {
    for e in contact_events.iter() {
        if let ContactEvent::Started(c1, c2) = e {
            if query.get(c1.entity()).is_ok() || query.get(c2.entity()).is_ok() {
                // Ragdoll mode for player, unlock rotations
                if let Ok(mut mp) = player_query.single_mut() {
                    mp.flags = RigidBodyMassPropsFlags::empty();
                }

                state.set(AppState::game_over());
            }
        };
    }
}

pub fn lerp(min: f32, max: f32, value: f32) -> f32 {
    let diff = max - min;

    (value - min) / diff
}

#[cfg(test)]
mod tests {
    use super::lerp;

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(-3., 3., 0.,), 0.5);
        assert_eq!(lerp(0., 30., 6.,), 0.2);
    }
}
