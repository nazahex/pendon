use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepInfo {
    pub path: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheFile {
    pub task: String,
    pub input_hash: String,
    pub output_hash: String,
    pub mtime: u64,
    pub deps: Vec<DepInfo>, // Berubah dari Vec<String> menjadi Vec<DepInfo>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheTask {
    pub config_hash: String,
    pub input_pattern: String,
    pub output_pattern: String,
    pub plugin: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheManifest {
    pub version: u32,
    pub config_hash: String,
    pub tasks: HashMap<String, CacheTask>,
    pub files: HashMap<String, CacheFile>,
}

const CACHE_DIR: &str = "./.pendon";
const CACHE_FILE: &str = "./.pendon/cache.toml";

impl CacheManifest {
    pub fn load() -> Self {
        if !Path::new(CACHE_FILE).exists() {
            return Self::default();
        }

        match fs::read_to_string(CACHE_FILE) {
            Ok(content) => match toml::from_str(&content) {
                Ok(manifest) => manifest,
                Err(e) => {
                    eprintln!("Warning: failed to parse cache file: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("Warning: failed to read cache file: {}", e);
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        fs::create_dir_all(CACHE_DIR)
            .map_err(|e| format!("cannot create cache directory: {}", e))?;

        let content =
            toml::to_string_pretty(self).map_err(|e| format!("cannot serialize cache: {}", e))?;

        fs::write(CACHE_FILE, content).map_err(|e| format!("cannot write cache file: {}", e))?;

        Ok(())
    }

    pub fn clean() -> Result<(), String> {
        if Path::new(CACHE_FILE).exists() {
            fs::remove_file(CACHE_FILE).map_err(|e| format!("cannot remove cache file: {}", e))?;
        }
        Ok(())
    }

    pub fn get_file(&self, path: &str) -> Option<&CacheFile> {
        self.files.get(path)
    }

    pub fn update_file(&mut self, path: String, file: CacheFile) {
        self.files.insert(path, file);
    }

    pub fn update_task(&mut self, name: String, task: CacheTask) {
        self.tasks.insert(name, task);
    }

    pub fn get_task(&self, name: &str) -> Option<&CacheTask> {
        self.tasks.get(name)
    }
}

pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub fn hash_file(path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("cannot read file {}: {}", path.display(), e))?;
    Ok(hash_content(&content))
}

pub fn get_file_mtime(path: &Path) -> Result<u64, String> {
    let metadata = fs::metadata(path)
        .map_err(|e| format!("cannot get metadata for {}: {}", path.display(), e))?;

    Ok(metadata
        .modified()
        .map_err(|e| format!("cannot get mtime for {}: {}", path.display(), e))?
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("cannot convert mtime for {}: {}", path.display(), e))?
        .as_secs())
}

pub fn should_skip_file(
    cache: &CacheManifest,
    input_path: &str,
    _output_path: &str, // we no longer use output_path for cache validation
    task_config_hash: &str,
) -> (bool, Vec<String>) {
    let input_path_obj = Path::new(input_path);

    if !input_path_obj.exists() {
        return (false, vec![]);
    }

    let cached = match cache.get_file(input_path) {
        Some(entry) => entry,
        None => return (false, vec![]),
    };

    if let Some(task) = cache.get_task(&cached.task) {
        if task.config_hash != task_config_hash {
            return (false, vec![]);
        }
    } else {
        return (false, vec![]);
    }

    // Check if the input file's hash has changed
    let current_input_hash = match hash_file(input_path_obj) {
        Ok(h) => h,
        Err(_) => return (false, vec![]),
    };

    if current_input_hash != cached.input_hash {
        return (false, vec![]);
    }

    // Check all hashes of dependencies
    let mut changed_deps = Vec::new();
    for cached_dep in &cached.deps {
        let dep_path = Path::new(&cached_dep.path);
        if !dep_path.exists() {
            changed_deps.push(cached_dep.path.clone());
            continue;
        }
        let current_dep_hash = match hash_file(dep_path) {
            Ok(h) => h,
            Err(_) => {
                changed_deps.push(cached_dep.path.clone());
                continue;
            }
        };
        if current_dep_hash != cached_dep.hash {
            changed_deps.push(cached_dep.path.clone());
        }
    }

    if !changed_deps.is_empty() {
        return (false, changed_deps);
    }

    // Every hash is correct. So we can skip the file.
    (true, vec![])
}

pub fn create_cache_entry(
    task_name: &str,
    input_path: &str,
    output_path: &str,
    deps: Vec<String>,
) -> Result<CacheFile, String> {
    let input_path_obj = Path::new(input_path);
    let output_path_obj = Path::new(output_path);

    let input_hash = hash_file(input_path_obj)?;

    let output_hash = if output_path_obj.exists() {
        hash_file(output_path_obj).unwrap_or_default()
    } else {
        String::new()
    };

    let mtime = get_file_mtime(input_path_obj).unwrap_or(0);

    // Save hashes of dependencies
    let mut dep_infos = Vec::new();
    for dep in deps {
        let dep_path = Path::new(&dep);
        if dep_path.exists() {
            if let Ok(hash) = hash_file(dep_path) {
                dep_infos.push(DepInfo { path: dep, hash });
            }
        }
    }

    Ok(CacheFile {
        task: task_name.to_string(),
        input_hash,
        output_hash,
        mtime,
        deps: dep_infos,
    })
}
