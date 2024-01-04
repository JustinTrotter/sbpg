use crate::tile_map::Block;
use crate::tile_map::Goal;
use crate::tile_map::IsMoving;
use crate::tile_map::LevelWalls;
use crate::GameState;
use crate::tile_map::Pushable;
use bevy::prelude::*;
use bevy_ecs_ldtk::{GridCoords, LdtkEntity, LevelSelection};

pub struct PlayerPlugin;

#[derive(Default, Component)]
pub struct Player;

#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    player: Player,
    #[sprite_sheet_bundle]
    sprite_bundle: SpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PushEvent>();
        app.add_systems(
            Update,
            (
                move_player_from_input.run_if(in_state(GameState::Playing)),
                check_goal.run_if(in_state(GameState::Playing)),
                push_system.run_if(in_state(GameState::Playing))
            ),
        );
    }
}

#[derive(Event)]
pub struct PushEvent(Entity, GridCoords);

pub fn push_system(
    mut commands: Commands,
    mut ev_push: EventReader<PushEvent>,
    mut pushable: Query<(Entity, &mut GridCoords), With<Pushable>>,
    level_walls: Res<LevelWalls>,
){
    let mut blocks: Vec<(Entity, GridCoords)> = Vec::new();
    for (entity, block_grid_coords) in pushable.iter() {
        blocks.push((entity,*block_grid_coords));
    }
    for ev in ev_push.iter() {
        for (entity, mut grid_coords) in pushable.iter_mut() {
            let destination = *grid_coords + ev.1;
            if entity == ev.0 {
                let mut hit_block = false;
                for (_, cords) in blocks.iter() {
                    if *cords == destination {
                        hit_block = true;
                    }
                }

                if !hit_block && !level_walls.in_wall(&destination) {
                    commands.entity(entity).insert(IsMoving);
                    *grid_coords = destination;
                }
            }
        }
    }
}


pub fn move_player_from_input(
    mut commands: Commands,
    mut set: ParamSet<(
        Query<(Entity, &mut GridCoords), (With<Player>, Without<IsMoving>)>,
        Query<(Entity, &mut GridCoords), (With<Block>, Without<IsMoving>)>,
    )>,
    input: Res<Input<KeyCode>>,
    level_walls: Res<LevelWalls>,
    mut ev_push: EventWriter<PushEvent>
) {
    let movement_direction = if input.pressed(KeyCode::W) {
        GridCoords::new(0, 1)
    } else if input.pressed(KeyCode::A) {
        GridCoords::new(-1, 0)
    } else if input.pressed(KeyCode::S) {
        GridCoords::new(0, -1)
    } else if input.pressed(KeyCode::D) {
        GridCoords::new(1, 0)
    } else {
        return;
    };

    let mut blocks: Vec<(Entity, GridCoords)> = Vec::new();
    for (entity, block_grid_coords) in set.p1().iter() {
        blocks.push((entity,*block_grid_coords));
    }

    for (entity, mut player_grid_coords) in set.p0().iter_mut() {
        let destination = *player_grid_coords + movement_direction;
        let block_destination = *player_grid_coords + movement_direction + movement_direction;
        let mut hit_block = false;
        let mut hit_second_block = false;
        for (entity, cords) in blocks.iter() {
            if *cords == destination {
                hit_block = true;
                ev_push.send(PushEvent(*entity, movement_direction));

            }
            if *cords == block_destination {
                hit_second_block = true;
            }
        }
        if hit_block {
            if !level_walls.in_wall(&block_destination) && !hit_second_block {
                commands.entity(entity).insert(IsMoving);
                *player_grid_coords = destination;
            }
        } else if !hit_block && !level_walls.in_wall(&destination) {
            commands.entity(entity).insert(IsMoving);
            *player_grid_coords = destination;
        }
    }
}

pub fn check_goal(
    level_selection: ResMut<LevelSelection>,
    players: Query<&GridCoords, (With<Player>, Changed<GridCoords>)>,
    goals: Query<&GridCoords, With<Goal>>,
) {
    if players
        .iter()
        .zip(goals.iter())
        .any(|(player_grid_coords, goal_grid_coords)| player_grid_coords == goal_grid_coords)
    {
        let indices = match level_selection.into_inner() {
            LevelSelection::Indices(indices) => indices,
            _ => panic!("level selection should always be Indices in this game"),
        };

        indices.level += 1;
    }
}
