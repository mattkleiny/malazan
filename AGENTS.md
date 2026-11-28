# AGENTS.md

- **Vocabulary**: [CONTEXT.md](CONTEXT.md) is the project's ubiquitous language: one line per term, grouped by section. Read it before exploring, and search the code by its terms.

## Keeping the domain model current

`CONTEXT.md` is part of the change, not follow-up work. A change is done when every domain concept it adds, renames or retires is reflected there.

A domain concept is anything a player or designer would name: a unit behaviour, order, resource, map feature, rule or state. When one appears, add one bullet under the section it belongs to: the bold term, one sentence on what it *is*, and `_Avoid_:` with the synonyms it replaces. Change it when the concept changes. When a section grows past what one line per term can carry, move its full entries (what it is and how it plays, plus the `_Avoid_:` line) into `docs/context/<section>.md`, link it under the section heading, and keep both in step from then on.

Keep the file to game language; type names, file paths and mechanics of the code belong in the code. When the code and `CONTEXT.md` name something differently, `CONTEXT.md` is canonical. Use its term in new code and report the divergence to the user, leaving renames of existing code for them to schedule.

## Project architecture

Core game logic lives in `crates/` as an ECS-based Rust workspace. `crates/common` is the root, `crates/core/*` holds the gameplay domains, and `crates/game` is the single entry point Godot loads:

- `surreal` (git submodule at `vendor/surreal`) - shared types and utilities used across crates; ECS, collections, reflection, Godot extensions, etc
- `surreal-macros` - proc macros supporting the ECS/codegen
- `surreal-build` - build-time helpers; generates `godot/game.gdextension`
- `game-common` (`crates/common`) - the root of the crate graph: the `FixedTick`/`VariableTick` events, the `SimulationClock`, and the few primitives every domain needs and none of them owns
- `game` (`crates/game`) - the GDExtension entry point exposing the game to Godot. It holds no gameplay

Godot (`godot/`) is the presentation layer: rendering, input, UI, and scenes. It talks to the simulation through the `game` GDExtension crate. Gameplay logic lives in `crates/`, while high-level flow, events, scripting and input can live in `godot/` as GDScript.

The editor icon folder is `godot/editor/icons/` (`res://editor/icons/`). Every GDScript node or resource type gets an `@icon` SVG there, and so does every Rust `GodotClass`. Icons are always named after the class in PascalCase (`KnowledgeTooltip.svg` for `knowledge_tooltip.gd`), never after the script file, whatever language the class is written in. This overrides any style guide that says snake_case. `crates/game/build.rs` maps each SVG in the folder to the class named by its file name.

Icons for surreal's classes live in surreal, in each crate's `icons/` folder, and are shared by every project on it. The build copies them into `godot/editor/icons/surreal/` and lists them in `godot/game.gdextension`; that subfolder is generated like the `.gdextension` file, so change an icon in `vendor/surreal`, never the copy, and commit the copies the build writes.

A new crate needs a `path` dependency **and** an `extern crate` line in `crates/game/src/lib.rs`, or it is never linked: its `GodotClass` types silently vanish from the extension and its system installer never runs. A new core crate goes in `crates/core/<domain>`, named `game-<domain>`; the workspace picks it up through the `crates/core/*` glob.

A crate's `pub` exports are the minimal set its dependent crates actually use, including what their tests use, and nothing more. A crate nobody depends on exports nothing at all: Godot reaches its classes through the extension, and `crates/game` only links it with `extern crate`. Keep modules private and re-export the items you need at the crate root rather than making modules `pub`. When a dependent stops using an item, take it out of the exports; when one needs a new item, export just that item.

## Working in this repo

`vendor/surreal` is a git submodule pointing at the private `mattkleiny/surreal-gdext`. Nothing compiles without it and `git submodule update --init` cannot authenticate on its own in a cloud session, so **both repositories have to be attached to the session** and the submodule's pinned commit placed at `vendor/surreal` before any Rust work starts. An empty `vendor/surreal` is the first thing to check when `cargo check` cannot find the `surreal` crate.

Commands, all from the repo root:

- `just test` (`cargo test --workspace`) - the suite. `surreal` sits inside the workspace directory, so it is a member and its tests run too.
- `just clippy` - lint; warnings fail.
- `cargo fmt --all` - two-space indent, 120 columns, grouped imports.
- `cargo check -p game` - also regenerates `godot/game.gdextension`.

`godot/game.gdextension` is **generated** by `crates/game/build.rs`. It is checked in, so it reads as hand-maintained but is not: run `cargo check -p game` and commit the regenerated file rather than editing it by hand.
