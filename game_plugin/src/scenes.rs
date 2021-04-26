use crate::audio::{AudioEffect, PauseBackground, ResumeBackground};
use crate::loading::{AudioAssets, TextureAssets};
use crate::map::{Acorn, ActiveElement, Collide, Map};
use crate::player::{Player, PlayerCamera};
use crate::{GameStage, GameState};
use bevy::prelude::*;
use std::f32::consts::PI;
use std::time::Duration;

pub struct ScenesPlugin;

pub struct TriggerScene {
    pub scene: CutScene,
}

#[derive(Clone)]
pub enum CutScene {
    Intro {
        camera_from: (f32, f32),
        camera_to: (f32, f32),
        acorn_falls: bool,
    },
    ActivateButton {
        button: Entity,
        player: (f32, f32),
        wall: (f32, f32),
    },
    MapTransition {
        to: Map,
    },
}

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<TriggerScene>().add_system_set(
            SystemSet::on_update(GameStage::Playing)
                .with_system(run_intro.system())
                .with_system(run_transition_scene.system())
                .with_system(run_activate_button_scene.system())
                .with_system(trigger_scene.system()),
        );
    }
}

fn run_intro(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    current_map: Res<Map>,
    time: Res<Time>,
    mut acorn: Query<(Entity, &mut Transform), (With<Acorn>, Without<PlayerCamera>)>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<Acorn>)>,
) {
    if let Some(scene) = game_state.scene.clone() {
        if let CutScene::Intro {
            camera_from,
            camera_to,
            acorn_falls,
        } = scene
        {
            const CAMERA_ON_PLAYER: Duration = Duration::from_millis(500);
            const CAMERA_TO_GOAL: Duration = Duration::from_millis(1500);
            const CAMERA_ON_GOAL: Duration = Duration::from_millis(2500);
            const CAMERA_BACK_TO_PLAYER: Duration = Duration::from_millis(3500);

            if time
                .time_since_startup()
                .lt(&(game_state.scene_start + CAMERA_ON_PLAYER))
            {
                return;
            }

            if time
                .time_since_startup()
                .gt(&(game_state.scene_start + CAMERA_BACK_TO_PLAYER))
            {
                if let Ok(mut transform) = camera.single_mut() {
                    transform.translation.x = camera_from.0;
                    transform.translation.y = camera_from.1;
                }
                game_state.scene = None;
                game_state.frozen = false;
                return;
            }

            if time
                .time_since_startup()
                .gt(&(game_state.scene_start + CAMERA_TO_GOAL))
                && time
                    .time_since_startup()
                    .lt(&(game_state.scene_start + CAMERA_ON_GOAL))
            {
                if let Ok((_acorn, mut acorn_transform)) = acorn.single_mut() {
                    let goal = current_map.goal_position();
                    let acorn = current_map.acorn_position();
                    let time_delta = (CAMERA_ON_GOAL - CAMERA_TO_GOAL).as_secs_f32() / 2.;
                    let mut partial = (2.
                        - ((game_state.scene_start + CAMERA_ON_GOAL - time.time_since_startup())
                            .as_secs_f32()
                            / time_delta))
                        .clamp(0., 2.);
                    if !acorn_falls {
                        partial /= 2.;
                    }
                    if partial < 1. {
                        let acorn_path =
                            (Vec2::new(goal.0, goal.1) - Vec2::new(acorn.0, acorn.1)) * partial;
                        acorn_transform.translation.x = acorn.0 + acorn_path.x;
                        acorn_transform.translation.y = acorn.1 + acorn_path.y;
                    } else {
                        acorn_transform.rotation = Quat::from_rotation_z(partial * 2. * PI);
                        acorn_transform.scale *= 0.95;
                    }
                }
                if let Ok(mut transform) = camera.single_mut() {
                    transform.translation.x = camera_to.0;
                    transform.translation.y = camera_to.1;
                }
                return;
            }

            if game_state.scene_step == 0
                && time
                    .time_since_startup()
                    .gt(&(game_state.scene_start + CAMERA_ON_GOAL))
            {
                game_state.scene_step += 1;
                if acorn_falls {
                    if let Ok((acorn, _acorn_transform)) = acorn.single_mut() {
                        commands.entity(acorn).despawn();
                    }
                }
            }

            let to_animate = if time
                .time_since_startup()
                .lt(&(CAMERA_TO_GOAL + game_state.scene_start))
            {
                (Vec2::new(camera_to.0 - camera_from.0, camera_to.1 - camera_from.1)
                    / (CAMERA_TO_GOAL - CAMERA_ON_PLAYER).as_secs_f32())
                    * time.delta().as_secs_f32()
            } else {
                (Vec2::new(camera_from.0 - camera_to.0, camera_from.1 - camera_to.1)
                    / (CAMERA_BACK_TO_PLAYER - CAMERA_ON_GOAL).as_secs_f32())
                    * time.delta().as_secs_f32()
            };
            if let Ok(mut transform) = camera.single_mut() {
                transform.translation.x += to_animate.x;
                transform.translation.y += to_animate.y;
            }
        }
    }
}

