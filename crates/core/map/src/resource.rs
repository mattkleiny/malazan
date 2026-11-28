//! The map file as a Godot resource, read and written as a zip archive.

use std::io::{Read, Write};

use godot::{classes::file_access::ModeFlags, global::Error, prelude::*, tools::GFile};

use crate::{archive::MapContents, map::SEA_LEVEL};

surreal::resource_format! {
  loader: MapResourceLoader,
  saver: MapResourceSaver,
  extensions: ["map"],
  resource: MapResource,
  load: load_map,
  save: save_map,
}

/// A map file (.map): a map's definitions, location image, heightmap and
/// labels, packed into one archive.
#[derive(GodotClass)]
#[class(init, tool, base = Resource)]
pub struct MapResource {
  base: Base<Resource>,
  pub contents: Option<MapContents>, // Empty until read from a file.
}

fn load_map(path: &GString) -> Result<Gd<MapResource>, Error> {
  let mut file = GFile::open(path, ModeFlags::READ).map_err(|_| Error::ERR_FILE_CANT_OPEN)?;
  let mut bytes = Vec::new();

  file.read_to_end(&mut bytes).map_err(|_| Error::ERR_FILE_CANT_READ)?;

  let contents = MapContents::from_archive(&bytes, SEA_LEVEL.get()).map_err(|error| {
    godot_error!("MapResource: cannot read the map in {path}: {error}");
    Error::ERR_FILE_CORRUPT
  })?;

  let mut resource = MapResource::new_gd();

  resource.bind_mut().contents = Some(contents);

  Ok(resource)
}

fn save_map(resource: Gd<MapResource>, path: &GString) -> Error {
  let archive = {
    let resource = resource.bind();
    let Some(contents) = &resource.contents else {
      godot_error!("MapResource: no map to write to {path}");
      return Error::ERR_INVALID_DATA;
    };

    contents.to_archive()
  };

  let bytes = match archive {
    Ok(bytes) => bytes,
    Err(error) => {
      godot_error!("MapResource: cannot write the map to {path}: {error}");
      return Error::ERR_INVALID_DATA;
    }
  };

  let Ok(mut file) = GFile::open(path, ModeFlags::WRITE) else {
    return Error::ERR_FILE_CANT_WRITE;
  };

  if file.write_all(&bytes).is_err() {
    return Error::ERR_FILE_CANT_WRITE;
  }

  Error::OK
}
