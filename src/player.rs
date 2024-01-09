use crate::tile_map::Block;
use crate::tile_map::Goal;
use crate::tile_map::IsMoving;
use crate::tile_map::LevelWalls;
use crate::tile_map::Pushable;
use crate::GameState;
use bevy::prelude::*;
use bevy_ecs_ldtk::{GridCoords, LdtkEntity, LevelSelection};

pub struct PlayerPlugin;

#[derive(Default, Debug, Copy, Clone)]
pub enum Direction {
    North,
    #[default]
    East,
    South,
    West,
    None,
}

impl PartialEq for Direction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Direction::North, Direction::North) => true,
            (Direction::East, Direction::East) => true,
            (Direction::South, Direction::South) => true,
            (Direction::West, Direction::West) => true,
            (Direction::None, Direction::None) => true,
            _ => false,
        }
    }
}

#[derive(Default, Component)]
pub struct Player {
    face_direction: Direction,
}

#[derive(Default, Component)]
pub struct Movable {
    north_neighbor: Option<Entity>,
    east_neighbor: Option<Entity>,
    south_neighbor: Option<Entity>,
    west_neighbor: Option<Entity>,
}

pub fn update_moveable_neighbors(
    mut movable_query: Query<(&GridCoords, &mut Movable)>,
    neighbor_query: Query<(Entity, &GridCoords), With<Movable>>,
) {
    for (movable_grid_coords, mut movable) in movable_query.iter_mut() {
        movable.north_neighbor = None;
        movable.east_neighbor = None;
        movable.south_neighbor = None;
        movable.west_neighbor = None;
        for (entity, neighbor_coords) in neighbor_query.iter() {
            if *movable_grid_coords + GridCoords::new(0, 1) == *neighbor_coords {
                movable.north_neighbor = Some(entity);
            } else if *movable_grid_coords + GridCoords::new(1, 0) == *neighbor_coords {
                movable.east_neighbor = Some(entity);
            } else if *movable_grid_coords + GridCoords::new(0, -1) == *neighbor_coords {
                movable.south_neighbor = Some(entity);
            } else if *movable_grid_coords + GridCoords::new(-1, 0) == *neighbor_coords {
                movable.west_neighbor = Some(entity);
            }
        }
    }
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    player: Player,
    movable: Movable,
    #[sprite_sheet_bundle]
    sprite_bundle: SpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalPlayerState>();
        app.add_event::<PushMoveEvent>();
        app.add_event::<PlayerMoveEvent>();
        app.add_event::<PullMoveEvent>();
        app.add_systems(
            Update,
            (
                turn_player_from_input.run_if(in_state(GameState::Playing)),
                grab_from_held_input.run_if(in_state(GameState::Playing)),
                ungrab_from_release_input.run_if(in_state(GameState::Playing)),
                highlight_grabbed.run_if(in_state(GameState::Playing)),
                unhighlight_grabbed.run_if(in_state(GameState::Playing)),
                update_player_facing_direction.run_if(in_state(GameState::Playing)),
                // check_goal.run_if(in_state(GameState::Playing)),
                // move_pushable_from_input.run_if(in_state(GameState::Playing)),
                //
                handle_move_player
                    // .before(translate_grid_coords_entities)
                    .run_if(in_state(GameState::Playing)),
                handle_move_player_event
                    .run_if(in_state(GameState::Playing))
                    .after(handle_move_player),
                update_moveable_neighbors
                    .after(handle_move_player_event)
                    .run_if(in_state(GameState::Playing)),
            ),
        );
    }
}
#[derive(Event)]
pub struct PushMoveEvent(Entity, Direction);

#[derive(Event)]
pub struct PullMoveEvent(Entity, Direction);

#[derive(Event)]
pub struct PlayerMoveEvent(Entity, Direction);

