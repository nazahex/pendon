use pendon_core::Event;
use pendon_renderer_ast::render_ast_to_string;

pub fn render_to_string(events: &[Event]) -> Result<String, serde_json::Error> {
    render_ast_to_string(events)
}
