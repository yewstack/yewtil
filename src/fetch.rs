use yew::services::fetch::FetchTask;
use yew::{ShouldRender, Properties, Component, Html, Renderable, ComponentLink, virtual_dom::{VComp, VNode}, Callback};
use yew::virtual_dom::vcomp::ScopeHolder;
use yew::virtual_dom::VChild;
use yew::html::ChildrenRenderer;
use crate::fetch::unloaded::{Unloaded, UnloadedProps};
use crate::fetch::fetching::{Fetching, FetchingProps};
use crate::fetch::failed::{Failed, FailedProps};
use crate::fetch::fetched::{Fetched, FetchedProps};
use std::rc::Rc;
use crate::fetch::canceled::{Canceled, CanceledProps};
use crate::fetch::persist_fetching::{PersistFetching, PersistFetchingProps};
use crate::fetch::persist_failed::{PersistFailedProps, PersistFailed};
use crate::fetch::persist_canceled::PersistCanceled;

pub mod unloaded;
pub mod fetching;
pub mod failed;
pub mod fetched;
pub mod canceled;
pub mod persist_fetching;
pub mod persist_failed;
pub mod persist_canceled;

// TODO, Wrap this in a newtype to make it impossible to instantiate for users.
/// Wraps all permutations of the various states a fetch request can be in.
#[derive(Clone)]
pub enum FetchState<T> {
    Unloaded,
    Fetching(Rc<FetchTask>),
    Failed, // TODO include error here.
    Canceled,
    Fetched(Option<Rc<T>>),
    PersistFetching(Option<Rc<T>>, Rc<FetchTask>),
    PersistFailed(Option<Rc<T>>),
    PersistCanceled(Option<Rc<T>>),
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


    pub fn fetched(&mut self, data: T) -> ShouldRender {
        *self = FetchState::Fetched(Some(Rc::new(data)));
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

    /// Will keep the old data around, while a new task is fetched.
    pub fn persist_fetching(&mut self, task: FetchTask) -> ShouldRender {
        match self {
            FetchState::Fetched(data)
            | FetchState::PersistFetching(data, _)
            | FetchState::PersistFailed(data)
            | FetchState::PersistCanceled(data)
            => {
                let data = data.take(); // TODO, consider making some local type wrapper around option<T> that can't be instantiated, but can be taken from. We don't want fetchstate to be able to be initialized with a None
                assert!(data.is_some());
                *self = FetchState::PersistFetching(data, Rc::new(task));
            }
            _ => {
                *self = FetchState::Fetching(Rc::new(task));
            }
        }
        true
    }


    pub fn persist_failed(&mut self) -> ShouldRender {
        match self {
            FetchState::Fetched(data)
            | FetchState::PersistFetching(data, _)
            | FetchState::PersistFailed(data)
            | FetchState::PersistCanceled(data)
            => {
                let data = data.take(); // TODO, consider making some local type wrapper around option<T> that can't be instantiated, but can be taken from. We don't want fetchstate to be able to be initialized with a None
                assert!(data.is_some());
                *self = FetchState::PersistFailed(data);
            }
            _ => {
                *self = FetchState::Failed;
            }
        }
        true
    }


    pub fn persist_canceled(&mut self) -> ShouldRender {
        match self {
            FetchState::Fetched(data)
            | FetchState::PersistFetching(data, _)
            | FetchState::PersistFailed(data)
            | FetchState::PersistCanceled(data)
            => {
                let data = data.take(); // TODO, consider making some local type wrapper around option<T> that can't be instantiated, but can be taken from. We don't want fetchstate to be able to be initialized with a None
                assert!(data.is_some());
                *self = FetchState::PersistCanceled(data);
            }
            _ => {
                *self = FetchState::Canceled;
            }
        }
        true
    }

    pub fn unwrap(self) -> Rc<T> {
        match self {
            FetchState::Fetched(data)
            | FetchState::PersistFetching(data,_)
            | FetchState::PersistFailed(data)
            | FetchState::PersistCanceled(data)
            => data.unwrap(),
            _ => panic!("Tried to unwrap Fetch state where no data was present.")
        }
    }
}

////////////////////////


#[derive(Properties)]
pub struct Fetch<T: 'static, M: 'static> {
    pub children: ChildrenRenderer<FetchVariant<T, M>>,
    #[props(required)]
    pub state: FetchState<T>,
    pub callback: Option<Callback<M>>
}

impl <T: 'static, M: 'static> Component for Fetch<T, M> {
    type Message = M;
    type Properties = Fetch<T, M>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        if let Some(callback) = &self.callback {
            callback.emit(msg)
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        *self = props;
        true
    }
}

impl <T: 'static, M: 'static> Renderable<Fetch<T, M>> for Fetch<T, M> {
    fn view(&self) -> Html<Self> {
        match &self.state {
            FetchState::Unloaded => self.view_unloaded(),
            FetchState::Fetching(_) => self.view_fetching(),
            FetchState::Failed => self.view_failed(),
            FetchState::Canceled => self.view_canceled(),
            FetchState::Fetched(data) => self.view_fetched(data.as_ref().unwrap()),
            FetchState::PersistFetching(data, _) => self.view_persist_fetching(data.as_ref().unwrap()),
            FetchState::PersistFailed(data) => self.view_persist_failed(data.as_ref().unwrap()),
            FetchState::PersistCanceled(data) => self.view_persist_canceled(data.as_ref().unwrap())
        }
    }
}

impl <T, M: 'static> Fetch<T, M> {
    fn view_unloaded(&self) -> Html<Self> {
        self.children
            .iter()
            .filter_map(move |mut x| {
                if let Variants::Unloaded(ref mut unloaded) = x.props {
                    unloaded.callback = self.callback.clone();
                    Some(x)
                } else {
                    None
                }
            })
            .next() // Get only the first
            .into_iter()
            .collect()
    }

