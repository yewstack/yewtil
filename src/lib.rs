//! Utility library for the Yew frontend web framework.
//!
//! All features:
//!
//! * "neq" - NeqAssign trait
//! * "pure" - Pure components
//! * "future" - Async support for Yew Messages
//! * "fetch" - Wrapper that holds requests and responses.
//! * "mrc_irc" - Ergonomic Rc pointers.
//! * "lrc" - Linked-list Rc pointer.
//! * "history" - History tracker
//! * "dsl" - Use functions instead of Yew's `html!` macro.

#[cfg(feature = "dsl")]
pub mod dsl;

#[cfg(feature = "neq")]
mod not_equal_assign;

#[cfg(feature = "pure")]
mod pure;

#[cfg(feature = "with_callback")]
mod with_callback;

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


#[cfg(feature = "with_callback")]
pub use with_callback::WithCallback;

#[cfg(feature = "fetch")]
pub mod fetch;

#[cfg(feature = "effect")]
mod effect;
#[cfg(feature = "effect")]
pub use effect::{Effect, effect};

#[cfg(all(target_arch = "wasm32", not(target_os="wasi"), not(cargo_web), feature = "future"))]
pub mod future;
