use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::map::{Collide, TILE_SIZE};
use crate::GameState;
use bevy::prelude::*;

pub struct PlayerPlugin;

pub struct Player;
pub struct PlayerCamera;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(spawn_player.system())
                .with_system(spawn_camera.system()),
        )
        .add_system_set(SystemSet::on_update(GameState::Playing).with_system(move_player.system()))
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(remove_player.system()));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(PlayerCamera);
}

fn spawn_player(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(textures.texture_player.clone().into()),
            transform: Transform::from_translation(Vec3::new(64., 64., 1.)),
            ..Default::default()
        })
        .insert(Player);
}

fn move_player(
    time: Res<Time>,
    actions: Res<Actions>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    collider_query: Query<&Collide>,
) {
    if actions.player_movement.is_none() {
        return;
    }
    let speed = 150.;
    let movement = Vec3::new(
        actions.player_movement.unwrap().x * speed * time.delta_seconds(),
        actions.player_movement.unwrap().y * speed * time.delta_seconds(),
        0.,
    );
    for mut player_transform in player_query.iter_mut() {
        let x =
            ((player_transform.translation.x + movement.x + TILE_SIZE / 2.) / TILE_SIZE) as usize;
        let y =
            ((player_transform.translation.y + movement.y + TILE_SIZE / 2.) / TILE_SIZE) as usize;
        for collide in collider_query.iter() {
            if collide.x == x && collide.y == y {
                return;
            }
        }
        player_transform.translation += movement;
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
        }
    }
}

fn remove_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    for player in player_query.iter() {
        commands.entity(player).despawn();
    }
}
