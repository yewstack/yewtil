use yew::virtual_dom::VNode;
use yew::{html, Callback, Properties, ClickEvent, Html};
use yewtil::{Pure, PureComponent, function_component};

#[function_component(Button)]
pub fn button(
    #[props(required)]
    callback: Callback<ClickEvent>,
    text: String)
    -> Html {
    html! {
        <button onclick=callback>{ text }</button>
    }
}


