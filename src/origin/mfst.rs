use std::path::{Path, PathBuf};

use crate::{
    error::{Error, ErrorKind, Result},
    prelude::{Game, GameCommands, GameState, GameType},
};

#[derive(Default)]
struct Manifest {
    id: String,
    dipinstallpath: String,
    previousstate: String,
    currentstate: String,
    downloading: bool,
    paused: bool,
    totaldownloadbytes: u64,
    totalbytes: u64,
    savedbytes: u64,
}

pub fn read(file: &Path, launcher_executable: &Path) -> Result<Game> {
    let manifest_data = std::fs::read_to_string(file).map_err(|error| {
        Error::new(
            ErrorKind::InvalidManifest,
            format!("Invalid Origin manifest: {} {}", file.display(), error),
        )
    })?;

    let manifest = String::from("http://localhost/") + &manifest_data;

    let manifest_url = url::Url::parse(&manifest).map_err(|error| {
        Error::new(
            ErrorKind::InvalidManifest,
            format!(
                "Error on read the Origin manifest: {} {}",
                file.display(),
                error
            ),
        )
    })?;

    let manifest_entries = manifest_url
        .query_pairs()
        .map(|(attr, value)| (attr.to_string(), value.to_string()))
        .collect::<Vec<_>>();

    let mut manifest = Manifest::default();

    for (attr, value) in manifest_entries {
        match attr.as_str() {
            "id" => {
                manifest.id = value;
            }
            "dipinstallpath" => {
                manifest.dipinstallpath =
                    make_dip_install_path(&value).map_or(String::new(), |value| value);
            }
            "currentstate" => {
                manifest.currentstate = value;
            }
            "previousstate" => {
                manifest.previousstate = value;
            }
            "totaldownloadbytes" => {
                manifest.totaldownloadbytes = value.parse::<u64>().unwrap();
            }
            "totalbytes" => {
                manifest.totalbytes = value.parse::<u64>().unwrap();
            }
            "savedbytes" => {
                manifest.savedbytes = value.parse::<u64>().unwrap();
            }
            "downloading" => {
                manifest.downloading = value == "1";
            }
            "paused" => {
                manifest.paused = value == "1";
            }
            _ => {}
        }
    }

    let name = get_game_name(file).unwrap_or_else(|| String::from("Unknown"));

    Ok(Game {
        type_: GameType::Origin.to_string(),
        id: manifest.id.clone(),
        name,
        path: Some(PathBuf::from(manifest.dipinstallpath)),
        commands: GameCommands {
            install:   Some(vec![
                launcher_executable.display().to_string(),
                format!("origin2://game/download?offerId={}", &manifest.id),
            ]),
            launch:    Some(vec![
                launcher_executable.display().to_string(),
                format!("origin2://game/launch?offerIds={}", &manifest.id),
            ]),
            uninstall: None,
        },
        state: GameState {
            installed:      true,
            needs_update:   (manifest.currentstate == "kTransferring"
                || manifest.currentstate == "kEnqueued"),
            downloading:    manifest.currentstate == "kTransferring",
            total_bytes:    Some(manifest.totalbytes),
            received_bytes: Some(manifest.savedbytes),
        },
    })
}

fn make_dip_install_path(value: &str) -> Option<String> {
    let separator = std::path::MAIN_SEPARATOR.to_string();

    return Option::from(value)
        .map(|path| path.replace("%5c", &separator))
        .map(|path| path.replace("%5C", &separator))
        .map(|path| path.replace("%2f", &separator))
        .map(|path| path.replace("%2F", &separator))
        .map(|path| path.replace("%3a", ":"))
        .map(|path| path.replace("%3A", ":"))
        .map(|path| path.replace("%20", " "))
        .map(PathBuf::from)
        .map(|path| path.display().to_string());
}

fn get_game_name(file: &Path) -> Option<String> {
    file.parent()
        .and_then(|path| path.file_name())
        .and_then(|path| path.to_str())
        .map(std::string::ToString::to_string)
}
