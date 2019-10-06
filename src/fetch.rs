use crate::fetch::canceled::{Canceled, CanceledProps};
use crate::fetch::failed::{Failed, FailedProps};
use crate::fetch::fetched::{Fetched, FetchedProps};
use crate::fetch::fetching::{Fetching, FetchingProps};
use crate::fetch::persist_canceled::PersistCanceled;
use crate::fetch::persist_failed::{PersistFailed, PersistFailedProps};
use crate::fetch::persist_fetching::{PersistFetching, PersistFetchingProps};
use crate::fetch::unloaded::{Unloaded, UnloadedProps};
use std::rc::Rc;
use yew::html::ChildrenRenderer;
use yew::services::fetch::FetchTask;
use yew::virtual_dom::vcomp::ScopeHolder;
use yew::virtual_dom::VChild;
use yew::{
    virtual_dom::{VComp, VNode},
    Callback, Component, ComponentLink, Html, Properties, Renderable, ShouldRender,
};
use std::fmt;

pub mod canceled;
pub mod failed;
pub mod fetched;
pub mod fetching;
pub mod persist_canceled;
pub mod persist_failed;
pub mod persist_fetching;
pub mod unloaded;

/// The state of a fetch request.
#[derive(Clone, Debug, PartialEq)]
pub struct FetchState<T> {
    variant: FetchStateVariant<T>
}

/// All permutations of the various states a fetch request can be in.
#[derive(Clone)]
enum FetchStateVariant<T> {
    Unloaded,
    Fetching(Rc<FetchTask>),
    Failed(Rc<::failure::Error>),
    Canceled,
    Fetched(Option<Rc<T>>),
    PersistFetching(Option<Rc<T>>, Rc<FetchTask>),
    PersistFailed(Option<Rc<T>>, Rc<::failure::Error>),
    PersistCanceled(Option<Rc<T>>),
}

// TODO remove this when FetchTask gets Debug implemented for it.
impl <T: fmt::Debug> fmt::Debug for FetchStateVariant<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FetchStateVariant")
    }
}

impl<T> PartialEq for FetchStateVariant<T> {
    fn eq(&self, other: &Self) -> bool {
        use FetchStateVariant as FS;
        match (self, other) {
            (FS::Unloaded, FS::Unloaded)
            | (FS::Fetching(_), FS::Fetching(_))
            | (FS::Canceled, FS::Canceled) => true,
            _ => false,
        }
    }
}

impl<T: Clone> FetchState<T> {
    // TODO consider handing out weak pointers instead of other RCs to the child components, so get_mut can work instead of having to clone _every_time_.
    // There will likely have to be a WeakFetchState version used by this lib, and only obtainable via calling a method on the FetchState.
    // Clone will have to be removed for FetchState, and will be replaced with this get_weak() method.
    //
    // The problem with a weak version of this is that if the parent changes, but doesn't re-render,
    // then child components won't get access to the data anymore.
    // This poses the problem of allowing users to shoot themselves in the foot if they don't read the docs,
    // especially if this is the only way to do this.
    // Ideally, the child components could be generic over either Strong or Weak RCs -  preserving the ability to "clone" the parent FetchStateVariant,
    // but that would be kinda hard to implement.
    // Maybe a Strong/Weak enum? and have strong() and weak() items?
    pub fn make_mut(&mut self) -> Option<&mut T> {
        match &mut self.variant {
            FetchStateVariant::Fetched(Some(data))
            | FetchStateVariant::PersistFetching(Some(data), _)
            | FetchStateVariant::PersistFailed(Some(data), _)
            | FetchStateVariant::PersistCanceled(Some(data)) => Some(Rc::make_mut(data)),
            _ => None,
        }
    }
}

impl<T> FetchState<T> {
    pub fn get(&self) -> Option<&T> {
        match &self.variant {
            FetchStateVariant::Fetched(Some(data))
            | FetchStateVariant::PersistFetching(Some(data), _)
            | FetchStateVariant::PersistFailed(Some(data), _)
            | FetchStateVariant::PersistCanceled(Some(data)) => Some(data.as_ref()),
            _ => None,
        }
    }

