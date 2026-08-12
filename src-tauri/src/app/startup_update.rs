use crate::commands::updater;
use std::ffi::OsStr;
use tauri::AppHandle;

const SKIP_UPDATE_ARG: &str = "--skip-update";

fn has_skip_update_arg<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    args.into_iter().any(|arg| arg.as_ref() == OsStr::new(SKIP_UPDATE_ARG))
}

fn should_skip_update() -> bool {
    has_skip_update_arg(std::env::args_os())
}

/// Runs the automatic update flow only after the main window is fully ready.
/// Updating is intentionally not part of the startup readiness gate.
pub fn start_background(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if should_skip_update() {
            log::info!("Skipping background update flow because {SKIP_UPDATE_ARG} was provided");
            return;
        }

        log::info!("Starting background update check after main window became ready");
        let updates_available = match updater::check_updates(app_handle.clone()).await {
            Ok(result) => result.has_updates,
            Err(error) => {
                log::error!("Background update check failed: {error}");
                return;
            }
        };

        if !updates_available {
            log::info!("Background update check completed; no update is available");
            return;
        }

        match updater::download_updates(app_handle.clone(), None).await {
            Ok(download_result) if download_result.success_count > 0 => {
                if let Err(error) = updater::start_update_install(app_handle).await {
                    log::error!("Background update installation start failed: {error}");
                }
            }
            Ok(_) => {
                log::warn!(
                    "Background update download failed; continuing with the current version"
                );
            }
            Err(error) => {
                log::error!("Background update download failed: {error}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::has_skip_update_arg;

    #[test]
    fn detects_skip_update_argument() {
        assert!(has_skip_update_arg(["simprint.exe", "--skip-update"]));
    }

    #[test]
    fn does_not_treat_other_arguments_as_skip_update() {
        assert!(!has_skip_update_arg([
            "simprint.exe",
            "--skip-update-check",
            "simprint://open"
        ]));
    }
}
