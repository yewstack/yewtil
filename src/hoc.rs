use yew::{Component, Properties, ShouldRender, ComponentLink, Html, Renderable};
use std::marker::PhantomData;
use yew::virtual_dom::vcomp::ScopeHolder;
use yew::virtual_dom::{VNode, VComp};

pub struct Hoc<T, U>
{
    state: U,
    phantom: PhantomData<T>
}


pub trait HigherOrderComponent<T>: Sized + 'static
    where
        T: Component + Renderable<T>,
{
    type Message;
    type Properties: Properties;

    fn create(properties: Self::Properties, link: ComponentLink<Hoc<T, Self>>) -> Self;

    fn mounted(&mut self) -> ShouldRender;

    fn update(&mut self, msg: Self::Message) -> ShouldRender;

    fn change(&mut self, msg: Self::Properties) -> ShouldRender;

    fn to_inner_properties(&self) -> T::Properties;

    fn render(&self, inner: Html<Hoc<T, Self>>) -> Html<Hoc<T, Self>> {
        inner
    }
}

impl <T, U> Component for Hoc<T, U>
    where
        T: Component + Renderable<T> + Properties,
        U: HigherOrderComponent<T>,
{
    type Message = U::Message;
    type Properties = U::Properties;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state = U::create(props, link);
        Hoc {
            state,
            phantom: PhantomData
        }
    }

    fn mounted(&mut self) -> bool {
        self.state.mounted()
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        U::update(&mut self.state, msg)
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        U::change(&mut self.state, props)
    }

    fn destroy(&mut self) {
        self.state.destroy()
    }
}

impl <T, U> Renderable<Hoc<T,U>> for Hoc<T,U>
where
    T: Component + Renderable<T> + Properties,
    U: HigherOrderComponent<T>,
{
    fn view(&self) -> VNode<Hoc<T, U>> {
        let inner = create_component::<T, Self>(self.state.to_inner_properties());
        self.state.render(inner)
    }
}



/// Creates a component using supplied props and scope.
pub(crate) fn create_component_with_scope<
    COMP: Component + Renderable<COMP>,
    CONTEXT: Component,
>(
    props: COMP::Properties,
    scope_holder: ScopeHolder<CONTEXT>,
) -> Html<CONTEXT> {
    VNode::VComp(VComp::new::<COMP>(props, scope_holder))
}

/// Creates a component using supplied props.
pub(crate) fn create_component<COMP: Component + Renderable<COMP>, CONTEXT: Component>(
    props: COMP::Properties,
) -> Html<CONTEXT> {
    let vcomp_scope: ScopeHolder<CONTEXT> = Default::default();
    create_component_with_scope::<COMP, CONTEXT>(props, vcomp_scope)
}
