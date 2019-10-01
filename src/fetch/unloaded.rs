use yew::{Renderable, Properties, Component, Html, ComponentLink, Children};
pub struct Unloaded {
    props: UnloadedProps
}

#[derive(Properties)]
pub struct UnloadedProps {
    children: Children<Unloaded>
}

impl Component for Unloaded {
    type Message = ();
    type Properties = UnloadedProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Unloaded {
            props
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        true
    }
}


impl Renderable<Unloaded> for Unloaded {
    fn view(&self) -> Html<Self>{
        self.props.children.iter().collect()
    }
}
