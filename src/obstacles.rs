use rand::{thread_rng, Rng};

use crate::prelude::*;

const OBSTACLE_MOVEMENT_SPEED: f32 = -4.;
const OBSTACLE_DISTANCE: f32 = 4.;
const OBSTACLE_SPAWN_X: f32 = 8.;

pub fn clean(mut commands: Commands, query: Query<(Entity, &Transform), With<Obstacle>>) {
    for (entity, tf) in query.iter() {
        if tf.translation.x < -8. {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Transform, With<ScoreTrigger>>,
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
                body_type: RigidBodyType::Dynamic.into(),
                position: Vec3::new(OBSTACLE_SPAWN_X, 7. + y, 0.).into(),
                mass_properties: (RigidBodyMassPropsFlags::ROTATION_LOCKED_X
                    | RigidBodyMassPropsFlags::ROTATION_LOCKED_Y)
                    .into(),
                velocity: RigidBodyVelocity {
                    linvel: Vec3::new(OBSTACLE_MOVEMENT_SPEED, 0., 0.).into(),
                    ..Default::default()
                }.into(),
                forces: RigidBodyForces {
                    gravity_scale: 0.,
                    ..Default::default()
                }.into(),
                damping: RigidBodyDamping {
                    linear_damping: 0.,
                    angular_damping: 0.,
                }.into(),
                ..Default::default()
            })
            .insert_bundle(ColliderBundle {
                shape: ColliderShape::cuboid(0.5, 4., 0.5).into(),
                flags: ColliderFlags {
                    active_events: ActiveEvents::CONTACT_EVENTS,
                    collision_groups: InteractionGroups {
                        memberships: 8,
                        filter: 4,
                    },
                    ..Default::default()
                }.into(),
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
                body_type: RigidBodyType::Dynamic.into(),
                position: Vec3::new(OBSTACLE_SPAWN_X, -1. + y, 0.).into(),
                velocity: RigidBodyVelocity {
                    linvel: Vec3::new(OBSTACLE_MOVEMENT_SPEED, 0., 0.).into(),
                    ..Default::default()
                }.into(),
                forces: RigidBodyForces {
                    gravity_scale: 0.,
                    ..Default::default()
                }.into(),
                damping: RigidBodyDamping {
                    linear_damping: 0.,
                    angular_damping: 0.,
                }.into(),
                ..Default::default()
            })
            .insert(GameComponent)
            .insert_bundle(ColliderBundle {
                shape: ColliderShape::cuboid(0.5, 3., 0.5).into(),
                flags: ColliderFlags {
                    active_events: ActiveEvents::CONTACT_EVENTS,
                    collision_groups: InteractionGroups {
                        memberships: 8,
                        filter: 4,
                    }.into(),
                    ..Default::default()
                }.into(),
                ..Default::default()
            })
            .insert(ColliderPositionSync::Discrete);
    }
}

pub fn stop(mut query: Query<(&mut RigidBodyVelocityComponent), (With<Obstacle>)>) {
    for (mut velocity) in query.iter_mut() {
        velocity.linvel.x = 0.;
    }
}
