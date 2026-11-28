use std::{env, io};

use surreal_build::prelude::*;

/// Steam's redistributable for each system, relative to `godot/`.
const STEAM_LIBRARIES: [(System, &str); 3] = [
  (
    System::Linux,
    "../vendor/surreal/surreal-steam/libs/linux64/libsteam_api.so",
  ),
  (
    System::MacOs,
    "../vendor/surreal/surreal-steam/libs/osx/libsteam_api.dylib",
  ),
  (
    System::Windows(WindowsAbi::Msvc),
    "../vendor/surreal/surreal-steam/libs/win64/steam_api64.dll",
  ),
];

fn main() -> io::Result<()> {
  if env::var_os("CARGO_FEATURE_STEAM").is_some() {
    configure_steam()?;
  }

  // The file is checked in, so it lists Steam's libraries whether or not this
  // build links them.
  let mut file = GDExtensionFile::new()
    .with_configuration(Configuration {
      reloadable: false,
      ..Configuration::default()
    })
    .with_target_directory("../target")
    .with_icons(
      Icons::new("editor/icons")
        .with_icons_from_directory("../../godot/editor/icons")?
        .with_surreal_icons("../../godot/editor/icons", "surreal")?,
    );

  for (system, path) in STEAM_LIBRARIES {
    for mode in [Mode::Debug, Mode::Release] {
      file = file.with_dependency(
        Target::new(system, mode, Architecture::Generic),
        Dependency::new(system, path),
      );
    }
  }

  file.write("../../godot/game.gdextension")
}

/// Checks the app ID `surreal-steam` bakes in, and lets the extension find
/// Steam's library beside it on Linux.
fn configure_steam() -> io::Result<()> {
  println!("cargo:rerun-if-env-changed=STEAM_APP_ID");

  match env::var("STEAM_APP_ID") {
    Ok(app_id) if app_id.parse::<u32>().is_ok() => {}
    Ok(app_id) => panic!("STEAM_APP_ID must be a number, not {app_id:?}"),
    Err(_) => println!("cargo:warning=STEAM_APP_ID is unset; Steam reads steam_appid.txt instead"),
  }

  if env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "linux") {
    println!("cargo:rustc-link-arg-cdylib=-Wl,-rpath,$ORIGIN");
  }

  Ok(())
}
