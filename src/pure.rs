use yew::{Properties, Html, Component, ComponentLink, ShouldRender, Renderable};
use crate::neq_assign;


pub trait PureComponent: Properties + Emissive + PartialEq + Sized + 'static {
    fn render(&self) -> Html<Pure<Self>>;
}

pub trait Emissive {
    type Message;
    fn emit(&self, msg: Self::Message);
}

#[derive(Debug)]
pub struct Pure<T>(T);


impl <T: PureComponent + Emissive + PartialEq + 'static> Component for Pure<T> {
    type Message = <T as Emissive>::Message;
    type Properties = T;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Pure(props)
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        self.0.emit(msg);
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        neq_assign(&mut self.0, props)
    }
}

impl <T: PureComponent + Emissive + PartialEq + 'static> Renderable<Pure<T>> for Pure<T> {
    fn view(&self) -> Html<Pure<T>> {
        self.0.render()
    }
}
