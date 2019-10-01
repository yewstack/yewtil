use yew::{Renderable, Properties, Component, Html, ComponentLink, Children};
pub struct Canceled {
    props: CanceledProps
}

#[derive(Properties)]
pub struct CanceledProps {
    children: Children<Canceled>
}

impl Component for Canceled {
    type Message = ();
    type Properties = CanceledProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Canceled {
            props
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        true
    }
}


impl Renderable<Canceled> for Canceled {
    fn view(&self) -> Html<Self>{
        self.props.children.iter().collect()
    }
}