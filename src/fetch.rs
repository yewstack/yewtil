use yew::services::fetch::FetchTask;
use yew::{ShouldRender, Properties, Component, Html, Renderable, ComponentLink, Children, html, virtual_dom::{VComp, VNode}};
use yew::virtual_dom::vcomp::ScopeHolder;
use yew::virtual_dom::VChild;
use yew::html::ChildrenRenderer;
use crate::{Emissive, Pure, PureComponent};
use crate::fetch::unloaded::{Unloaded, UnloadedProps};
use crate::fetch::fetching::{Fetching, FetchingProps};
use crate::fetch::failed::{Failed, FailedProps};
use crate::fetch::fetched::{Fetched, PartialFetchedProps, FetchedProps};
use std::rc::Rc;
use crate::fetch::canceled::{Canceled, CanceledProps};

pub mod unloaded;
pub mod fetching;
pub mod failed;
pub mod fetched;
pub mod canceled;

#[derive(Clone)]
pub enum FetchState<T> {
    Unloaded,
    Fetching(Rc<FetchTask>),
    FetchingWithPersistence(Option<Rc<T>>, Rc<FetchTask>),
    Failed, // TODO actually include error here.
    Fetched(Option<Rc<T>>),
    Canceled
}

impl <T> PartialEq for FetchState<T> {
    fn eq(&self, other: &Self) -> bool {
        use FetchState as FS;
        match (self, other) {
            (FS::Unloaded, FS::Unloaded) | (FS::Fetching(_), FS::Fetching(_)) | (FS::Canceled, FS::Canceled) => true,
            _ => false
        }
    }
}

impl <T> FetchState<T> {
    pub fn new() -> FetchState<T> {
        FetchState::Unloaded
    }

    pub fn unload(&mut self) -> ShouldRender {
        *self = FetchState::Unloaded;
        true
    }

    pub fn fetching(&mut self, task: FetchTask) -> ShouldRender {
        *self = FetchState::Fetching(Rc::new(task));
        true
    }

    /// Will keep the old data around, while a new task is fetched.
    pub fn fetching_with_persistence(&mut self, task: FetchTask) -> ShouldRender {
        match self {
            FetchState::FetchingWithPersistence(data,_) | FetchState::Fetched(data) => {
                let data = data.take(); // TODO, consider making some local type wrapper around option<T> that can't be instantiated, but can be taken from. We don't want fetchstate to be able to be initialized with a None
                assert!(data.is_some());
                *self = FetchState::FetchingWithPersistence(data, Rc::new(task));
            }
            _ => {
                *self = FetchState::Fetching(Rc::new(task));
            }
        }
        true
    }

    pub fn failed(&mut self) -> ShouldRender {
        *self = FetchState::Failed;
        true
    }

    pub fn cancel(&mut self) -> ShouldRender {
        *self = FetchState::Canceled;
        true
    }
    pub fn fetched(&mut self, data: T) -> ShouldRender {
        *self = FetchState::Fetched(Some(Rc::new(data)));
        true
    }

//    pub fn unwrap(self) -> T {
//        match self {
//            FetchState::FetchingWithPersistence(data,_) | FetchState::Fetched(data) => data.unwrap(),
//            _ => panic!("Tried to unwrap Fetch state where no data was present.")
//        }
//    }
}

////////////////////////


#[derive(Properties)]
pub struct Fetch<T: 'static> {
    pub children: ChildrenRenderer<FetchVariant<T>>,
    #[props(required)]
    pub state: FetchState<T>
}

impl <T: 'static> Component for Fetch<T> {
    type Message = ();
    type Properties = Fetch<T>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        *self = props;
        true
    }
}

