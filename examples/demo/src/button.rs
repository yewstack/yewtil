use crate::Msg;
use yew::virtual_dom::VNode;
use yew::{html, Callback, Properties};
use yewtil::{Emissive, Pure, PureComponent};

#[derive(PartialEq, Properties, Emissive)]
pub struct Button {
    #[props(required)]
    pub callback: Callback<Msg>,
    pub text: String,
}

impl PureComponent for Button {
    fn render(&self) -> VNode<Pure<Self>> {
        html! {
            <button onclick=|_| Msg::DoIt>{ &self.text }</button>
        }
    }
}
