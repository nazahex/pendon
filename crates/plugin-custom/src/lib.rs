mod loader;
mod processor;
mod specs;

pub use loader::{load_index_from_path, load_spec_from_path};
pub use processor::process;
pub use specs::{
    AstSpec, AttrSpec, IndexedPlugin, MatcherSpec, PluginIndexEntry, PluginIndexFile, PluginSpec,
    RendererSpec, SolidImportEntry, SolidRendererSpec,
};
