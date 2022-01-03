use crate::prelude::*;

const FLY_SPEED: f32 = 8.;
const MAX_FLY_SPEED: f32 = 3.8;

pub fn player_death(
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

pub fn rotate_player_body(
    mut query: Query<(&RigidBodyVelocity, &mut RigidBodyPosition), (With<Player>)>,
) {
    for (vel, mut pos) in query.iter_mut() {
        let rotation_percentage = lerp(-3., 3., vel.linvel.y) * 2.0 - 1.;

        pos.position.rotation = Quat::from_rotation_z(rotation_percentage).into();
    }
}

pub fn clamp_player_y(mut query: Query<(&mut RigidBodyVelocity, &Transform), (With<Player>)>) {
    if let Ok((mut rb, tf)) = query.single_mut() {
        if tf.translation.y > 7. {
            rb.linvel.y = -0.1;
        }
    }
}

pub fn fly(input: Res<Input<KeyCode>>, mut query: Query<(&mut RigidBodyVelocity), (With<Player>)>) {
    if input.just_pressed(KeyCode::Space) {
        for (mut velocity) in query.iter_mut() {
            velocity.linvel.y = MAX_FLY_SPEED.min(velocity.linvel.y.max(0.) + FLY_SPEED);
        }
    }
}

pub fn count_score(mut query: Query<(&Transform, &mut ScoreTrigger)>, mut score: ResMut<Score>) {
    for ((tf, mut score_trigger)) in query.iter_mut() {
        if score_trigger.0 && tf.translation.x < -2.5 {
            score.0 += 1;
            score_trigger.0 = false;
        }
    }
}

pub fn setup_game(
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
        .insert(ColliderPositionSync::Discrete);

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

pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-3., 5., 10.).looking_at(Vec3::new(0., 4., 2.), Vec3::Y),
            ..Default::default()
        })
        .insert(Camera);
}

pub fn clean_up_previous_game(
    mut commands: Commands,
    menu_query: Query<(Entity), (With<GameComponent>)>,
) {
    for (entity) in menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn lerp(min: f32, max: f32, value: f32) -> f32 {
    let diff = max - min;

    (value - min) / diff
}

#[cfg(test)]
mod tests {
    use super::lerp;

    #[test]
    fn test_lerp() {
        assert!(lerp(-3., 3., 0.,) - 0.5 < f32::EPSILON);
        assert!(lerp(0., 30., 6.,) - 0.2 < f32::EPSILON);
    }
}
