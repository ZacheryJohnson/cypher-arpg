use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::prelude::{Commands, Entity, Query, Res, ResMut, Time, Transform, Vec2, With, Without};
use bevy_renet::renet::{DefaultChannel, RenetServer};
use cypher_net::components::net_entity::NetEntity;
use cypher_net::components::server_entity::ServerEntity;
use cypher_net::messages::server::server_message::ServerMessage;
use cypher_net::resources::server_net_entity_registry::ServerNetEntityRegistry;

use crate::components::collider::Collider;
use crate::components::hit_points::HitPoints;
use crate::components::projectile::Projectile;
use crate::components::team::Team;
use crate::resources::world_state::{DeathEvent, LootPoolDropper, WorldState};

type CollidableQueryAccessT<'a> = (
    &'a Transform,
    &'a mut HitPoints,
    &'a Team,
    Option<&'a LootPoolDropper>,
    Entity,
    &'a NetEntity,
);
type CollidableQueryFilterT = (With<Collider>, Without<Projectile>, With<ServerEntity>);

pub fn update_projectiles(
    mut commands: Commands,
    mut projectiles: Query<
        (&mut Transform, &mut Projectile, Entity, &NetEntity),
        With<ServerEntity>,
    >,
    mut collidables: Query<CollidableQueryAccessT, CollidableQueryFilterT>,
    time: Res<Time>,
    mut game_state: ResMut<WorldState>,
    mut server: ResMut<RenetServer>,
    mut net_entities: ResMut<ServerNetEntityRegistry>,
) {
    for (mut projectile_transform, mut projectile, entity, net_entity) in &mut projectiles {
        let forward = -projectile_transform.local_y();
        let distance = forward * projectile.move_speed * time.delta().as_secs_f32();
        projectile_transform.translation += distance;
        projectile.lifetime -= distance.length();

        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();

            server.broadcast_message(
                DefaultChannel::ReliableOrdered,
                ServerMessage::EntityDestroyed {
                    net_entity_id: net_entity.id,
                }
                .serialize()
                .unwrap(),
            );

            continue;
        }

        for (
            collidable_transform,
            mut hit_points,
            team,
            maybe_loot,
            collider_entity,
            collider_net_entity,
        ) in &mut collidables
        {
            // Don't let projectiles hurt their own team members
            if team.id == projectile.team_id {
                continue;
            }

            let projectile_aabb = Aabb2d::new(
                projectile_transform.translation.truncate(),
                projectile_transform.scale.truncate(),
            );
            let collidable_aabb = Aabb2d::new(
                collidable_transform.translation.truncate(),
                collidable_transform.scale.truncate(),
            );
            if projectile_aabb.intersects(&collidable_aabb) {
                hit_points.health -= projectile.damage;
                if hit_points.health <= 0.0 {
                    commands.entity(collider_entity).despawn();
                    net_entities.delete(&collider_net_entity.id);

                    server.broadcast_message(
                        DefaultChannel::ReliableOrdered,
                        ServerMessage::EntityDestroyed {
                            net_entity_id: collider_net_entity.id,
                        }
                        .serialize()
                        .unwrap(),
                    );

                    game_state.death_events.send(DeathEvent {
                        loot_pool: maybe_loot.map(|loot| loot.to_owned()),
                        position: Vec2 {
                            x: collidable_transform.translation.x,
                            y: collidable_transform.translation.y,
                        },
                    });
                }

                commands.entity(entity).despawn();
                net_entities.delete(&net_entity.id);

                server.broadcast_message(
                    DefaultChannel::ReliableOrdered,
                    ServerMessage::EntityDestroyed {
                        net_entity_id: net_entity.id,
                    }
                    .serialize()
                    .unwrap(),
                );

                continue;
            }
        }

        server.broadcast_message(
            DefaultChannel::Unreliable,
            ServerMessage::EntityTransformUpdate {
                net_entity_id: net_entity.id,
                transform: *projectile_transform,
            }
            .serialize()
            .unwrap(),
        );
    }
}
