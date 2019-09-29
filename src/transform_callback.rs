use yew::{Callback, Component, Renderable};
use yew::html::Scope;

/// Transforms the callback from an arbitrary message type to the one for the specified component
pub fn transform_cb<COMP, T>() -> Callback<T>
    where
        COMP: Component + Renderable<COMP>,
        T: Into<<COMP as Component>::Message>,
{
    Callback::from(move |t: T| {
        let mut scope: Scope<COMP> = Scope::default();
        scope.send_message(t.into())
    })
}