use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportEntry {
    Raw(String),
    Structured {
        module: String,
        default: Option<String>,
        names: Vec<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentTemplate {
    pub node_type: String,
    pub node_name: Option<String>,
    pub template: String,
}

#[derive(Clone, Debug, Default)]
pub struct SolidRenderHints {
    pub imports: Vec<ImportEntry>,
    pub templates: Vec<ComponentTemplate>,
}

pub fn normalize_imports(hints: Option<&SolidRenderHints>) -> Vec<String> {
    let mut raw: BTreeSet<String> = BTreeSet::new();
    let mut structured: BTreeMap<String, (Option<String>, BTreeSet<String>)> = BTreeMap::new();

    if let Some(h) = hints {
        for imp in &h.imports {
            match imp {
                ImportEntry::Raw(line) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        raw.insert(trimmed.to_string());
                    }
                }
                ImportEntry::Structured {
                    module,
                    default,
                    names,
                } => {
                    let entry = structured
                        .entry(module.to_string())
                        .or_insert((None, BTreeSet::new()));
                    if entry.0.is_none() {
                        entry.0 = default.clone();
                    }
                    for n in names {
                        if !n.trim().is_empty() {
                            entry.1.insert(n.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut lines: Vec<String> = Vec::new();
    for (module, (default, names)) in structured {
        if default.is_none() && names.is_empty() {
            continue;
        }
        let mut line = String::from("import ");
        let mut placed = false;
        if let Some(def) = default {
            line.push_str(&def);
            placed = true;
        }
        if !names.is_empty() {
            if placed {
                line.push_str(", ");
            }
            line.push('{');
            let mut first = true;
            for name in names {
                if !first {
                    line.push_str(", ");
                }
                line.push_str(&name);
                first = false;
            }
            line.push('}');
            placed = true;
        }
        if placed {
            line.push_str(" from \"");
            line.push_str(&module);
            line.push_str("\";");
            lines.push(line);
        }
    }

    for line in raw {
        lines.push(line);
    }

    lines
}
