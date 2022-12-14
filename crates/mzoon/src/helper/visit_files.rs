use anyhow::{Error, Result};
use fehler::throws;
use futures::stream::{self, Stream, StreamExt};
use std::path::PathBuf;
use tokio::fs::{self, DirEntry};

// https://stackoverflow.com/a/58825638
pub fn visit_files(
    path: impl Into<PathBuf>,
) -> impl Stream<Item = Result<DirEntry>> + Send + 'static {
    #[throws]
    async fn one_level(path: PathBuf, to_visit: &mut Vec<PathBuf>) -> Vec<DirEntry> {
        let mut dir = fs::read_dir(path).await?;
        let mut files = Vec::new();

        while let Some(child) = dir.next_entry().await? {
            if child.metadata().await?.is_dir() {
                to_visit.push(child.path());
            } else {
                files.push(child)
            }
        }
        files
    }

    stream::unfold(vec![path.into()], |mut to_visit| async {
        let path = to_visit.pop()?;
        let file_stream = match one_level(path, &mut to_visit).await {
            Ok(files) => stream::iter(files).map(Ok).left_stream(),
            Err(error) => stream::once(async { Err(error) }).right_stream(),
        };
        Some((file_stream, to_visit))
    })
    .flatten()
}
