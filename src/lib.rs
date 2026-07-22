use std::fs;
use std::path::Path;

use zed_extension_api::{self as zed};

mod server;

struct StellariumExtension;

fn development_server_command(
    manifest_path: Option<&str>,
    cargo_path: Option<String>,
) -> zed::Result<Option<zed::Command>> {
    let Some(manifest_path) = manifest_path else {
        return Ok(None);
    };
    let cargo_path = cargo_path.ok_or_else(|| {
        "local Stellarium extension development requires `cargo` on Zed's PATH".to_string()
    })?;

    Ok(Some(zed::Command {
        command: cargo_path,
        args: vec![
            "run".to_string(),
            "--quiet".to_string(),
            "--manifest-path".to_string(),
            manifest_path.to_string(),
            "--package".to_string(),
            "stellarium-lsp".to_string(),
            "--".to_string(),
        ],
        env: vec![],
    }))
}

fn report_installation_failure(
    language_server_id: &zed::LanguageServerId,
    message: &str,
) -> String {
    zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::Failed(message.to_string()),
    );
    message.to_string()
}

impl zed::Extension for StellariumExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        if let Some(command) = development_server_command(
            option_env!("STELLARIUM_ZED_LOCAL_DEV_MANIFEST"),
            worktree.which("cargo"),
        )? {
            return Ok(command);
        }

        let (os, arch) = zed::current_platform();
        let env = worktree.shell_env();
        let worktree_root = worktree.root_path();
        let work_dir = std::env::current_dir().map_err(|e| e.to_string())?;

        if let Some(path) = server::find_server_path(&env, (os, arch), &work_dir)? {
            return Ok(zed::Command {
                command: path.to_string_lossy().into_owned(),
                args: vec![],
                env: vec![],
            });
        }

        if let Some(path) =
            server::find_workspace_server_path(Path::new(&worktree_root), (os, arch))?
        {
            return Ok(zed::Command {
                command: path.to_string_lossy().into_owned(),
                args: vec![],
                env: vec![],
            });
        }

        let target = server::target_triple(os, arch)?;
        let checksum = server::pinned_checksum(&target)
            .ok_or_else(|| format!("no pinned checksum for {target}"))?;

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );

        let asset_name = server::asset_name(os, arch)?;
        let release = zed::github_release_by_tag_name(
            server::REPO,
            &format!("v{}", server::PINNED_SERVER_VERSION),
        )?;
        let asset = release
            .assets
            .into_iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("release {} has no asset {}", release.version, asset_name))?;

        let cached = server::versioned_dir(&work_dir, server::PINNED_SERVER_VERSION, &target)
            .join(server::binary_name(&target, os));

        let relative = cached
            .strip_prefix(&work_dir)
            .unwrap_or(&cached)
            .to_string_lossy()
            .into_owned();

        if let Err(e) = fs::create_dir_all(cached.parent().unwrap_or(Path::new(""))) {
            return Err(report_installation_failure(
                language_server_id,
                &e.to_string(),
            ));
        }

        if let Err(e) = zed::download_file(
            &asset.download_url,
            &relative,
            zed::DownloadedFileType::Uncompressed,
        ) {
            let _ = fs::remove_file(&cached);
            return Err(report_installation_failure(language_server_id, &e));
        }

        if let Err(e) = zed::make_file_executable(&relative) {
            return Err(report_installation_failure(language_server_id, &e));
        }

        let actual = server::sha256_file(&cached)?;
        if actual != checksum {
            let _ = fs::remove_file(&cached);
            let msg =
                format!("checksum mismatch for {asset_name}: expected {checksum}, got {actual}");
            return Err(report_installation_failure(language_server_id, &msg));
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );

        Ok(zed::Command {
            command: cached.to_string_lossy().into_owned(),
            args: vec![],
            env: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zed::Extension;

    #[test]
    fn can_instantiate_extension() {
        let _ = StellariumExtension::new();
    }

    #[test]
    fn development_command_runs_workspace_lsp_through_cargo() {
        let command = development_server_command(
            Some("/repo/Cargo.toml"),
            Some("/toolchain/bin/cargo".to_string()),
        )
        .unwrap()
        .expect("development command");

        assert_eq!(command.command, "/toolchain/bin/cargo");
        assert_eq!(
            command.args,
            [
                "run",
                "--quiet",
                "--manifest-path",
                "/repo/Cargo.toml",
                "--package",
                "stellarium-lsp",
                "--",
            ]
        );
    }

    #[test]
    fn release_build_does_not_select_the_cargo_development_command() {
        assert!(development_server_command(None, Some("cargo".to_string()))
            .unwrap()
            .is_none());
    }
}

zed::register_extension!(StellariumExtension);
