#![recursion_limit = "256"]
use std::rc::Rc;
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yewtil::fetch::fetched::Fetched;
use yewtil::fetch::unloaded::Unloaded;
use yewtil::fetch::{Fetch, FetchStateVariant};

pub struct Model {
    fetch_state: FetchStateVariant<String>,
}

pub enum Msg {
    DoIt,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model {
            fetch_state: FetchStateVariant::Fetched(Some(Rc::new("Lorem ipsum dolor sit".to_string()))),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DoIt => {
                log::info!("{:?}", self.fetch_state.make_mut());
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
                <Fetch<String, Msg> state = &self.fetch_state, callback=From::from >
                    <Fetched<String, Msg>  render=Fetched::render(|s| html!{
                        <>
                            {s}
                            <button onclick=|_| Msg::DoIt>{"Button"}</button>
                        </>
                    })  />
                    <Unloaded<Msg>>
                        <div> {"hello there"} </div>
                    </Unloaded>
                </Fetch>
            </>

        }
    }
}

fn main() {
    web_logger::init();
    yew::start_app::<Model>();
}
