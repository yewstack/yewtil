use yew::{Callback, Children, Component, ComponentLink, Html, Properties};
use std::rc::Rc;

pub struct Failed<M: 'static> {
    props: FailedProps<M>,
}


pub struct Render<M: 'static> {
    render: Option<Box<dyn Fn(&::failure::Error) -> Html<Failed<M>>>>,
}

impl<M: 'static> Render<M> {
    fn new<F: Fn(&::failure::Error) -> Html<Failed<M>> + 'static>(f: F) -> Self {
        Render {
            render: Some(Box::new(f)),
        }
    }
}

impl<M> Default for Render<M> {
    fn default() -> Self {
        Render { render: None }
    }
}


impl<M: 'static> Failed<M> {
    pub fn render<F: Fn(&::failure::Error) -> Html<Failed<M>> + 'static>(f: F) -> Render<M> {
        Render::new(f)
    }
}

#[derive(Properties)]
pub struct FailedProps<M: 'static> {
    children: Children<Failed<M>>,
    pub(crate) callback: Option<Callback<M>>,
    pub(crate) error: Option<Rc<::failure::Error>>,
    pub render: Render<M>
}

impl<M: 'static> Component for Failed<M> {
    type Message = M;
    type Properties = FailedProps<M>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Failed { props }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        if let Some(callback) = &self.props.callback {
            callback.emit(msg)
        }
        false
    }

    fn view(&self) -> Html<Self> {
        if let Some(render) = &self.props.render.render {
            (render)(&self.props.error.as_ref().unwrap())
        } else {
            self.props.children.iter().collect()
        }
    }
}
