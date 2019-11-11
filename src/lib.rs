//! Utility library for the Yew frontend web framework.
//pub mod fetch;

#[cfg(feature = "dsl")]
pub mod dsl;

#[cfg(feature = "neq")]
mod not_equal_assign;

#[cfg(feature = "pure")]
mod pure;


#[cfg(any(feature = "mrc_irc", feature = "lrc"))]
pub mod ptr;

#[cfg(feature = "history")]
mod history;

#[cfg(feature = "history")]
pub use history::History;

#[cfg(feature = "neq")]
pub use not_equal_assign::NeqAssign;

#[cfg(feature = "pure")]
pub use pure::{Emissive, Pure, PureComponent, PureEmissiveComponent};

#[cfg(feature = "pure")]
pub use yewtil_macro::Emissive;