impl <T: 'static> Renderable<Fetch<T>> for Fetch<T> {
    fn view(&self) -> Html<Self> {
        match &self.state {
            FetchState::Unloaded => self.view_unloaded(),
            FetchState::Fetching(_) => self.view_fetching(),
            FetchState::FetchingWithPersistence(_, _) => unimplemented!(),
            FetchState::Failed => self.view_failed(),
            FetchState::Fetched(data) => self.view_fetched(data.as_ref().unwrap()),
            FetchState::Canceled => unimplemented!()
        }
    }
}

impl <T> Fetch<T> {
    fn view_unloaded(&self) -> Html<Self> {
        self.children
            .iter()
            .filter(|x| {
                if let Variants::Unloaded(_) = x.props {
                    true
                } else {
                    false
                }
            })
            .next() // Get only the first
            .into_iter()
            .collect()
    }

    fn view_fetching(&self) -> Html<Self> {
        self.children
            .iter()
            .filter(|x| {
                if let Variants::Fetching(_) = x.props {
                    true
                } else {
                    false
                }
            })
            .next() // Get only the first
            .into_iter()
            .collect() // Won't show anything if not specified.
    }

    fn view_failed(&self) -> Html<Self> {
        self.children
            .iter()
            .filter(|x| {
                if let Variants::Failed(_) = x.props {
                    true
                } else {
                    false
                }
            })
            .next() // Get only the first
            .into_iter()
            .collect()
    }

    fn view_fetched(&self, data: &Rc<T>) -> Html<Self> {
        self.children
            .iter()
            .filter_map(move |mut x| {
                if let Variants::Fetched(ref mut fetched) = x.props {
                    fetched.data = Some(data.clone());
                    Some(x)
                } else {
                    None
                }
            })
            .next() // Get only the first
            .into_iter()
            .collect()
    }
}




pub enum Variants<T: 'static> {
    Unloaded(<Unloaded as Component>::Properties),
    Fetching(<Fetching as Component>::Properties),
    Failed(<Failed as Component>::Properties),
    Fetched(<Fetched<T> as Component>::Properties),
    Canceled(<Canceled as Component>::Properties),
}

impl <T> From<UnloadedProps> for Variants<T> {
    fn from(props: UnloadedProps) -> Self {
        Variants::Unloaded(props)
    }
}


impl <T> From<FetchingProps> for Variants<T> {
    fn from(props: FetchingProps) -> Self {
        Variants::Fetching(props)
    }
}


impl <T> From<FailedProps> for Variants<T> {
    fn from(props: FailedProps) -> Self {
        Variants::Failed(props)
    }
}

impl <T> From<FetchedProps<T>> for Variants<T> {
    fn from(props: FetchedProps<T>) -> Self {
        Variants::Fetched(props)
    }
}

impl <T> From<CanceledProps> for Variants<T> {
    fn from(props: CanceledProps) -> Self {
        Variants::Canceled(props)
    }
}




pub struct FetchVariant<T: 'static> {
    props: Variants<T>,
    scope: ScopeHolder<Fetch<T>>,
}


impl<CHILD, T: 'static> From<VChild<CHILD, Fetch<T>>> for FetchVariant<T>
    where
        CHILD: Component,
        CHILD::Properties: Into<Variants<T>>,
{
    fn from(vchild: VChild<CHILD, Fetch<T>>) -> Self {
        FetchVariant {
            props: vchild.props.into(),
            scope: vchild.scope,
        }
    }
}


impl <T> Into<VNode<Fetch<T>>> for FetchVariant<T> {
    fn into(self) -> VNode<Fetch<T>> {
        match self.props {
            Variants::Unloaded(props) => VComp::new::<Unloaded>(props, self.scope).into(),
            Variants::Fetching(props) => VComp::new::<Fetching>(props, self.scope).into(),
            Variants::Failed(props) => VComp::new::<Failed>(props, self.scope).into(),
            Variants::Fetched(props) => VComp::new::<Fetched<T>>(props, self.scope).into(),
            Variants::Canceled(props) => VComp::new::<Canceled>(props, self.scope).into(),
            _ => unimplemented!()
        }
    }
}