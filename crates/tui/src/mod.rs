pub mod assets;
pub mod locales;
pub mod templates;
pub mod theme;
pub mod widgets;

use is_terminal::IsTerminal as _;

pub fn is_interactive_stderr() -> bool {
    std::io::stderr().is_terminal()
}
