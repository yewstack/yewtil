use yew::{Callback, Children, Component, ComponentLink, Html, Properties};
pub struct Unloaded<M: 'static> {
    props: UnloadedProps<M>,
}

#[derive(Properties)]
pub struct UnloadedProps<M: 'static> {
    children: Children<Unloaded<M>>,
    pub(crate) callback: Option<Callback<M>>,
}

impl<M: 'static> Component for Unloaded<M> {
    type Message = M;
    type Properties = UnloadedProps<M>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Unloaded { props }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        if let Some(callback) = &self.props.callback {
            callback.emit(msg)
        }
        false
    }

    fn view(&self) -> Html<Self> {
        self.props.children.iter().collect()
    }
}

