use std::path::{PathBuf, MAIN_SEPARATOR_STR};

use case::CaseExt;

use crate::{
    error::{Error, ErrorKind, Result},
    utils::registry,
};

pub fn get_launcher_executable() -> Result<PathBuf> {
    let launcher_executable = registry::get_current_user_reg_key("Valve\\Steam")
        .and_then(|user_launcher_reg| registry::get_value(&user_launcher_reg, "SteamExe"))
        .map(PathBuf::from)
        .map(fix_launcher_executable_path)
        .map_err(|error| {
            Error::new(
                ErrorKind::LauncherNotFound,
                format!("Invalid Steam path, maybe this launcher is not installed: {error}"),
            )
        })?;

    if !launcher_executable.exists() {
        return Err(Error::new(
            ErrorKind::LauncherNotFound,
            format!(
                "Invalid Steam path, maybe this launcher is not installed: {}",
                launcher_executable.display()
            ),
        ));
    }

    Ok(launcher_executable)
}

pub fn get_manifests_path() -> Result<PathBuf> {
    let manifests_path = registry::get_local_machine_reg_key("Valve\\Steam")
        .and_then(|launcher_reg| registry::get_value(&launcher_reg, "InstallPath"))
        .map(PathBuf::from)
        .map(|path| path.join("steamapps"))
        .map_err(|error| {
            Error::new(
                ErrorKind::LauncherNotFound,
                format!("Invalid Steam path, maybe this launcher is not installed: {error}"),
            )
        })?;

    if !manifests_path.exists() {
        return Err(Error::new(
            ErrorKind::LauncherNotFound,
            format!(
                "Invalid Steam path, maybe this launcher is not installed: {}",
                manifests_path.display()
            ),
        ));
    }

    Ok(manifests_path)
}

#[allow(clippy::needless_pass_by_value)]
pub fn fix_launcher_executable_path(path: PathBuf) -> PathBuf {
    let path_string = path.display().to_string();
    let words = path_string.split('/').collect::<Vec<_>>();
    let mut result_path = PathBuf::new();

    for word in words {
        let mut new_word = String::from(word);

        if new_word.contains(':') {
            new_word = new_word.to_camel();
            new_word.push_str(MAIN_SEPARATOR_STR);
        } else if new_word.contains("x86") {
            new_word = new_word
                .split(' ')
                .collect::<Vec<_>>()
                .into_iter()
                .map(|value| {
                    if value.contains("86") {
                        String::from(value)
                    } else {
                        value.to_camel()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
        } else if !new_word.contains(".exe") {
            new_word = new_word.to_camel();
        }

        result_path = result_path.join(new_word);
    }

    result_path
}
