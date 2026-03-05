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
    pub global_imports: Vec<ImportEntry>,
    pub template_imports: BTreeMap<(String, Option<String>), Vec<ImportEntry>>,
    pub text_imports: Vec<(String, Vec<ImportEntry>)>,
    pub templates: Vec<ComponentTemplate>,
}

pub fn normalize_imports(
    hints: Option<&SolidRenderHints>,
    used_nodes: &BTreeSet<(String, Option<String>)>,
    used_markers: &BTreeSet<String>,
) -> Vec<String> {
    let mut raw: BTreeSet<String> = BTreeSet::new();
    let mut structured: BTreeMap<String, (Option<String>, BTreeSet<String>)> = BTreeMap::new();

    if let Some(h) = hints {
        ingest_imports(&mut raw, &mut structured, &h.global_imports);

        for ((node_type, node_name), imports) in &h.template_imports {
            let used = used_nodes.contains(&(node_type.clone(), node_name.clone()))
                || (node_name.is_none() && used_nodes.iter().any(|(t, _)| t == node_type));
            if !used {
                continue;
            }
            ingest_imports(&mut raw, &mut structured, imports);
        }

        for (marker, imports) in &h.text_imports {
            if !used_markers.contains(marker) {
                continue;
            }
            ingest_imports(&mut raw, &mut structured, imports);
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

fn ingest_imports(
    raw: &mut BTreeSet<String>,
    structured: &mut BTreeMap<String, (Option<String>, BTreeSet<String>)>,
    imports: &[ImportEntry],
) {
    for imp in imports {
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
