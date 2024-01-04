use crate::tile_map::Goal;
use crate::tile_map::IsMoving;
use crate::tile_map::LevelWalls;
use crate::GameState;
use bevy::prelude::*;
use bevy_ecs_ldtk::{GridCoords, LdtkEntity, LevelSelection};


pub struct PlayerPlugin;

#[derive(Default, Component)]
pub struct Player {
    pub is_moving: bool,
}

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
        app.add_systems(
            Update,
            (
                move_player_from_input.run_if(in_state(GameState::Playing)),
                check_goal.run_if(in_state(GameState::Playing)),
            ),
        );
    }
}

pub fn move_player_from_input(
    mut commands: Commands,
    mut players: Query<(Entity, &mut GridCoords, &Player), Without<IsMoving>>,
    input: Res<Input<KeyCode>>,
    level_walls: Res<LevelWalls>,
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

    for (entity, mut player_grid_coords, _) in players.iter_mut() {
        let destination = *player_grid_coords + movement_direction;
        if !level_walls.in_wall(&destination) {
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