    /// Creates an unleaded state.
    pub fn unloaded() -> Self {
        FetchState {
            variant: FetchStateVariant::Unloaded
        }
    }

    /// Creates a fetching state.
    pub fn fetching(task: FetchTask) -> Self {
        FetchState {
            variant: FetchStateVariant::Fetching(Rc::new(task))
        }
    }

    /// Creates a fetched state.
    pub fn fetched(data: T) -> Self {
        FetchState {
            variant: FetchStateVariant::Fetched(Some(Rc::new(data)))
        }
    }

    pub fn failed(error: ::failure::Error) -> Self {
        FetchState {
            variant: FetchStateVariant::Failed(Rc::new(error))
        }
    }

    pub fn canceled() -> Self {
        FetchState {
            variant: FetchStateVariant::Canceled
        }

    }

    // TODO consider making this take Self, task -> Self
    /// Will keep the old data around, while a new task is fetched.
    pub fn persist_fetching(&mut self, task: FetchTask) -> ShouldRender {
        match &mut self.variant {
            FetchStateVariant::Fetched(data)
            | FetchStateVariant::PersistFetching(data, _)
            | FetchStateVariant::PersistFailed(data, _)
            | FetchStateVariant::PersistCanceled(data) => {
                let data = data.take();
                assert!(data.is_some());
                self.variant = FetchStateVariant::PersistFetching(data, Rc::new(task));
            }
            _ => {
                self.variant = FetchStateVariant::Fetching(Rc::new(task));
            }
        }
        true
    }

    pub fn persist_failed(&mut self, error: ::failure::Error) -> ShouldRender {
        match &mut self.variant {
            FetchStateVariant::Fetched(data)
            | FetchStateVariant::PersistFetching(data, _)
            | FetchStateVariant::PersistFailed(data, _)
            | FetchStateVariant::PersistCanceled(data) => {
                let data = data.take();
                assert!(data.is_some());
                self.variant = FetchStateVariant::PersistFailed(data, Rc::new(error));
            }
            _ => {
                self.variant = FetchStateVariant::Failed(Rc::new(error));
            }
        }
        true
    }

    pub fn persist_canceled(&mut self) -> ShouldRender {
        match &mut self.variant {
            FetchStateVariant::Fetched(data)
            | FetchStateVariant::PersistFetching(data, _)
            | FetchStateVariant::PersistFailed(data, _)
            | FetchStateVariant::PersistCanceled(data) => {
                let data = data.take();
                assert!(data.is_some());
                self.variant = FetchStateVariant::PersistCanceled(data);
            }
            _ => {
                self.variant = FetchStateVariant::Canceled;
            }
        }
        true
    }

    pub fn unwrap(self) -> Rc<T> {
        match self.variant {
            FetchStateVariant::Fetched(data)
            | FetchStateVariant::PersistFetching(data, _)
            | FetchStateVariant::PersistFailed(data, _)
            | FetchStateVariant::PersistCanceled(data) => data.unwrap(),
            _ => panic!("Tried to unwrap Fetch state where no data was present."),
        }
    }
}

////////////////////////

#[derive(Properties)]
pub struct Fetch<T: 'static, M: 'static> {
    pub children: ChildrenRenderer<FetchVariant<T, M>>,
    #[props(required)]
    pub state: FetchState<T>,
    pub callback: Option<Callback<M>>,
}

