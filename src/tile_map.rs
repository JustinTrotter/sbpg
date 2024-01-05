// This example has a tutorial in the bevy_ecs_ldtk book associated with it:
// <https://trouv.github.io/bevy_ecs_ldtk/latest/tutorials/tile-based-game/index.html>
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween, TweenCompleted};
use std::{collections::HashSet, time::Duration};

use crate::{player::PlayerBundle, GameState};

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup)
            .insert_resource(LevelSelection::index(0))
            .register_ldtk_entity::<PlayerBundle>("Player")
            .register_ldtk_entity::<GoalBundle>("Goal")
            .register_ldtk_entity::<BlockBundle>("Block")
            .add_systems(
                Update,
                translate_grid_coords_entities.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                cache_wall_locations.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                move_complete_listener.run_if(in_state(GameState::Playing)),
            )
            .register_ldtk_int_cell::<WallBundle>(1)
            .init_resource::<LevelWalls>();
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("tile-based-game.ldtk"),
        ..Default::default()
    });
}

#[derive(Default, Component)]
pub struct Goal;

#[derive(Default, Bundle, LdtkEntity)]
pub struct GoalBundle {
    goal: Goal,
    #[sprite_sheet_bundle]
    sprite_bundle: SpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Default, Component)]
pub struct Wall;

#[derive(Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}

#[derive(Default, Component)]
pub struct Pushable;

#[derive(Default, Component)]
pub struct Pullable;

#[derive(Default, Component)]
pub struct Block;

#[derive(Default, Bundle, LdtkEntity)]
pub struct BlockBundle {
    block: Block,
    pushable: Pushable,
    pullable: Pullable,
    #[sprite_sheet_bundle]
    sprite_bundle: SpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Default, Resource)]
pub struct LevelWalls {
    wall_locations: HashSet<GridCoords>,
    level_width: i32,
    level_height: i32,
}

impl LevelWalls {
    pub fn in_wall(&self, grid_coords: &GridCoords) -> bool {
        grid_coords.x < 0
            || grid_coords.y < 0
            || grid_coords.x >= self.level_width
            || grid_coords.y >= self.level_height
            || self.wall_locations.contains(grid_coords)
    }
}

#[derive(Default, Component)]
pub struct IsMoving;

const GRID_SIZE: i32 = 16;

pub fn translate_grid_coords_entities(
    mut commands: Commands,
    mut grid_coords_entities: Query<(Entity, &mut Transform, &GridCoords), Changed<GridCoords>>,
) {
    for (entity, transform, grid_coords) in grid_coords_entities.iter_mut() {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(100),
            TransformPositionLens {
                start: transform.translation,
                end: Vec3::new(
                    bevy_ecs_ldtk::utils::grid_coords_to_translation(
                        *grid_coords,
                        IVec2::splat(GRID_SIZE),
                    )
                    .x,
                    bevy_ecs_ldtk::utils::grid_coords_to_translation(
                        *grid_coords,
                        IVec2::splat(GRID_SIZE),
                    )
                    .y,
                    0.,
                ),
            },
        )
        .with_completed_event(0);
        commands.entity(entity).insert(Animator::new(tween));
    }
}
fn move_complete_listener(
    mut commands: Commands,
    mut reader: EventReader<TweenCompleted>,
    query: Query<(Entity, &IsMoving)>,
) {
    for _ in reader.iter() {
        for (entity, _) in query.iter() {
            commands.entity(entity).remove::<IsMoving>();
        }
    }
}

pub fn cache_wall_locations(
    mut level_walls: ResMut<LevelWalls>,
    mut level_events: EventReader<LevelEvent>,
    walls: Query<&GridCoords, With<Wall>>,
    ldtk_project_entities: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    for level_event in level_events.iter() {
        if let LevelEvent::Spawned(level_iid) = level_event {
            let ldtk_project = ldtk_project_assets
                .get(ldtk_project_entities.single())
                .expect("LdtkProject should be loaded when level is spawned");
            let level = ldtk_project
                .get_raw_level_by_iid(level_iid.get())
                .expect("spawned level should exist in project");

            let wall_locations = walls.iter().copied().collect();

            let new_level_walls = LevelWalls {
                wall_locations,
                level_width: level.px_wid / GRID_SIZE,
                level_height: level.px_hei / GRID_SIZE,
            };

            *level_walls = new_level_walls;
        }
    }
}
