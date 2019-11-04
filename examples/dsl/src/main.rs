use yew::{Component, ComponentLink, Html, ShouldRender};

use yewtil::dsl::{tag, list, text, BoxedVNodeProducer};

pub struct Model {}

pub enum Msg {
    DoIt,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model {}
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DoIt => {
                log::info!("got message");
                true
            }
        }
    }

    fn view(&self) -> Html<Self> {
        BoxedVNodeProducer::from(
            list()
                .child(text("Hello there"))
                .child(
                    tag("p")
                        .child(text("Paragraph content"))
                )
                .child(
                    list()
                        .child(
                            tag("b")
                                .child(text("Bolded"))
                        )
                        .child(text("Normal text"))
                )
            )
            .build()
    }
}


fn main() {
    web_logger::init();
    yew::start_app::<Model>();
}