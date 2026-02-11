use crate::specs::{IndexedPlugin, PluginIndexFile, PluginSpec};
use std::path::{Path, PathBuf};

pub fn load_spec_from_path<P: AsRef<Path>>(path: P) -> Result<PluginSpec, String> {
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read plugin spec {}: {}", path.as_ref().display(), e))?;
    toml::from_str(&text)
        .map_err(|e| format!("invalid plugin spec {}: {}", path.as_ref().display(), e))
}

pub fn load_index_from_path<P: AsRef<Path>>(path: P) -> Result<Vec<IndexedPlugin>, String> {
    let text = std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "cannot read plugin index {}: {}",
            path.as_ref().display(),
            e
        )
    })?;
    let idx: PluginIndexFile = toml::from_str(&text)
        .map_err(|e| format!("invalid plugin index {}: {}", path.as_ref().display(), e))?;
    let base = path
        .as_ref()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(PathBuf::new);
    let mut out: Vec<IndexedPlugin> = Vec::new();
    for entry in idx.plugins.into_iter().filter(|p| p.enabled) {
        if let Some(spec) = entry.inline {
            out.push(IndexedPlugin { id: entry.id, spec });
            continue;
        }
        if let Some(rel) = entry.path {
            let resolved = base.join(rel);
            let spec = load_spec_from_path(&resolved)?;
            out.push(IndexedPlugin { id: entry.id, spec });
        }
    }
    Ok(out)
}
