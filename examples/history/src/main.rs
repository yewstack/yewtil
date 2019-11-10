use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yewtil::History;

pub struct Model {
    text: History<String>
}

pub enum Msg {
    SetText(String),
    Reset,
    Forget
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model {
            text: History::new("".to_string())
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetText(text) => {
                self.text.set(text); // TODO needs a neq_assign_set
                true
            }
            Msg::Reset => {
                self.text.reset();
                true
            }
            Msg::Forget => {
                self.text.forget();
                true
            }
        }
    }

    fn view(&self) -> Html<Self> {
        html! {
            <>
                <div>
                   {&*self.text} // This implicit clone is cheap, as it doesn't copy the String
                </div>
                <div>
                    <input
                        type = "text"
                        value = &*self.text,
                        oninput = |x| Msg::SetText(x.value)
                    />
                    <button onclick=|_| Msg::Reset >{"Reset to the oldest value"} </button>
                    <button onclick=|_| Msg::Forget>{"Forget prior values"} </button>
                </div>
            </>
        }
    }
}

fn main() {
    web_logger::init();
    yew::start_app::<Model>();
}
