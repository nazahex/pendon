pub use crate::assets::*;
pub use crate::locales::*;
pub use crate::templates::*;
pub use crate::theme::*;
pub use crate::widgets::*;

mod assets;
mod locales;
mod templates;
mod theme;
pub mod widgets;

use is_terminal::IsTerminal as _;

pub fn is_interactive_stderr() -> bool {
    std::io::stderr().is_terminal()
}
