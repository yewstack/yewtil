use yew::{Renderable, Properties, Component, Html, ComponentLink, Children};
pub struct Failed {
    props: FailedProps
}

#[derive(Properties)]
pub struct FailedProps {
    children: Children<Failed>
}

impl Component for Failed {
    type Message = ();
    type Properties = FailedProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Failed {
            props
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        true
    }
}


impl Renderable<Failed> for Failed {
    fn view(&self) -> Html<Self>{
        self.props.children.iter().collect()
    }
}
