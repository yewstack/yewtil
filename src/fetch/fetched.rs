use yew::{Renderable, Properties, Component, Html, ComponentLink, Children};
use std::rc::Rc;

pub struct Fetched<T: 'static> {
    props: FetchedProps<T>
}

pub struct Render<T: 'static> {
    render: Option<Box<Fn(&T) -> Html<Fetched<T>>>>
}
impl <T> Render<T> {
    pub fn new<F:Fn(&T) -> Html<Fetched<T>> + 'static >(f: F) -> Self {
        Render {
            render: Some(Box::new(f))
        }
    }
}
impl <T> Default for Render<T> {
    fn default() -> Self {
        Render {
            render: None
        }
    }
}

#[derive(Properties)]
pub struct FetchedProps<T: 'static> {
    children: Children<Fetched<T>>,
    pub render: Render<T>,
    /// The user should not set this, but it will always be set by the Fetch component.
    pub(crate) data: Option<Rc<T>> // TODO maybe use a wrapper around MaybeUninit<RC<T>> with Default implemented for it here. That way, we can eliminate the discriminant.
}

#[derive(Properties)]
pub struct PartialFetchedProps<T: 'static> {
    children: Children<Fetched<T>>,
    render: Option<Box<Fn(&T) -> Html<Fetched<T>>>>
}

impl <T: 'static> Component for Fetched<T> {
    type Message = ();
    type Properties = FetchedProps<T>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Fetched {
            props
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        true
    }
}


impl <T: 'static> Renderable<Fetched<T>> for Fetched<T> {
    fn view(&self) -> Html<Self>{
        if let Some(render) = &self.props.render.render {
            (render)(&self.props.data.as_ref().unwrap())
        } else {
            self.props.children.iter().collect()
        }
    }
}