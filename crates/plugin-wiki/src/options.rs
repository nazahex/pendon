use pendon_plugin_latex::LatexOptions;

#[derive(Debug, Clone, Default)]
pub struct WikiOptions {
    pub link_prefix: Option<String>,
    /// When `Some`, math inside infobox fragments is rendered. `None` (the
    /// default) leaves `$...$` untouched, matching a task without `latex`.
    pub latex: Option<LatexOptions>,
}
