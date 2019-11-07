use crate::NeqAssign;
use yew::{Component, ComponentLink, Html, Properties, ShouldRender};

pub trait PureComponent: Properties + Emissive + PartialEq + Sized + 'static {
    fn render(&self) -> Html<Pure<Self>>;
}

/// # Note
/// When deriving, the derive macro will attempt to locate a field with a `Callback<_>`.
/// type and use the inner type of the callback to specify the `Message` type of `Emissive`.
/// The derived `emit` function will call `self.<name of the callback struct>.emit(msg)`.
///
/// If it cannot find a callback struct, the `Message` type will be set to `()` and `emit` will do nothing.
pub trait Emissive {
    type Message;
    fn emit(&self, msg: Self::Message);
}

#[derive(Debug)]
pub struct Pure<T>(T);

impl<T: PureComponent + Emissive + PartialEq + 'static> Component for Pure<T> {
    type Message = <T as Emissive>::Message;
    type Properties = T;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Pure(props)
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.0.emit(msg);
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.0.neq_assign(props)
    }

    fn view(&self) -> Html<Self> {
        self.0.render()
    }
}