pub fn handle_move_player(
    mut commands: Commands,
    moving_player_query: Query<Entity, (With<Player>, With<IsMoving>)>,
    mut player_query: Query<
        (
            Entity,
            &Movable,
            Option<&Block>,
            Option<&Grabbed>,
            Option<&Grabbing>,
        ),
        Without<IsMoving>,
    >,
    grid_coords_query: Query<&GridCoords, With<Movable>>,
    movible_query: Query<&Movable>,
    block_query: Query<&Block, Without<Grabbed>>,
    level_walls: Res<LevelWalls>,
    input: Res<Input<KeyCode>>,
    mut ev_player_move: EventWriter<PlayerMoveEvent>,
    global_player_state: Res<GlobalPlayerState>,
) {
    // if any player is moving, don't move any players
    // this is very important because otherwise the will move
    // out of sync and have a chance of merging into one space
    if moving_player_query.iter().count() > 0 {
        return;
    }
    let movement_direction = get_movement_direction_from_input(&input);
    if movement_direction == Direction::None {
        return;
    }
    for (entity, movable, block, grabbed, grabbing) in player_query.iter_mut() {
        if grabbed.is_none() && block.is_some() {
            continue;
        }

        // if grabbing, only move if facing direction is the same as movement direction
        if (grabbing.is_some() || grabbed.is_some() || global_player_state.grabbing)
            && (global_player_state.direction != movement_direction
                && global_player_state.direction != get_reversed_direction(movement_direction))
        {
            continue;
        }

        let player_grid_coords = grid_coords_query.get(entity).unwrap();
        let player_destination = *player_grid_coords
            + match movement_direction {
                Direction::North => GridCoords::new(0, 1),
                Direction::East => GridCoords::new(1, 0),
                Direction::South => GridCoords::new(0, -1),
                Direction::West => GridCoords::new(-1, 0),
                _ => GridCoords::new(0, 0),
            };
        let can_move = can_move(
            &grid_coords_query,
            movement_direction,
            movable,
            &movible_query,
            &level_walls,
            &block_query,
        );
        if can_move && !level_walls.in_wall(&player_destination) {
            commands.entity(entity).insert(IsMoving);
            ev_player_move.send(PlayerMoveEvent(entity, movement_direction));
        }
    }
}

pub fn handle_move_player_event(
    mut ev_player_move: EventReader<PlayerMoveEvent>,
    mut grid_coords_query: Query<&mut GridCoords, With<Movable>>,
) {
    for ev in ev_player_move.iter() {
        let mut player_grid_coords = grid_coords_query.get_mut(ev.0).unwrap();
        let player_destination = *player_grid_coords
            + match ev.1 {
                Direction::North => GridCoords::new(0, 1),
                Direction::East => GridCoords::new(1, 0),
                Direction::South => GridCoords::new(0, -1),
                Direction::West => GridCoords::new(-1, 0),
                _ => GridCoords::new(0, 0),
            };
        *player_grid_coords = player_destination;
    }
}

fn can_move(
    grid_coords_query: &Query<&GridCoords, With<Movable>>,
    direction: Direction,
    movable: &Movable,
    movable_query: &Query<&Movable>,
    level_walls: &Res<LevelWalls>,
    block_query: &Query<&Block, Without<Grabbed>>,
) -> bool {
    let neighbor_entity = match direction {
        Direction::North => movable.north_neighbor,
        Direction::East => movable.east_neighbor,
        Direction::South => movable.south_neighbor,
        Direction::West => movable.west_neighbor,
        _ => None,
    };
    if neighbor_entity.is_none() {
        return true;
    }
    let neighbor_entity = neighbor_entity.unwrap();
    let is_neighbor_block = block_query.get(neighbor_entity).is_ok();
    if is_neighbor_block {
        return false;
    }
    let neighbor_movable = movable_query.get(neighbor_entity).unwrap();
    let neighbor_grid_coords = grid_coords_query.get(neighbor_entity).unwrap();
    let neighbor_destination = *neighbor_grid_coords
        + match direction {
            Direction::North => GridCoords::new(0, 1),
            Direction::East => GridCoords::new(1, 0),
            Direction::South => GridCoords::new(0, -1),
            Direction::West => GridCoords::new(-1, 0),
            _ => GridCoords::new(0, 0),
        };
    if level_walls.in_wall(&neighbor_destination) {
        return false;
    }

    return can_move(
        grid_coords_query,
        direction,
        neighbor_movable,
        movable_query,
        level_walls,
        block_query,
    );
}

pub fn turn_player_from_input(
    mut player_query: Query<(&mut Player, Option<&Grabbing>), Without<IsMoving>>,
    moving_player_query: Query<Entity, With<IsMoving>>,
    input: Res<Input<KeyCode>>,
    global_player_state: Res<GlobalPlayerState>,
) {
    // if any player is moving, don't change facing of any players
    // this is very important because otherwise the will move
    // out of sync and have a chance of merging into one space
    if moving_player_query.iter().count() > 0 {
        return;
    }
    for (mut player, grabbing) in player_query.iter_mut() {
        if grabbing.is_some() || global_player_state.grabbing {
            continue;
        }
        if input.pressed(KeyCode::W) {
            player.face_direction = Direction::North;
        } else if input.pressed(KeyCode::A) {
            player.face_direction = Direction::West;
        } else if input.pressed(KeyCode::S) {
            player.face_direction = Direction::South;
        } else if input.pressed(KeyCode::D) {
            player.face_direction = Direction::East;
        }
    }
}

fn get_neighbor_direction(origin: &GridCoords, neighbor: &GridCoords) -> Direction {
    if *origin + GridCoords::new(0, 1) == *neighbor {
        return Direction::North;
    } else if *origin + GridCoords::new(1, 0) == *neighbor {
        return Direction::East;
    } else if *origin + GridCoords::new(0, -1) == *neighbor {
        return Direction::South;
    } else if *origin + GridCoords::new(-1, 0) == *neighbor {
        return Direction::West;
    }
    return Direction::None;
}

