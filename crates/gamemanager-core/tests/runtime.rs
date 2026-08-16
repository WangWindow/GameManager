use std::{io::Write, sync::Arc};

use futures_util::StreamExt;
use gamemanager_core::{
    AppPaths, CoreError, HttpClient, NwjsFlavor, Operation, OperationStage, RuntimeManager,
    current_nwjs_target, ensure_compatibility_patch,
};

#[tokio::test]
async fn operation_reports_progress_then_one_terminal_outcome() -> gamemanager_core::Result<()> {
    let operation = Operation::from_steps([("download", 25), ("install", 100)], async {
        Ok::<_, CoreError>(7)
    });
    let mut progress = operation.progress();
    assert_eq!(
        progress.next().await.expect("first event").percent,
        Some(25)
    );
    assert_eq!(operation.into_future().await?, 7);
    Ok(())
}

#[tokio::test]
async fn future_operation_emits_terminal_progress_after_the_future_finishes() {
    let operation = Operation::from_future("download", async { Ok::<_, CoreError>(7) });
    let mut progress = operation.progress();
    let future = operation.into_future();

    assert_eq!(progress.next().await.expect("start event").percent, Some(0));
    assert_eq!(future.await.expect("operation result"), 7);

    let terminal = progress.next().await.expect("terminal event");
    assert_eq!(terminal.percent, Some(100));
    assert_eq!(terminal.state, OperationStage::Completed);
}

#[test]
fn managed_mkxpz_runtime_contains_the_global_steam_patch() -> gamemanager_core::Result<()> {
    let root = tempfile::tempdir()?;
    ensure_compatibility_patch(root.path())?;
    let patch = std::fs::read_to_string(root.path().join("patches/compatibility.rb"))?;
    assert!(patch.contains("steam_acheivement"));
    Ok(())
}

#[test]
fn importing_mkxpz_replaces_current_and_keeps_the_patch() -> gamemanager_core::Result<()> {
    let root = tempfile::tempdir()?;
    let paths = AppPaths::from_data_dir(root.path().join("data"));
    let archive = root.path().join("mkxp-z-test.zip");
    let mut writer = zip::ZipWriter::new(std::fs::File::create(&archive)?);
    writer
        .start_file("mkxp-z.x86_64", zip::write::SimpleFileOptions::default())
        .map_err(|error| CoreError::Engine(error.to_string()))?;
    let mut elf = [0_u8; 20];
    elf[..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']);
    elf[4] = 2;
    elf[5] = 1;
    elf[18..20].copy_from_slice(&62_u16.to_le_bytes());
    writer.write_all(&elf)?;
    writer
        .finish()
        .map_err(|error| CoreError::Engine(error.to_string()))?;

    let result = RuntimeManager::new(paths).import_mkxpz_archive(&archive)?;
    assert!(result.executable_path.is_file());
    assert!(
        result
            .install_dir
            .join("patches/compatibility.rb")
            .is_file()
    );
    Ok(())
}

#[tokio::test]
async fn nwjs_download_uses_injected_client_without_network() -> gamemanager_core::Result<()> {
    let root = tempfile::tempdir()?;
    let paths = AppPaths::from_data_dir(root.path().join("data"));
    let archive = zip_bytes("nw");
    let manager = RuntimeManager::new(paths.clone())
        .with_http_client(Arc::new(FakeHttpClient { bytes: archive }));
    let operation = manager.download_nwjs(
        "0.84.0".to_owned(),
        NwjsFlavor::Normal,
        "win-x64".to_owned(),
    );
    let result = operation.into_future().await?;
    assert!(result.install_dir.join("nw").is_file());
    Ok(())
}

#[tokio::test]
async fn latest_nwjs_download_resolves_the_stable_version_without_network()
-> gamemanager_core::Result<()> {
    let root = tempfile::tempdir()?;
    let paths = AppPaths::from_data_dir(root.path().join("data"));
    let manager = RuntimeManager::new(paths).with_http_client(Arc::new(LatestHttpClient {
        archive: current_target_archive("nw"),
    }));

    let result = manager
        .download_latest_nwjs(NwjsFlavor::Normal)
        .into_future()
        .await?;

    assert_eq!(result.version, "0.114.1");
    assert!(result.install_dir.join("nw").is_file());
    Ok(())
}

fn zip_bytes(name: &str) -> Vec<u8> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(&mut bytes);
    writer
        .start_file(name, zip::write::SimpleFileOptions::default())
        .expect("zip entry");
    writer.write_all(b"runtime").expect("zip bytes");
    writer.finish().expect("zip finish");
    bytes.into_inner()
}

fn current_target_archive(name: &str) -> Vec<u8> {
    if current_nwjs_target()
        .expect("supported NW.js target")
        .starts_with("linux-")
    {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        {
            let mut archive = tar::Builder::new(&mut encoder);
            let mut header = tar::Header::new_gnu();
            header.set_size(b"runtime".len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            archive
                .append_data(&mut header, name, &b"runtime"[..])
                .expect("tar entry");
            archive.finish().expect("tar finish");
        }
        encoder.finish().expect("gzip finish")
    } else {
        zip_bytes(name)
    }
}

struct FakeHttpClient {
    bytes: Vec<u8>,
}

struct LatestHttpClient {
    archive: Vec<u8>,
}

impl HttpClient for LatestHttpClient {
    fn get(
        &self,
        url: &str,
    ) -> futures_util::future::BoxFuture<'static, gamemanager_core::Result<Vec<u8>>> {
        let response = if url.ends_with("versions.json") {
            br#"{"stable":"v0.114.1"}"#.to_vec()
        } else {
            self.archive.clone()
        };
        Box::pin(async move { Ok(response) })
    }
}

impl HttpClient for FakeHttpClient {
    fn get(
        &self,
        _: &str,
    ) -> futures_util::future::BoxFuture<'static, gamemanager_core::Result<Vec<u8>>> {
        let bytes = self.bytes.clone();
        Box::pin(async move { Ok(bytes) })
    }
}
