use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use zed_extension_api::{Architecture, Os};

/// The release-pinned language-server version used by the Zed extension.
pub const PINNED_SERVER_VERSION: &str = "0.1.0";

/// The GitHub repository that publishes native server releases.
pub const REPO: &str = "fits-ki/stellarium";

const LOCAL_PATH_ENV: &str = "STELLARIUM_LSP_PATH";

// Release-pinned SHA-256 values are produced by the release pipeline for each
// supported target. The extension refuses to execute a server binary whose
// digest is not present here.
const PINNED_CHECKSUMS: &[(&str, &str)] = &[
    (
        "aarch64-apple-darwin",
        "25b84a0188095712a313071fb68cf88c4dcc0ecd14155f9fe673749ebf504966",
    ),
    (
        "x86_64-apple-darwin",
        "be53639181ae39e320c960b3bb864101414427a7d57286d591d1384cd5f1e2b9",
    ),
    (
        "aarch64-unknown-linux-gnu",
        "5fe362e3a3fe5b9a9cf0b21ac5e12939ac8ccf99d930d5c3ac9948d74de94aff",
    ),
    (
        "x86_64-unknown-linux-gnu",
        "93c576908392966bebed8328951139c1ade990aacd24f6d3eee6656452f6c0dc",
    ),
    (
        "aarch64-pc-windows-msvc",
        "c58e10038b231f97866b4c4115eb60236fd87a27093ff87a3e85e7ad5c7b7421",
    ),
    (
        "x86_64-pc-windows-msvc",
        "eb961d14572a012c73312b3f1fbc9ec9e2936e036201e803e7aaca9d33076b45",
    ),
];

/// Returns the pinned SHA-256 for the given Rust target triple, if one exists.
pub fn pinned_checksum(target: &str) -> Option<&'static str> {
    PINNED_CHECKSUMS
        .iter()
        .find(|(t, _)| *t == target)
        .map(|(_, c)| *c)
}

/// Returns the value of an environment variable from a Zed-provided key/value
/// slice, or `None` if it is absent or not valid UTF-8.
pub fn env_value(env: &[(String, String)], key: &str) -> Option<String> {
    env.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

/// Maps Zed's runtime platform to the Rust target triple used to name release
/// artifacts. Unsupported platforms return an error.
pub fn target_triple(os: Os, arch: Architecture) -> Result<String, String> {
    let arch = match arch {
        Architecture::Aarch64 => "aarch64",
        Architecture::X8664 => "x86_64",
        Architecture::X86 => {
            return Err("32-bit x86 is not a supported Stellarium LSP target".into())
        }
    };

    let vendor_os = match os {
        Os::Mac => "apple-darwin",
        Os::Linux => "unknown-linux-gnu",
        Os::Windows => "pc-windows-msvc",
    };

    Ok(format!("{}-{}", arch, vendor_os))
}

fn exe_suffix(os: Os) -> &'static str {
    if os == Os::Windows {
        ".exe"
    } else {
        ""
    }
}

fn cargo_binary_name(os: Os) -> String {
    format!("stellarium-lsp{}", exe_suffix(os))
}

/// Returns the name of the release asset for the given platform. The asset is
/// treated as an uncompressed executable.
pub fn asset_name(os: Os, arch: Architecture) -> Result<String, String> {
    let triple = target_triple(os, arch)?;
    Ok(format!("stellarium-lsp-{}{}", triple, exe_suffix(os)))
}

/// Returns the local binary name inside a versioned cache directory.
pub fn binary_name(target: &str, os: Os) -> String {
    format!("stellarium-lsp-{}{}", target, exe_suffix(os))
}

/// Returns the versioned cache directory for a server release and target.
pub fn versioned_dir(work_dir: &Path, version: &str, target: &str) -> PathBuf {
    work_dir.join("stellarium-lsp").join(version).join(target)
}

/// Computes the lower-case hex SHA-256 digest of the file at `path`.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|e| format!("failed to open {}: {}", path.display(), e))?;

    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Verifies that `path` has the expected SHA-256 digest.
pub fn verify_checksum(path: &Path, expected: &str) -> Result<(), String> {
    let actual = sha256_file(path)?;
    if actual != expected {
        return Err(format!(
            "checksum mismatch for {}: expected {expected}, got {actual}",
            path.display()
        ));
    }
    Ok(())
}

