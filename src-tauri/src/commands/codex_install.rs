//! App-owned macOS Codex installation from verified official GitHub assets.

use futures::StreamExt;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const RELEASE_API: &str = "https://api.github.com/repos/openai/codex/releases/latest";
const MAX_METADATA_BYTES: usize = 2 * 1024 * 1024;
const MAX_ARCHIVE_BYTES: usize = 128 * 1024 * 1024;
const MAX_EXECUTABLE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Clone, Deserialize)]
struct Asset {
    id: u64,
    url: String,
    name: String,
    browser_download_url: String,
    digest: Option<String>,
    size: u64,
}

struct SelectedAsset {
    asset: Asset,
    member: String,
    version: String,
}

pub(super) fn bin_dir() -> PathBuf {
    crate::config::get_app_config_dir().join("bin")
}

pub(super) fn executable() -> PathBuf {
    bin_dir().join("codex")
}

fn select_asset(release: Release, arch: &str) -> Result<SelectedAsset, String> {
    let member = match arch {
        "aarch64" => "codex-aarch64-apple-darwin",
        "x86_64" => "codex-x86_64-apple-darwin",
        _ => return Err("This Mac architecture is not supported".into()),
    };
    let version = release
        .tag_name
        .strip_prefix("rust-v")
        .filter(|v| v.bytes().all(|c| c.is_ascii_digit() || c == b'.'))
        .filter(|v| super::parse_semver(v).is_some())
        .ok_or_else(|| "Official Codex release metadata is invalid".to_string())?
        .to_string();
    if release.draft || release.prerelease {
        return Err("Official Codex release is not stable".into());
    }
    let name = format!("{member}.tar.gz");
    let mut matches = release
        .assets
        .into_iter()
        .filter(|asset| asset.name == name);
    let asset = matches
        .next()
        .ok_or_else(|| "Official Codex release has no asset for this Mac".to_string())?;
    if matches.next().is_some()
        || asset.id == 0
        || asset.url
            != format!(
                "https://api.github.com/repos/openai/codex/releases/assets/{}",
                asset.id
            )
        || asset.browser_download_url
            != format!(
                "https://github.com/openai/codex/releases/download/{}/{name}",
                release.tag_name
            )
        || asset.size == 0
        || asset.size > MAX_ARCHIVE_BYTES as u64
        || asset
            .digest
            .as_deref()
            .and_then(|s| s.strip_prefix("sha256:"))
            .is_none_or(|s| s.len() != 64 || !s.bytes().all(|c| c.is_ascii_hexdigit()))
    {
        return Err("Official Codex asset metadata is invalid".into());
    }
    Ok(SelectedAsset {
        asset,
        member: member.into(),
        version,
    })
}

