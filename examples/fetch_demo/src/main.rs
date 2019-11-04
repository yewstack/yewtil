#![recursion_limit = "256"]
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yewtil::fetch::fetched::Fetched;
use yewtil::fetch::unloaded::Unloaded;
use yewtil::fetch::{Fetch, FetchState};
use yewtil::NeqAssign;

pub struct Model {
    fetch_state: FetchState<String>,
}

pub enum Msg {
    DataLoaded,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model {
            fetch_state: FetchState::unloaded()
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DataLoaded => {
                self.fetch_state.neq_assign(FetchState::fetched("Lorem Ipsum Dolor Sit".to_string()))
            }
        }
    }

    fn view(&self) -> Html<Self> {
        html! {
            <Fetch<String, Msg> state = &self.fetch_state, callback=From::from >
                <Fetched<String, Msg>  render=Fetched::render(|s| html!{
                    <>
                        <div> {s} </div>
                    </>
                })  />
                <Unloaded<Msg>>
                    <div> {"hello there"} </div>
                    <button onclick=|_| Msg::DataLoaded>{"Load Data"}</button>
                </Unloaded>
            </Fetch>
        }
    }
}



fn main() {
    web_logger::init();
    yew::start_app::<Model>();
}