/// Resolves a verified language-server binary path.
///
/// Search order:
/// 1. `STELLARIUM_LSP_PATH`, explicitly supplied by the developer.
/// 2. A previously downloaded binary in the versioned work directory whose
///    digest matches the pinned checksum for the current target.
/// 3. `Ok(None)` if nothing is installed yet.
///
/// The local override is intentionally not opened from the extension. Zed runs
/// extensions in a WASM sandbox that cannot read arbitrary workspace paths,
/// while the host process can still launch the returned command path.
pub fn find_server_path(
    env: &[(String, String)],
    platform: (Os, Architecture),
    work_dir: &Path,
) -> Result<Option<PathBuf>, String> {
    let (os, arch) = platform;
    let target = target_triple(os, arch)?;

    if let Some(local_path) = env_value(env, LOCAL_PATH_ENV) {
        if !local_path.trim().is_empty() {
            return Ok(Some(PathBuf::from(local_path)));
        }
    }

    let cached =
        versioned_dir(work_dir, PINNED_SERVER_VERSION, &target).join(binary_name(&target, os));

    if cached.exists() {
        let expected =
            pinned_checksum(&target).ok_or_else(|| format!("no pinned checksum for {target}"))?;
        verify_checksum(&cached, expected)?;
        return Ok(Some(cached));
    }

    Ok(None)
}

/// Resolves a verified language-server binary built by the local workspace.
///
/// This supports Zed dev-extension installs before a release tag exists. The
/// binary is still accepted only when it matches the release-pinned checksum.
pub fn find_workspace_server_path(
    worktree_root: &Path,
    platform: (Os, Architecture),
) -> Result<Option<PathBuf>, String> {
    let (os, arch) = platform;
    let target = target_triple(os, arch)?;
    let expected =
        pinned_checksum(&target).ok_or_else(|| format!("no pinned checksum for {target}"))?;

    find_workspace_server_path_with_checksum(worktree_root, &target, os, expected)
}