fn verify_digest(bytes: &[u8], digest: &str) -> Result<(), String> {
    let expected = digest
        .strip_prefix("sha256:")
        .filter(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
        .ok_or_else(|| "Official Codex SHA256 digest is missing or invalid".to_string())?;
    let actual = format!("{:x}", Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(
            "Codex download failed SHA256 verification; existing installation was preserved".into(),
        );
    }
    Ok(())
}

async fn bounded_body(response: reqwest::Response, limit: usize) -> Result<Vec<u8>, String> {
    if !response.status().is_success()
        || response.content_length().is_some_and(|n| n > limit as u64)
    {
        return Err("Official Codex download was rejected or exceeded its size limit".into());
    }
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| "Official Codex download was interrupted".to_string())?;
        if bytes.len().saturating_add(chunk.len()) > limit {
            return Err("Official Codex download exceeded its size limit".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

pub(super) async fn install(destination: PathBuf) -> Result<(), String> {
    let metadata_client = reqwest::Client::builder()
        .user_agent("bit2-switch-native-installer")
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Unable to initialize Codex installer".to_string())?;
    let response = metadata_client
        .get(RELEASE_API)
        .send()
        .await
        .map_err(|_| "Unable to fetch official Codex release metadata".to_string())?;
    let metadata = bounded_body(response, MAX_METADATA_BYTES).await?;
    let release = serde_json::from_slice(&metadata)
        .map_err(|_| "Official Codex release metadata is invalid".to_string())?;
    let selected = select_asset(release, std::env::consts::ARCH)?;
    let download_client = reqwest::Client::builder()
        .user_agent("bit2-switch-native-installer")
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(240))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            let url = attempt.url();
            if attempt.previous().len() >= 4
                || url.scheme() != "https"
                || !url.username().is_empty()
                || url.password().is_some()
                || url.port().is_some()
                || !matches!(
                    url.host_str(),
                    Some("api.github.com")
                        | Some("github.com")
                        | Some("release-assets.githubusercontent.com")
                )
            {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|_| "Unable to initialize Codex downloader".to_string())?;
    let response = download_client
        .get(format!(
            "https://api.github.com/repos/openai/codex/releases/assets/{}",
            selected.asset.id
        ))
        .header(reqwest::header::ACCEPT, "application/octet-stream")
        .send()
        .await
        .map_err(|_| "Unable to download the official Codex executable".to_string())?;
    let bytes = bounded_body(response, MAX_ARCHIVE_BYTES).await?;
    tokio::task::spawn_blocking(move || install_archive(&destination, &selected, &bytes))
        .await
        .map_err(|_| "Codex installer task failed".to_string())?
}

struct StageDir(PathBuf);
impl Drop for StageDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The archive is never unpacked into the filesystem: only the exact expected
/// member is streamed to a fresh regular file. Every failure precedes rename.
fn install_archive(bin: &Path, selected: &SelectedAsset, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() as u64 != selected.asset.size {
        return Err("Codex download size does not match official metadata".into());
    }
    verify_digest(bytes, selected.asset.digest.as_deref().unwrap_or(""))?;
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(bin)
        .map_err(|_| "Unable to create the app-owned Codex directory".to_string())?;
    std::fs::set_permissions(bin, std::fs::Permissions::from_mode(0o700))
        .map_err(|_| "Unable to secure the app-owned Codex directory".to_string())?;
    let stage = StageDir(bin.join(format!(".codex-install-{}", uuid::Uuid::new_v4())));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&stage.0)
        .map_err(|_| "Unable to prepare Codex installation".to_string())?;
    let archive = stage.0.join("official.tar.gz");
    std::fs::write(&archive, bytes).map_err(|_| "Unable to stage Codex download".to_string())?;
    let candidate = stage.0.join("codex");
    let mut extraction = Command::new("/usr/bin/tar");
    extraction
        .arg("-xOzf")
        .arg(&archive)
        .arg("--")
        .arg(&selected.member);
    run_bounded_to_file(
        &mut extraction,
        &candidate,
        MAX_EXECUTABLE_BYTES,
        Duration::from_secs(30),
    )?;
    std::fs::set_permissions(&candidate, std::fs::Permissions::from_mode(0o700))
        .map_err(|_| "Unable to prepare the Codex executable".to_string())?;
    let mut probe = Command::new(&candidate);
    probe
        .arg("--version")
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin");
    let version_output = stage.0.join("version.txt");
    run_bounded_to_file(&mut probe, &version_output, 4096, Duration::from_secs(10))?;
    let output = std::fs::read_to_string(&version_output)
        .map_err(|_| "Downloaded Codex executable returned an invalid version".to_string())?;
    if output.trim() != format!("codex-cli {}", selected.version) {
        return Err("Downloaded Codex version does not match the official release".into());
    }
    std::fs::File::open(&candidate)
        .and_then(|file| file.sync_all())
        .map_err(|_| "Unable to persist the verified Codex executable".to_string())?;
    std::fs::rename(&candidate, bin.join("codex")).map_err(|_| {
        "Unable to atomically install Codex; existing installation was preserved".to_string()
    })?;
    Ok(())
}

fn run_bounded_to_file(
    command: &mut Command,
    output: &Path,
    max_bytes: u64,
    timeout: Duration,
) -> Result<(), String> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(output)
        .map_err(|_| "Unable to create Codex verification output".to_string())?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(file))
        .stderr(Stdio::null());
    super::isolate_child_process_group(command);
    let mut child = command
        .spawn()
        .map_err(|_| "Unable to run Codex extraction or verification".to_string())?;
    let started = Instant::now();
    loop {
        let size = std::fs::metadata(output)
            .map(|m| m.len())
            .unwrap_or(u64::MAX);
        if size > max_bytes || started.elapsed() > timeout {
            if super::terminate_child_tree(&mut child) {
                let _ = child.wait();
            }
            return Err("Codex extraction or verification exceeded its safety limit".into());
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() => {
                if std::fs::metadata(output)
                    .map(|m| m.len())
                    .unwrap_or(u64::MAX)
                    > max_bytes
                {
                    return Err("Codex extraction or verification exceeded its size limit".into());
                }
                return Ok(());
            }
            Ok(Some(_)) => return Err("Codex extraction or executable verification failed".into()),
            Err(_) => {
                if super::terminate_child_tree(&mut child) {
                    let _ = child.wait();
                }
                return Err("Unable to complete Codex executable verification".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_modified_archive_before_it_can_be_executed() {
        let digest = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(verify_digest(b"abc", digest).is_ok());
        assert!(verify_digest(b"modified", digest).is_err());
        assert!(verify_digest(b"abc", "sha256:invalid").is_err());
    }
}

#[cfg(test)]
mod install_tests {
    use super::*;

    fn release_fixture() -> Release {
        serde_json::from_value(serde_json::json!({
            "tag_name": "rust-v9.9.9", "draft": false, "prerelease": false,
            "assets": [{"id": 123, "url": "https://api.github.com/repos/openai/codex/releases/assets/123", "name": "codex-aarch64-apple-darwin.tar.gz", "size": 100,
                "digest": "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
                "browser_download_url": "https://github.com/openai/codex/releases/download/rust-v9.9.9/codex-aarch64-apple-darwin.tar.gz"}]
        })).unwrap()
    }

    #[test]
    fn rejects_wrong_asset_untrusted_url_and_unstable_release() {
        assert!(select_asset(release_fixture(), "aarch64").is_ok());
        assert!(select_asset(release_fixture(), "x86_64").is_err());
        for url in [
            "http://github.com/openai/codex/releases/download/rust-v9.9.9/codex-aarch64-apple-darwin.tar.gz",
            "https://example.com/codex-aarch64-apple-darwin.tar.gz",
            "https://github.com/other/codex/releases/download/rust-v9.9.9/codex-aarch64-apple-darwin.tar.gz",
        ] {
            let mut release = release_fixture();
            release.assets[0].browser_download_url = url.into();
            assert!(select_asset(release, "aarch64").is_err());
        }
        let mut release = release_fixture();
        release.prerelease = true;
        assert!(select_asset(release, "aarch64").is_err());
        let mut release = release_fixture();
        release.assets[0].digest = None;
        assert!(select_asset(release, "aarch64").is_err());
    }

    #[test]
    fn rejects_asset_api_url_not_bound_to_positive_official_id() {
        for url in [
            "https://api.github.com/repos/openai/codex/releases/assets/999",
            "https://api.github.com/repos/other/codex/releases/assets/123",
            "https://example.com/repos/openai/codex/releases/assets/123",
            "http://api.github.com/repos/openai/codex/releases/assets/123",
            "https://api.github.com/repos/openai/codex/releases/assets/123?download=other",
        ] {
            let mut release = release_fixture();
            release.assets[0].url = url.into();
            assert!(select_asset(release, "aarch64").is_err());
        }
        let mut release = release_fixture();
        release.assets[0].id = 0;
        release.assets[0].url =
            "https://api.github.com/repos/openai/codex/releases/assets/0".into();
        assert!(select_asset(release, "aarch64").is_err());
    }

    fn archive_fixture(root: &Path, member: &str, executable: &[u8]) -> Vec<u8> {
        std::fs::write(root.join(member), executable).unwrap();
        let archive = root.join("fixture.tar.gz");
        assert!(Command::new("/usr/bin/tar")
            .args(["-czf"])
            .arg(&archive)
            .arg("-C")
            .arg(root)
            .arg("--")
            .arg(member)
            .status()
            .unwrap()
            .success());
        std::fs::read(archive).unwrap()
    }

    fn selected_for(bytes: &[u8]) -> SelectedAsset {
        let mut release = release_fixture();
        release.assets[0].size = bytes.len() as u64;
        release.assets[0].digest = Some(format!("sha256:{:x}", Sha256::digest(bytes)));
        select_asset(release, "aarch64").unwrap()
    }

    #[test]
    fn failed_digest_member_or_probe_preserves_previous_executable() {
        let fixture = tempfile::tempdir().unwrap();
        let bin = fixture.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        let previous = b"previous working executable";
        std::fs::write(bin.join("codex"), previous).unwrap();
        let bytes = archive_fixture(
            fixture.path(),
            "codex-aarch64-apple-darwin",
            b"#!/bin/sh\nexit 1\n",
        );
        let selected = selected_for(&bytes);
        assert!(install_archive(&bin, &selected, &bytes).is_err());
        assert_eq!(std::fs::read(bin.join("codex")).unwrap(), previous);
        let mut modified = bytes.clone();
        modified[0] ^= 1;
        assert!(install_archive(&bin, &selected, &modified).is_err());
        assert_eq!(std::fs::read(bin.join("codex")).unwrap(), previous);
        let wrong_member = archive_fixture(fixture.path(), "other-member", b"#!/bin/sh\nexit 0\n");
        assert!(install_archive(&bin, &selected_for(&wrong_member), &wrong_member).is_err());
        assert_eq!(std::fs::read(bin.join("codex")).unwrap(), previous);
        assert_eq!(
            std::fs::read_dir(&bin).unwrap().count(),
            1,
            "failed staging directories must be removed"
        );
    }

    #[test]
    fn verified_version_replaces_old_binary_atomically_without_node() {
        let fixture = tempfile::tempdir().unwrap();
        let bin = fixture.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        std::fs::write(bin.join("codex"), b"old").unwrap();
        let script = b"#!/bin/sh\nprintf 'codex-cli 9.9.9\\n'\n";
        let bytes = archive_fixture(fixture.path(), "codex-aarch64-apple-darwin", script);
        install_archive(&bin, &selected_for(&bytes), &bytes).unwrap();
        assert_eq!(std::fs::read(bin.join("codex")).unwrap(), script);
        assert_eq!(
            std::fs::metadata(bin.join("codex"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(std::fs::read_dir(&bin).unwrap().count(), 1);
    }

    #[tokio::test]
    #[ignore = "downloads and probes the latest official Codex binary into an isolated test directory"]
    async fn downloads_verified_official_native_codex_without_touching_user_path() {
        let fixture = tempfile::tempdir().unwrap();
        let bin = fixture.path().join("bin");
        install(bin.clone()).await.unwrap();
        let output = Command::new(bin.join("codex"))
            .arg("--version")
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).starts_with("codex-cli "));
        eprintln!(
            "Verified native install: {}",
            String::from_utf8_lossy(&output.stdout).trim()
        );
    }
}
