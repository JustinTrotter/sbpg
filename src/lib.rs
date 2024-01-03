#![allow(clippy::type_complexity)]

mod actions;
mod loading;
mod menu;
mod player;
mod tile_map;

use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use player::PlayerPlugin;
use tile_map::TilemapPlugin;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Loading,
    Playing,
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_plugins((LdtkPlugin, LoadingPlugin, MenuPlugin, TilemapPlugin, PlayerPlugin));

        #[cfg(debug_assertions)]
        {
            app.add_plugins(FrameTimeDiagnosticsPlugin);
        }
    }
}
