use std::path::Path;
use flate2::read::GzDecoder;
use http_body_util::BodyExt;
use octocrab::Octocrab;
use octocrab::params::repos::Commitish;
use tar::Archive;

use crate::app::BoxError;

pub async fn download_repo_tarball(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    commitish: impl Into<Commitish>,
    dest: &Path,
) -> Result<(), BoxError> {
    let response = client
        .repos(owner, repo)
        .download_tarball(commitish)
        .await?;

    let bytes = response.into_body().collect().await?.to_bytes();

    Archive::new(GzDecoder::new(&bytes[..])).unpack(dest)?;
    Ok(())
}