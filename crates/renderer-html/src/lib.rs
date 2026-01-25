use pendon_core::Event;
use pendon_renderer_ast::render_ast_to_string;
use serde_json::Value;

mod compact;
mod pretty;
mod utils;

pub use compact::render_html;
pub use pretty::render_html_pretty;

pub(crate) fn events_to_ast_value(events: &[Event]) -> Value {
    let ast_json = render_ast_to_string(events).expect("AST serialization failed");
    serde_json::from_str(&ast_json).expect("Invalid AST JSON")
}
