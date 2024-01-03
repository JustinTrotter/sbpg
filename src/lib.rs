#![allow(clippy::type_complexity)]

mod actions;
mod loading;
mod menu;
mod player;
mod tile_map;

use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::tile_map::{
    cache_wall_locations, check_goal, move_player_from_input, setup,
    translate_grid_coords_entities, GoalBundle, LevelWalls, PlayerBundle, WallBundle,
};

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;


// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>().add_plugins((
            LoadingPlugin,
            MenuPlugin,
        ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins(FrameTimeDiagnosticsPlugin)
                .add_plugins(LdtkPlugin)
                .add_systems(OnEnter(GameState::Playing), setup)
                .insert_resource(LevelSelection::index(0))
                .register_ldtk_entity::<PlayerBundle>("Player")
                .register_ldtk_entity::<GoalBundle>("Goal")
                .add_systems(
                    Update,
                    (
                        move_player_from_input.run_if(in_state(GameState::Playing)),
                        translate_grid_coords_entities.run_if(in_state(GameState::Playing)),
                        cache_wall_locations.run_if(in_state(GameState::Playing)),
                        check_goal.run_if(in_state(GameState::Playing)),
                    ),
                )
                .register_ldtk_int_cell::<WallBundle>(1)
                .init_resource::<LevelWalls>();
        }
    }
}
