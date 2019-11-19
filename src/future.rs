use std::future::Future;
use yew::{ComponentLink, Component};

pub trait ComponentLinkFuture {
    type Message;
    /// This method processes a Future that returns a message and sends it back to the component's
    /// loop.
    ///
    /// # Panics
    /// If the future panics, then the promise will not resolve, and will leak.
    fn send_future<F>(&self, future: F) where F: Future<Output = Self::Message> + 'static;
}

impl <COMP: Component> ComponentLinkFuture for ComponentLink<COMP> {
    type Message = COMP::Message;

    fn send_future<F>(&self, future: F) where F: Future<Output=Self::Message> + 'static {
        let link: ComponentLink<COMP>  = self.clone();
        send_future_impl(link, future)
    }
}


fn send_future_impl<COMP: Component, F>(mut link: ComponentLink<COMP>, future: F)
    where F: Future<Output=COMP::Message> + 'static
{
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::future_to_promise;


    let js_future = async move{
        let message: COMP::Message = future.await;
        // Force movement of the cloned scope into the async block.
        let scope_send = move || link.send_self(message);
        scope_send();
        Ok(JsValue::NULL)
    };
    future_to_promise(js_future);
}