fn run_transition_scene(
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut current_map: ResMut<Map>,
    mut audio_effect: EventWriter<AudioEffect>,
    audio_assets: Res<AudioAssets>,
    mut pause_background: EventWriter<PauseBackground>,
    mut player: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    if let Some(scene) = game_state.scene.clone() {
        if let CutScene::MapTransition { to } = scene {
            if game_state.scene_step == 0 {
                game_state.scene_step += 1;
                pause_background.send(PauseBackground);
                audio_effect.send(AudioEffect {
                    handle: audio_assets.fall.clone(),
                })
            }
            let camera_scale_offset: Vec3 = Vec3::new(-0.98, -0.98, 0.);
            let player_scale_offset: Vec3 = Vec3::new(-0.95, -0.95, 0.);
            const ZOOM: Duration = Duration::from_secs(2);

            if time
                .time_since_startup()
                .lt(&(game_state.scene_start + ZOOM))
            {
                if let Ok(mut transform) = camera.single_mut() {
                    transform.scale +=
                        (camera_scale_offset / ZOOM.as_secs_f32()) * time.delta().as_secs_f32();
                }
                if let Ok(mut transform) = player.single_mut() {
                    transform.scale +=
                        (player_scale_offset / ZOOM.as_secs_f32()) * time.delta().as_secs_f32();
                }
                return;
            }
            *current_map = to.clone();
            game_state.scene = None;
            game_state.frozen = false;
            return;
        }
    }
}

fn run_activate_button_scene(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    textures: Res<TextureAssets>,
    mut audio_effect: EventWriter<AudioEffect>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    audio_assets: Res<AudioAssets>,
    mut pause_background: EventWriter<PauseBackground>,
    mut resume_background: EventWriter<ResumeBackground>,
    mut elements: Query<
        (Entity, &Transform, &mut Handle<ColorMaterial>),
        (With<ActiveElement>, Without<PlayerCamera>),
    >,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<ActiveElement>)>,
) {
    if let Some(scene) = game_state.scene.clone() {
        if let CutScene::ActivateButton {
            button,
            player,
            wall,
        } = scene
        {
            if game_state.scene_step == 0 {
                game_state.scene_step += 1;
                if let Ok((_entity, _transform, mut material)) = elements.get_mut(button) {
                    *material = materials.add(textures.texture_button_down.clone().into());
                }
                pause_background.send(PauseBackground);
                audio_effect.send(AudioEffect {
                    handle: audio_assets.button_click.clone(),
                })
            }
            const CAMERA_ON_PLAYER: Duration = Duration::from_millis(300);
            const CAMERA_TO_WALL: Duration = Duration::from_millis(1000);
            const CAMERA_ON_WALL: Duration = Duration::from_millis(1300);
            const CAMERA_BACK_TO_PLAYER: Duration = Duration::from_millis(2000);

            if time
                .time_since_startup()
                .lt(&(game_state.scene_start + CAMERA_ON_PLAYER))
            {
                return;
            }

            if time
                .time_since_startup()
                .gt(&(game_state.scene_start + CAMERA_BACK_TO_PLAYER))
            {
                resume_background.send(ResumeBackground);
                if let Ok(mut transform) = camera.single_mut() {
                    transform.translation.x = player.0;
                    transform.translation.y = player.1;
                }
                game_state.scene = None;
                game_state.frozen = false;
                return;
            }

            if time
                .time_since_startup()
                .gt(&(game_state.scene_start + CAMERA_TO_WALL))
                && time
                    .time_since_startup()
                    .lt(&(game_state.scene_start + CAMERA_ON_WALL))
            {
                if game_state.scene_step == 1 {
                    game_state.scene_step += 1;
                    audio_effect.send(AudioEffect {
                        handle: audio_assets.wall_moving.clone(),
                    })
                }
                if let Ok(mut transform) = camera.single_mut() {
                    transform.translation.x = wall.0;
                    transform.translation.y = wall.1;
                }
                return;
            }

            if game_state.scene_step == 2 {
                game_state.scene_step += 1;
                for (entity, transform, mut material) in elements.iter_mut() {
                    if transform.translation.x == wall.0 && transform.translation.y == wall.1 {
                        *material = materials.add(textures.texture_wall_down.clone().into());
                        commands.entity(entity).remove::<Collide>();
                    }
                }
            }

            let to_animate = if time
                .time_since_startup()
                .lt(&(CAMERA_TO_WALL + game_state.scene_start))
            {
                (Vec2::new(wall.0 - player.0, wall.1 - player.1)
                    / (CAMERA_TO_WALL - CAMERA_ON_PLAYER).as_secs_f32())
                    * time.delta().as_secs_f32()
            } else {
                (Vec2::new(player.0 - wall.0, player.1 - wall.1)
                    / (CAMERA_BACK_TO_PLAYER - CAMERA_ON_WALL).as_secs_f32())
                    * time.delta().as_secs_f32()
            };
            if let Ok(mut transform) = camera.single_mut() {
                transform.translation.x += to_animate.x;
                transform.translation.y += to_animate.y;
            }
        }
    }
}

fn trigger_scene(
    time: Res<Time>,
    mut trigger_scene: EventReader<TriggerScene>,
    mut game_state: ResMut<GameState>,
) {
    for event in trigger_scene.iter() {
        game_state.scene = Some(event.scene.clone());
        game_state.frozen = true;
        game_state.scene_start = time.time_since_startup();
        game_state.scene_step = 0;
    }
}