#[derive(Default, Component)]
pub struct Grabbed;

#[derive(Default, Component)]
pub struct Grabbing;

#[derive(Resource)]
pub struct GlobalPlayerState {
    pub direction: Direction,
    pub grabbing: bool,
}

impl Default for GlobalPlayerState {
    fn default() -> Self {
        GlobalPlayerState {
            direction: Direction::East,
            grabbing: false,
        }
    }
}

pub fn grab_from_held_input(
    mut commands: Commands,
    player_query: Query<(Entity, &Player, &GridCoords), Without<IsMoving>>,
    pushable_query: Query<(Entity, &GridCoords), (Without<IsMoving>, With<Pushable>)>,
    input: Res<Input<KeyCode>>,
    mut global_player_state: ResMut<GlobalPlayerState>,
) {
    if input.just_pressed(KeyCode::Space) {
        for (entity, pushable_grid_coords) in pushable_query.iter() {
            for (player_entity, player, player_grid_coords) in player_query.iter() {
                if get_neighbor_direction(player_grid_coords, pushable_grid_coords)
                    == player.face_direction
                {
                    commands.entity(entity).insert(Grabbed);
                    commands.entity(player_entity).insert(Grabbing);
                    global_player_state.direction = player.face_direction;
                    global_player_state.grabbing = true;
                }
            }
        }
    }
}

pub fn ungrab_from_release_input(
    mut commands: Commands,
    movable_query: Query<Entity, With<Movable>>,
    input: Res<Input<KeyCode>>,
    mut global_player_state: ResMut<GlobalPlayerState>,
) {
    if input.just_released(KeyCode::Space) {
        for entity in movable_query.iter() {
            commands.entity(entity).remove::<Grabbed>();
            commands.entity(entity).remove::<Grabbing>();
            global_player_state.grabbing = false;
        }
    }
}

pub fn update_player_facing_direction(
    mut player_query: Query<(&mut Player, &mut TextureAtlasSprite), Changed<Player>>,
) {
    for (player, mut sprite) in player_query.iter_mut() {
        match player.face_direction {
            Direction::North => sprite.index = 2,
            Direction::East => {
                sprite.index = 1;
                sprite.flip_x = true;
            }
            Direction::South => sprite.index = 0,
            Direction::West => {
                sprite.index = 1;
                sprite.flip_x = false;
            }
            _ => {
                // default to facing right
                sprite.index = 1;
                sprite.flip_x = true;
            }
        }
    }
}

pub fn highlight_grabbed(mut grabbed_query: Query<&mut TextureAtlasSprite, With<Grabbed>>) {
    for mut sprite in grabbed_query.iter_mut() {
        sprite.color = Color::rgb(0.0, 1.0, 0.0);
    }
}

pub fn unhighlight_grabbed(mut grabbed_query: Query<&mut TextureAtlasSprite, Without<Grabbed>>) {
    for mut sprite in grabbed_query.iter_mut() {
        sprite.color = Color::rgb(1.0, 1.0, 1.0);
    }
}

fn get_movement_coords_from_input(input: &Res<Input<KeyCode>>) -> Option<GridCoords> {
    if input.pressed(KeyCode::W) {
        return Some(GridCoords::new(0, 1));
    } else if input.pressed(KeyCode::A) {
        return Some(GridCoords::new(-1, 0));
    } else if input.pressed(KeyCode::S) {
        return Some(GridCoords::new(0, -1));
    } else if input.pressed(KeyCode::D) {
        return Some(GridCoords::new(1, 0));
    }
    return None;
}

fn get_movement_coords_from_direction(direction: Direction) -> Option<GridCoords> {
    if direction == Direction::North {
        return Some(GridCoords::new(0, 1));
    } else if direction == Direction::West {
        return Some(GridCoords::new(-1, 0));
    } else if direction == Direction::South {
        return Some(GridCoords::new(0, -1));
    } else if direction == Direction::East {
        return Some(GridCoords::new(1, 0));
    }
    return None;
}
//recursive function to see if neighbor can move

fn get_movement_direction_from_input(input: &Res<Input<KeyCode>>) -> Direction {
    if input.pressed(KeyCode::W) {
        return Direction::North;
    } else if input.pressed(KeyCode::A) {
        return Direction::West;
    } else if input.pressed(KeyCode::S) {
        return Direction::South;
    } else if input.pressed(KeyCode::D) {
        return Direction::East;
    }
    return Direction::None;
}

fn get_reversed_direction(direction: Direction) -> Direction {
    if direction == Direction::North {
        return Direction::South;
    } else if direction == Direction::West {
        return Direction::East;
    } else if direction == Direction::South {
        return Direction::North;
    } else if direction == Direction::East {
        return Direction::West;
    }
    return Direction::None;
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
