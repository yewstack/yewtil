use std::future::Future;
use yew::{ComponentLink, Component};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::future_to_promise;

pub trait ComponentLinkFuture {
    type Message;
    /// This method processes a Future that returns a message and sends it back to the component's
    /// loop.
    ///
    /// # Panics
    /// If the future panics, then the promise will not resolve, and will leak.
    fn send_future<F>(&self, future: F) where F: Future<Output = Self::Message> + 'static;

    /// Registers a future that resolves to multiple messages.
    /// # Panics
    /// If the future panics, then the promise will not resolve, and will leak.
    fn send_future_batch<F>(&self, future: F) where F: Future<Output=Vec<Self::Message>> + 'static;
}

impl <COMP: Component> ComponentLinkFuture for ComponentLink<COMP> {
    type Message = COMP::Message;

    fn send_future<F>(&self, future: F) where F: Future<Output=Self::Message> + 'static {
        let mut link: ComponentLink<COMP>  = self.clone();
        let js_future = async move{
            let message: COMP::Message = future.await;
            link.send_self(message);
            Ok(JsValue::NULL)
        };
        future_to_promise(js_future);

    }

    fn send_future_batch<F>(&self, future: F) where F: Future<Output=Vec<Self::Message>> + 'static {
        let mut link: ComponentLink<COMP> = self.clone();
        let js_future = async move {
            let messages: Vec<COMP::Message> = future.await;
            link.send_self_batch(messages);
            Ok(JsValue::NULL)
        };
        future_to_promise(js_future);
    }
}
