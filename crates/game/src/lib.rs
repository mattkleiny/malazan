//! Godot bindings for the game: the extension's single entry point.

use godot::prelude::*;

extern crate game_common;
extern crate game_knowledge;
extern crate game_map;
extern crate game_terrain;

/// The entry point for the extension library.
struct GameLibrary;

#[gdextension]
unsafe impl ExtensionLibrary for GameLibrary {
  fn on_stage_init(stage: InitStage) {
    surreal::install_extensions(stage);
  }

  fn on_stage_deinit(stage: InitStage) {
    surreal::uninstall_extensions(stage);
  }

  fn on_main_loop_frame() {
    surreal::on_main_loop_frame();
  }
}
