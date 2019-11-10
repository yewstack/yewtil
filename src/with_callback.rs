use yew::{Callback, Properties, Html};
use crate::{Emissive, PureComponent, Pure, PureEmissiveComponent};


/// Helper struct for including callbacks in props.
///
/// This helps by allowing you to specify props separately from callbacks.
/// Keeping them separate should allow more ergonomic creation of large monolithic state structures at the
/// root component, as callbacks can be provided at at appropriate component levels in the app,
/// instead of at the root.
///
/// # Note
/// Unfortunately, due to coherence rules, this can't be used to crate pure components.
///
/// # Example
/// ```
///# use yewtil::{WithCallback, PureComponent, Pure};
///# use yew::{Html, Component, ComponentLink, html};
///# pub enum ModelMsg {}
///#
/// pub struct Model {
///     many: Vec<Data>
/// }
///
/// impl Component for Model {
///    type Message = ModelMsg;
///    type Properties = ();
///
///#   fn create(props: Self::Properties,link: ComponentLink<Self>) -> Self { unimplemented!()}
///#   fn update(&mut self,msg: Self::Message) -> bool {unimplemented!()}
///    // ...
///    fn view(&self) -> Html<Self> {
///        self.many.iter().map(|data| html!{
///            html!{ <InnerComponent data = data callback=From::from />}
///         }).collect()
///     }
/// }
///
/// #[derive(Clone)]
/// pub struct Data {
///     text: String,
///     size: usize
/// }
///
/// pub struct InnerComponent {
///    props: WithCallback<Data, ModelMsg>
/// }
/// impl Component for InnerComponent {
///    type Message = ModelMsg;
///    type Properties = WithCallback<Data, ModelMsg>;
///
///    // ...
///
///#    fn create(props: Self::Properties,link: ComponentLink<Self>) -> Self {unimplemented!()}
///    fn update(&mut self, msg: Self::Message) -> bool {
///        self.props.callback.emit(msg);
///        false
///     }
///
///    // ...
///#    fn view(&self) -> Html<Self> {unimplemented!()}
/// }
/// ```
#[derive(PartialEq, Properties, Clone, Debug)]
pub struct WithCallback<T, MSG> {
    /// The data.
    #[props(required)]
    pub data: T,
    /// The callback.
    #[props(required)]
    pub callback: Callback<MSG>
}

impl <T, MSG> Emissive for WithCallback<T, MSG> {
    type Message = MSG;

    fn emit(&self, msg: Self::Message) {
        self.callback.emit(msg)
    }
}

// TODO, the partialeq bound should not be needed.
//impl <T: PureComponent, MSG: 'static + PartialEq> PureComponent for WithCallback<T, MSG> {
//    fn render(&self) -> Html<Pure<Self>> {
//        self.data.render()
//    }
//}