    fn view_fetching(&self) -> Html<Self> {
        self.children
            .iter()
            .filter_map(move |mut x| {
                if let Variants::Fetching(ref mut fetching) = x.props {
                    fetching.callback = self.callback.clone();
                    Some(x)
                } else {
                    None
                }
            })
            .next() // Get only the first
            .into_iter()
            .collect() // Won't show anything if not specified.
    }

    fn view_failed(&self) -> Html<Self> {
        self.children
            .iter()
            .filter_map(move |mut x| {
                if let Variants::Failed(ref mut failed) = x.props {
                    failed.callback = self.callback.clone();
                    Some(x)
                } else {
                    None
                }
            })
            .next() // Get only the first
            .into_iter()
            .collect()
    }

    fn view_canceled(&self) -> Html<Self> {
        self.children
            .iter()
            .filter_map(move |mut x| {
                if let Variants::Canceled(ref mut canceled) = x.props {
                    canceled.callback = self.callback.clone();
                    Some(x)
                } else {
                    None
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
                    fetched.callback = self.callback.clone();
                    Some(x)
                } else {
                    None
                }
            })
            .next() // Get only the first
            .into_iter()
            .collect()
    }

    fn view_persist_fetching(&self, data: &Rc<T>) -> Html<Self> {
        self.children
            .iter()
            .filter_map(move |mut x| {
                if let Variants::PersistFetching(ref mut fetched) = x.props {
                    fetched.data = Some(data.clone());
                    fetched.callback = self.callback.clone();
                    Some(x)
                } else {
                    None
                }
            })
            .next() // Get only the first
            .map(|v| v.into())
            .unwrap_or_else(|| self.view_fetching())
    }


    fn view_persist_failed(&self, data: &Rc<T>) -> Html<Self> {
        self.children
            .iter()
            .filter_map(move |mut x| {
                if let Variants::PersistFailed(ref mut failed) = x.props {
                    failed.data = Some(data.clone());
                    failed.callback = self.callback.clone();
                    Some(x)
                } else {
                    None
                }
            })
            .next()
            .map(|v| v.into())
            .unwrap_or_else(|| self.view_failed()) // Default to viewing the failed case if no PersistFailed case is supplied
    }

