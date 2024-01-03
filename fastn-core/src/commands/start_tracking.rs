pub async fn start_tracking(
    config: &fastn_core::Config,
    source: &str,
    target: &str,
) -> fastn_core::Result<()> {
    tokio::fs::create_dir_all(config.track_dir()).await?;

    let snapshots = fastn_core::snapshot::get_latest_snapshots(config.ds.root()).await?;
    check(
        config,
        config.ds.root().as_str(),
        &snapshots,
        source,
        target,
    )
    .await?;
    Ok(())
}

async fn check(
    config: &fastn_core::Config,
    base_path: &str,
    snapshots: &std::collections::BTreeMap<String, u128>,
    source: &str,
    target: &str,
) -> fastn_core::Result<()> {
    if !snapshots.contains_key(target) {
        return Err(fastn_core::Error::UsageError {
            message: format!(
                "{} is not synced yet. suggestion: Run `fastn sync {}` to sync the file",
                target, target
            ),
        });
    }

    let timestamp = if let Some(timestamp) = snapshots.get(source) {
        timestamp
    } else {
        return Err(fastn_core::Error::UsageError {
            message: format!(
                "{} is not synced yet. suggestion: Run `fastn sync {}` to sync the file",
                source, source
            ),
        });
    };

    // if source is already tracking target, print message and return
    {
        let track_path = fastn_core::utils::track_path(source, base_path);
        let tracks = fastn_core::tracker::get_tracks(config, base_path, &track_path).await?;

        if tracks.contains_key(target) {
            println!("{} is already tracking {}", source, target);
            return Ok(());
        }
    }

    if let Some((dir, _)) = source.rsplit_once('/') {
        tokio::fs::create_dir_all(
            camino::Utf8PathBuf::from(base_path)
                .join(".tracks")
                .join(dir),
        )
        .await?;
    }

    let new_file_path = fastn_core::utils::track_path(source, base_path);

    write(config, target, *timestamp, &new_file_path).await?;
    println!("{} is now tracking {}", source, target);

    Ok(())
}

async fn write(
    config: &fastn_core::Config,
    target: &str,
    timestamp: u128,
    path: &camino::Utf8PathBuf,
) -> fastn_core::Result<()> {
    let string = if path.exists() {
        let existing_doc = config.ds.read_to_string(path).await?;
        format!(
            "{}\n\n-- fastn.track: {}\nself-timestamp: {}",
            existing_doc, target, timestamp
        )
    } else {
        format!(
            "-- import: fastn\n\n-- fastn.track: {}\nself-timestamp: {}",
            target, timestamp
        )
    };

    config.ds.write_content(path, string.into()).await?;
    Ok(())
}
