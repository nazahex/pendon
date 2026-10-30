//! `pendon-types` — type selectors and document-scoped type resolution.
//!
//! One algorithm, three consumers (see
//! `docs/workflows/type-resolution.md`):
//!
//! | Consumer         | Asks                                                    |
//! | ---------------- | ------------------------------------------------------- |
//! | `renderer-solid` | which component renders an element                       |
//! | `number`  | which numbering block an element belongs to              |
//! | `plugin-extract` | which extraction block(s) an element belongs to          |
//!
//! A config `type = …` entry is a [`Selector`] list. Resolution is scoped to a
//! **definition group** — one element section of one task — and answered per
//! *occurring type* of the document ([`TypeSet`]):
//!
//! * [`Group::winner`] — exclusive consumers (one component, one counter).
//! * [`Group::matching`] — additive consumers (one rule per destination).
//!
//! The crate depends on nothing but `std` (plus `serde` derives) so every
//! consumer can share it without a dependency cycle.

mod group;
mod selector;
mod type_set;

pub use group::{matching, winner, Definition, Group, Rule};
pub use selector::{Selector, SelectorError};
pub use type_set::{TypeId, TypeSet};
