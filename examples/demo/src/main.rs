use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

use yew_pure::Pure;
mod button;
use crate::button::Button;

pub struct Model { }

pub enum Msg {
    DoIt,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model { }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DoIt => {
                true
            }
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <Pure<Button> callback=|x| {x} text = "Click me!" />
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}

