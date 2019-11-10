//! Smart pointers for use within Yew.
//!
//! These all offer similar semantics to `std::rc::Rc`, but offer better ergonomics within Yew,
//! or functionality not available in `Rc`.
mod irc;
mod lrc;
mod mrc;
mod rc_box;
mod takeable;

pub use irc::Irc;
pub use lrc::Lrc;
pub use mrc::Mrc;

pub(crate) type IsZero = bool;