impl<T: 'static, M: 'static> Component for Fetch<T, M> {
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

impl<T: 'static, M: 'static> Renderable<Fetch<T, M>> for Fetch<T, M> {
    fn view(&self) -> Html<Self> {
        match &self.state.variant {
            FetchStateVariant::Unloaded => self.view_unloaded(),
            FetchStateVariant::Fetching(_) => self.view_fetching(),
            FetchStateVariant::Failed(error) => self.view_failed(&error),
            FetchStateVariant::Canceled => self.view_canceled(),
            FetchStateVariant::Fetched(data) => self.view_fetched(data.as_ref().unwrap()),
            FetchStateVariant::PersistFetching(data, _) => {
                self.view_persist_fetching(data.as_ref().unwrap())
            }
            FetchStateVariant::PersistFailed(data, error) => self.view_persist_failed(data.as_ref().unwrap(), &error),
            FetchStateVariant::PersistCanceled(data) => self.view_persist_canceled(data.as_ref().unwrap()),
        }
    }
}

impl<T, M: 'static> Fetch<T, M> {
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

    fn view_failed(&self, error: &Rc<::failure::Error>) -> Html<Self> {
        self.children
            .iter()
            .filter_map(move |mut x| {
                if let Variants::Failed(ref mut failed) = x.props {
                    failed.callback = self.callback.clone();
                    failed.error = Some(error.clone());
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

    fn view_persist_failed(&self, data: &Rc<T>, error: &Rc<::failure::Error>) -> Html<Self> {
        self.children
            .iter()
            .filter_map(move |mut x| {
                if let Variants::PersistFailed(ref mut failed) = x.props {
                    failed.data = Some(data.clone());
                    failed.error = Some(error.clone());
                    failed.callback = self.callback.clone();
                    Some(x)
                } else {
                    None
                }
            })
            .next()
            .map(|v| v.into())
            .unwrap_or_else(|| self.view_failed(error)) // Default to viewing the failed case if no PersistFailed case is supplied
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

impl<T, M: 'static> From<UnloadedProps<M>> for Variants<T, M> {
    fn from(props: UnloadedProps<M>) -> Self {
        Variants::Unloaded(props)
    }
}

impl<T, M: 'static> From<FetchingProps<M>> for Variants<T, M> {
    fn from(props: FetchingProps<M>) -> Self {
        Variants::Fetching(props)
    }
}

impl<T, M: 'static> From<FailedProps<M>> for Variants<T, M> {
    fn from(props: FailedProps<M>) -> Self {
        Variants::Failed(props)
    }
}

impl<T, M: 'static> From<CanceledProps<M>> for Variants<T, M> {
    fn from(props: CanceledProps<M>) -> Self {
        Variants::Canceled(props)
    }
}
impl<T, M: 'static> From<FetchedProps<T, M>> for Variants<T, M> {
    fn from(props: FetchedProps<T, M>) -> Self {
        Variants::Fetched(props)
    }
}

impl<T, M: 'static> From<PersistFetchingProps<T, M>> for Variants<T, M> {
    fn from(props: PersistFetchingProps<T, M>) -> Self {
        Variants::PersistFetching(props)
    }
}

impl<T, M: 'static> From<PersistFailedProps<T, M>> for Variants<T, M> {
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

impl<T, M: 'static> Into<VNode<Fetch<T, M>>> for FetchVariant<T, M> {
    fn into(self) -> VNode<Fetch<T, M>> {
        match self.props {
            Variants::Unloaded(props) => VComp::new::<Unloaded<M>>(props, self.scope).into(),
            Variants::Fetching(props) => VComp::new::<Fetching<M>>(props, self.scope).into(),
            Variants::Failed(props) => VComp::new::<Failed<M>>(props, self.scope).into(),
            Variants::Canceled(props) => VComp::new::<Canceled<M>>(props, self.scope).into(),
            Variants::Fetched(props) => VComp::new::<Fetched<T, M>>(props, self.scope).into(),
            Variants::PersistFetching(props) => {
                VComp::new::<PersistFetching<T, M>>(props, self.scope).into()
            }
            Variants::PersistFailed(props) => {
                VComp::new::<PersistFailed<T, M>>(props, self.scope).into()
            }
            Variants::PersistCanceled(props) => {
                VComp::new::<PersistCanceled<T, M>>(props, self.scope).into()
            }
        }
    }
}