    fn view_persist_canceled(&self, data: &Rc<T>) -> Html<Self> {
        self.children
            .iter()
            .filter_map(move |mut x| {
                if let Variants::PersistCanceled(ref mut canceled) = x.props {
                    canceled.data = Some(data.clone());
                    canceled.callback = self.callback.clone();
                    Some(x)
                } else {
                    None
                }
            })
            .next()
            .map(|v| v.into())
            .unwrap_or_else(|| self.view_canceled()) // Default to viewing the canceled case if no PersistCanceled case is supplied
    }
}




pub enum Variants<T: 'static, M: 'static> {
    Unloaded(<Unloaded<M> as Component>::Properties),
    Fetching(<Fetching<M> as Component>::Properties),
    Failed(<Failed<M> as Component>::Properties),
    Fetched(<Fetched<T, M> as Component>::Properties),
    Canceled(<Canceled<M> as Component>::Properties),
    PersistFetching(<PersistFetching<T, M> as Component>::Properties),
    PersistFailed(<PersistFailed<T, M> as Component>::Properties),
    PersistCanceled(<PersistCanceled<T, M> as Component>::Properties),
}

impl <T, M: 'static> From<UnloadedProps<M>> for Variants<T, M> {
    fn from(props: UnloadedProps<M>) -> Self {
        Variants::Unloaded(props)
    }
}


impl <T, M: 'static> From<FetchingProps<M>> for Variants<T, M> {
    fn from(props: FetchingProps<M>) -> Self {
        Variants::Fetching(props)
    }
}


impl <T, M: 'static> From<FailedProps<M>> for Variants<T, M> {
    fn from(props: FailedProps<M>) -> Self {
        Variants::Failed(props)
    }
}



impl <T, M: 'static> From<CanceledProps<M>> for Variants<T, M> {
    fn from(props: CanceledProps<M>) -> Self {
        Variants::Canceled(props)
    }
}
impl <T, M: 'static> From<FetchedProps<T, M>> for Variants<T, M> {
    fn from(props: FetchedProps<T, M>) -> Self {
        Variants::Fetched(props)
    }
}

impl <T, M: 'static> From<PersistFetchingProps<T, M>> for Variants<T, M> {
    fn from(props: PersistFetchingProps<T, M>) -> Self {
        Variants::PersistFetching(props)
    }
}

impl <T, M: 'static> From<PersistFailedProps<T, M>> for Variants<T, M> {
    fn from(props: PersistFailedProps<T, M>) -> Self {
        Variants::PersistFailed(props)
    }
}




pub struct FetchVariant<T: 'static, M: 'static> {
    props: Variants<T, M>,
    scope: ScopeHolder<Fetch<T, M>>,
}


impl<CHILD, T: 'static, M: 'static> From<VChild<CHILD, Fetch<T, M>>> for FetchVariant<T, M>
    where
        CHILD: Component,
        CHILD::Properties: Into<Variants<T, M>>,
{
    fn from(vchild: VChild<CHILD, Fetch<T, M>>) -> Self {
        FetchVariant {
            props: vchild.props.into(),
            scope: vchild.scope,
        }
    }
}


impl <T, M: 'static> Into<VNode<Fetch<T, M>>> for FetchVariant<T, M> {
    fn into(self) -> VNode<Fetch<T, M>> {
        match self.props {
            Variants::Unloaded(props) => VComp::new::<Unloaded<M>>(props, self.scope).into(),
            Variants::Fetching(props) => VComp::new::<Fetching<M>>(props, self.scope).into(),
            Variants::Failed(props) => VComp::new::<Failed<M>>(props, self.scope).into(),
            Variants::Canceled(props) => VComp::new::<Canceled<M>>(props, self.scope).into(),
            Variants::Fetched(props) => VComp::new::<Fetched<T, M>>(props, self.scope).into(),
            Variants::PersistFetching(props) => VComp::new::<PersistFetching<T, M>>(props, self.scope).into(),
            Variants::PersistFailed(props) => VComp::new::<PersistFailed<T, M>>(props, self.scope).into(),
            Variants::PersistCanceled(props) => VComp::new::<PersistCanceled<T, M>>(props, self.scope).into(),
        }
    }
}

