use crate::Msg;
use yew::virtual_dom::VNode;
use yew::{html, Callback, Properties};
use yewtil::{Emissive, Pure, PureComponent};

/// Alias to make usability better.
pub type Button = Pure<PureButton>;

#[derive(PartialEq, Properties, Emissive)]
pub struct PureButton {
    #[props(required)]
    pub callback: Callback<Msg>,
    pub text: String,
}

impl PureComponent for PureButton {
    fn render(&self) -> VNode<Pure<Self>> {
        html! {
            <button onclick=|_| Msg::DoIt>{ &self.text }</button>
        }
    }
}
