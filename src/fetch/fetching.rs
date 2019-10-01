use yew::{Renderable, Properties, Component, Html, ComponentLink, Children};
pub struct Fetching {
    props: FetchingProps
}

#[derive(Properties)]
pub struct FetchingProps {
    children: Children<Fetching>
}

impl Component for Fetching {
    type Message = ();
    type Properties = FetchingProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Fetching {
            props
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        true
    }
}


impl Renderable<Fetching> for Fetching {
    fn view(&self) -> Html<Self>{
        self.props.children.iter().collect()
    }
}