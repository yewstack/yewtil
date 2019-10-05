use yew::{Renderable, Properties, Component, Html, ComponentLink, Children, Callback};
pub struct Failed<M: 'static> {
    props: FailedProps<M>
}

#[derive(Properties)]
pub struct FailedProps<M: 'static> {
    children: Children<Failed<M>>,
    pub (crate) callback: Option<Callback<M>>
}

impl <M: 'static> Component for Failed<M> {
    type Message = M;
    type Properties = FailedProps<M>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Failed {
            props
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        if let Some(callback) = &self.props.callback {
            callback.emit(msg)
        }
        false
    }
}


impl <M: 'static> Renderable<Failed<M>> for Failed<M> {
    fn view(&self) -> Html<Self>{
        self.props.children.iter().collect()
    }
}
