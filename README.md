# Game

A Godot game with its gameplay in Rust, built on [surreal](https://github.com/mattkleiny/surreal-gdext).

Gameplay lives in a Rust workspace under `crates/`; Godot (`godot/`) is the presentation layer. surreal is a git submodule at `vendor/surreal`.

```sh
git submodule update --init
just test
cargo build -p game
```

Then open `godot/project.godot` in Godot 4.

## Starting a new project from this template

1. Rename `game` / `game-common` in the crates' `Cargo.toml`, the `extern crate` line in `crates/game/src/lib.rs`, the `.gdextension` path in `crates/game/build.rs` and the `-p game` lines in `justfile`.
2. Set `config/name` in `godot/project.godot`.
3. Rename `Game` in `godot/export_presets.cfg` (export paths and the macOS bundle identifier), the package names in `.github/workflows/release.yml` and the devcontainer's name.
4. Run `cargo check -p game` to regenerate the `.gdextension` file, and update this README, `CONTEXT.md` and `AGENTS.md`.

## CI

`surreal` is private, so the workflows check it out with a `SURREAL_REPO_TOKEN` secret: a token with read access to `mattkleiny/surreal-gdext`.

- **Build and Test** (`.github/workflows/build.yml`) runs `just test`. It only runs by hand until you add a `push` trigger.
- **Release** (`.github/workflows/release.yml`) builds the GDExtension on Linux, Windows and macOS, then exports every preset in `godot/export_presets.cfg` from one Linux job. Set its default Godot version to the one the project uses.

## Releasing

1. Tag the commit to release `v0.1.0` and push the tag.
2. The Release workflow exports every platform, publishes a GitHub Release, and uploads the build to Steam, live on the `beta` branch, once Steam is set up.
3. In Steamworks, promote the build to the default branch.

Manual runs of the workflow upload to Steam only when given a `steam_branch`.

### Steam setup

Repository variables (Settings → Secrets and variables → Actions → Variables):

- `STEAM_APP_ID` - the app; tags upload to Steam once this is set. It is not baked into the build: the GitHub Release and the depots ship the same libraries, which connect to Steam when it launches them.
- `STEAM_FIRST_DEPOT_ID` - optional; the Linux depot, with Windows and macOS the next two IDs. Defaults to the app ID plus one. Limit each depot to its OS in Steamworks.
- `STEAM_BRANCH` - the branch tags set live on (default `beta`). Never `default`: Steam does not let a build account set it live.

Secrets, used by [game-ci/steam-deploy](https://github.com/game-ci/steam-deploy):

- `STEAM_USERNAME`, `STEAM_PASSWORD` - a build account with only *Edit App Metadata* and *Publish App Changes To Steam*.
- `STEAM_SHARED_SECRET` - the `shared_secret` of the build account's Steam Guard authenticator, so CI makes its own codes. Set one up with [steamguard-cli](https://github.com/dyc3/steamguard-cli) (`steamguard setup`), then `jq -r .shared_secret ~/.config/steamguard-cli/maFiles/<steamid>.maFile | gh secret set STEAM_SHARED_SECRET`. Keep its revocation code.
- `STEAM_PUBLISHER_KEY` - optional; a Web API publisher key, needed only to set a build live on the `default` branch from a manual run.

Launch options in Steamworks: `Game.x86_64` (Linux), `Game.exe` (Windows), `Game.app` (macOS).
