use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

use yewtil::Pure;
mod button;
use crate::button::Button;
use yewtil::transform_cb;

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
                log::info!("got message");
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
    web_logger::init();
    yew::start_app::<Model>();
}

