
use yew_pure::{PureComponent, Emissive, Pure};
use yew::{Callback, Component, Properties, html};

use crate::{Model, Msg};
use yew::virtual_dom::VNode;

#[derive(PartialEq, Properties)]
pub struct Button {
    #[props(required)]
    pub callback: Callback<<Model as Component>::Message>,
    pub text: String
}

impl PureComponent for Button {
    fn render(&self) -> VNode<Pure<Self>> {
        html! {
            <button onclick=|_| Msg::DoIt>{ &self.text }</button>
        }
    }
}

// TODO, this could be easily derived by just annotating the Button to indicate which struct is the callback.
impl Emissive for Button {
    type Message = <Model as Component>::Message;

    fn emit(&self, msg: Self::Message) {
        self.callback.emit(msg)
    }
}