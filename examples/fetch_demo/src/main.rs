use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yewtil::fetch::{Fetch, FetchState};
use yewtil::fetch::unloaded::Unloaded;
use yewtil::fetch::fetched::Fetched;
use yewtil::fetch::fetched::Render;
use std::rc::Rc;

pub struct Model {
    fetch_state: FetchState<String>
}

pub enum Msg {
    DoIt,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model { fetch_state: FetchState::Fetched(Some(Rc::new("Yeet".to_string()))) }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DoIt => {
                self.fetch_state.unload();
                true
            }
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <>
                <Fetch<String> state = &self.fetch_state>
                    <Fetched<String>  render=Render::new(|s| html!{s})  />
                    <Unloaded>
                        <div> {"hello there"} </div>
                    </Unloaded>
                </Fetch>
                <button onclick=|_| Msg::DoIt>{"do it"}</button>
            </>

        }
    }
}

fn main() {
    web_logger::init();
    yew::start_app::<Model>();
}