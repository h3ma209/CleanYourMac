use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs;
use tokio::sync::mpsc;

use crate::control::ExecutionControl;
use crate::error::ScanError;

#[derive(Clone, Debug)]
pub struct Chunk {
    pub paths: Vec<PathBuf>,
    pub finale: bool,
}

#[derive(Clone)]
pub struct WalkOptions {
    pub root: PathBuf,
    pub chunk_size: usize,
    pub follow_symlinks: bool,
    /// Return true to skip descending into a directory. Default behavior is to descend.
    pub skip_dir: Arc<dyn Fn(&Path) -> bool + Send + Sync>,
}

impl WalkOptions {
    pub fn new(root: impl Into<PathBuf>, chunk_size: usize) -> Self {
        Self {
            root: root.into(),
            chunk_size: chunk_size.max(1),
            follow_symlinks: false,
            skip_dir: Arc::new(|_| false),
        }
    }
}

/// Depth-first walk. Directories are not emitted. Files are grouped into chunks.
/// The last chunk has `finale = true`, including an empty chunk when the tree has no files.
pub async fn walk(
    options: WalkOptions,
    control: &ExecutionControl,
    out: mpsc::Sender<Chunk>,
) -> Result<(), ScanError> {
    let mut stack = vec![options.root.clone()];
    let mut buf = Vec::with_capacity(options.chunk_size);
    let mut sent_any = false;

    while let Some(dir) = stack.pop() {
        control.checkpoint().await?;
        let mut entries = match fs::read_dir(&dir).await {
            Ok(entries) => entries,
            Err(source) => return Err(ScanError::io(dir, source)),
        };
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|source| ScanError::io(&dir, source))?
        {
            control.checkpoint().await?;
            let path = entry.path();
            let meta = match fs::symlink_metadata(&path).await {
                Ok(meta) => meta,
                Err(source) => return Err(ScanError::io(&path, source)),
            };
            let symlink = meta.file_type().is_symlink();
            if meta.is_dir() && (!symlink || options.follow_symlinks) {
                if !(options.skip_dir)(&path) {
                    stack.push(path);
                }
                continue;
            }
            buf.push(path);
            if buf.len() >= options.chunk_size {
                sent_any = true;
                out.send(Chunk {
                    paths: std::mem::take(&mut buf),
                    finale: false,
                })
                .await
                .map_err(|_| ScanError::Message("chunk channel closed".into()))?;
            }
        }
    }

    if !buf.is_empty() || !sent_any {
        out.send(Chunk {
            paths: buf,
            finale: true,
        })
        .await
        .map_err(|_| ScanError::Message("chunk channel closed".into()))?;
    } else {
        out.send(Chunk {
            paths: Vec::new(),
            finale: true,
        })
        .await
        .map_err(|_| ScanError::Message("chunk channel closed".into()))?;
    }
    Ok(())
}