fn find_workspace_server_path_with_checksum(
    worktree_root: &Path,
    target: &str,
    os: Os,
    expected: &str,
) -> Result<Option<PathBuf>, String> {
    for root in worktree_root.ancestors() {
        let packaged_artifact = root.join("server").join(binary_name(&target, os));
        if packaged_artifact.exists() {
            verify_checksum(&packaged_artifact, expected)?;
            return Ok(Some(packaged_artifact));
        }

        let release_artifact = root
            .join("release")
            .join("artifacts")
            .join(binary_name(&target, os));
        if release_artifact.exists() {
            verify_checksum(&release_artifact, expected)?;
            return Ok(Some(release_artifact));
        }

        let cargo_artifact = root
            .join("target")
            .join(&target)
            .join("release")
            .join(cargo_binary_name(os));

        if cargo_artifact.exists() {
            verify_checksum(&cargo_artifact, expected)?;
            return Ok(Some(cargo_artifact));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Digest;
    use std::io::Write;

    fn sha(content: &[u8]) -> String {
        hex::encode(Sha256::digest(content))
    }

    fn temp_file(name: &str, content: &[u8]) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "stellarium-zed-{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn target_triple_maps_mac_aarch64() {
        assert_eq!(
            target_triple(Os::Mac, Architecture::Aarch64).unwrap(),
            "aarch64-apple-darwin"
        );
    }

    #[test]
    fn target_triple_maps_linux_x8664() {
        assert_eq!(
            target_triple(Os::Linux, Architecture::X8664).unwrap(),
            "x86_64-unknown-linux-gnu"
        );
    }

    #[test]
    fn target_triple_maps_windows_aarch64() {
        assert_eq!(
            target_triple(Os::Windows, Architecture::Aarch64).unwrap(),
            "aarch64-pc-windows-msvc"
        );
    }

    #[test]
    fn target_triple_rejects_x86() {
        assert!(target_triple(Os::Linux, Architecture::X86).is_err());
    }

    #[test]
    fn asset_name_uses_exe_suffix_on_windows() {
        assert_eq!(
            asset_name(Os::Windows, Architecture::X8664).unwrap(),
            "stellarium-lsp-x86_64-pc-windows-msvc.exe"
        );
        assert_eq!(
            asset_name(Os::Windows, Architecture::Aarch64).unwrap(),
            "stellarium-lsp-aarch64-pc-windows-msvc.exe"
        );
    }

    #[test]
    fn asset_name_formats_correctly_for_all_platforms() {
        assert_eq!(
            asset_name(Os::Mac, Architecture::Aarch64).unwrap(),
            "stellarium-lsp-aarch64-apple-darwin"
        );
        assert_eq!(
            asset_name(Os::Mac, Architecture::X8664).unwrap(),
            "stellarium-lsp-x86_64-apple-darwin"
        );
        assert_eq!(
            asset_name(Os::Linux, Architecture::Aarch64).unwrap(),
            "stellarium-lsp-aarch64-unknown-linux-gnu"
        );
        assert_eq!(
            asset_name(Os::Linux, Architecture::X8664).unwrap(),
            "stellarium-lsp-x86_64-unknown-linux-gnu"
        );
    }

    #[test]
    fn sha256_file_matches_independent_digest() {
        let content = b"hello, stellarium";
        let path = temp_file("sha", content);
        let expected = sha(content);
        assert_eq!(sha256_file(&path).unwrap(), expected);
    }

    #[test]
    fn verify_checksum_accepts_match() {
        let content = b"verified";
        let path = temp_file("ok", content);
        let expected = sha(content);
        verify_checksum(&path, &expected).unwrap();
    }

    #[test]
    fn verify_checksum_rejects_mismatch() {
        let content = b"tampered";
        let path = temp_file("bad", content);
        assert!(verify_checksum(&path, "0000").is_err());
    }

    #[test]
    fn find_server_path_prefers_env_override() {
        let path = temp_file("env-lsp", b"env-override-binary");
        let env = [(
            LOCAL_PATH_ENV.to_string(),
            path.to_string_lossy().into_owned(),
        )];
        let work_dir = std::env::temp_dir().join(format!("stellarium-work-{}", std::process::id()));
        let found = find_server_path(&env, (Os::Mac, Architecture::Aarch64), &work_dir)
            .unwrap()
            .unwrap();
        assert_eq!(found, path);
    }

    #[test]
    fn local_env_override_does_not_require_wasm_filesystem_access() {
        let local = PathBuf::from("/outside-the-extension-wasm-sandbox/stellarium-lsp");
        let env = [(
            LOCAL_PATH_ENV.to_string(),
            local.to_string_lossy().into_owned(),
        )];

        let found = find_server_path(
            &env,
            (Os::Mac, Architecture::Aarch64),
            Path::new("/extension-work-dir"),
        )
        .unwrap()
        .unwrap();

        assert_eq!(found, local);
    }

    #[test]
    fn find_server_path_returns_none_when_not_installed() {
        let work_dir = std::env::temp_dir().join(format!(
            "stellarium-work-notinstalled-{}",
            std::process::id()
        ));
        fs::remove_dir_all(&work_dir).ok();
        assert_eq!(
            find_server_path(&[], (Os::Linux, Architecture::X8664), &work_dir).unwrap(),
            None
        );
    }

    #[test]
    fn find_workspace_server_path_walks_up_from_nested_worktree_root() {
        let repo = std::env::temp_dir().join(format!(
            "stellarium-workspace-server-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let nested = repo.join("scripts");
        let artifact = repo
            .join("target")
            .join("aarch64-apple-darwin")
            .join("release")
            .join("stellarium-lsp");

        fs::create_dir_all(artifact.parent().unwrap()).unwrap();
        fs::create_dir_all(&nested).unwrap();
        let content = b"standalone cargo artifact";
        fs::write(&artifact, content).unwrap();

        let found = find_workspace_server_path_with_checksum(
            &nested,
            "aarch64-apple-darwin",
            Os::Mac,
            &sha(content),
        )
        .unwrap()
        .unwrap();
        assert_eq!(found, artifact);

        fs::remove_dir_all(repo).ok();
    }

    #[test]
    fn find_workspace_server_path_walks_up_to_release_artifacts() {
        let repo = std::env::temp_dir().join(format!(
            "stellarium-release-artifact-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let nested = repo.join("extensions").join("zed");
        let artifact = repo
            .join("release")
            .join("artifacts")
            .join("stellarium-lsp-aarch64-apple-darwin");

        fs::create_dir_all(artifact.parent().unwrap()).unwrap();
        fs::create_dir_all(&nested).unwrap();
        let content = b"standalone release artifact";
        fs::write(&artifact, content).unwrap();

        let found = find_workspace_server_path_with_checksum(
            &nested,
            "aarch64-apple-darwin",
            Os::Mac,
            &sha(content),
        )
        .unwrap()
        .unwrap();
        assert_eq!(found, artifact);

        fs::remove_dir_all(repo).ok();
    }

    #[test]
    fn find_workspace_server_path_finds_packaged_extension_artifact() {
        let extension = std::env::temp_dir().join(format!(
            "stellarium-packaged-server-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let artifact = extension
            .join("server")
            .join("stellarium-lsp-aarch64-apple-darwin");

        fs::create_dir_all(artifact.parent().unwrap()).unwrap();
        let content = b"standalone packaged artifact";
        fs::write(&artifact, content).unwrap();

        let found = find_workspace_server_path_with_checksum(
            &extension,
            "aarch64-apple-darwin",
            Os::Mac,
            &sha(content),
        )
        .unwrap()
        .unwrap();
        assert_eq!(found, artifact);

        fs::remove_dir_all(extension).ok();
    }
}
