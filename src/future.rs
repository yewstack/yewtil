use std::future::Future;
use yew::{ComponentLink, Component, agent::{AgentLink, Agent}};
use stdweb::spawn_local;


/// Trait that allows you to use `ComponentLink`s to register futures.
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
            link.send_message(message);
        };
        spawn_local(js_future);

    }

    fn send_future_batch<F>(&self, future: F) where F: Future<Output=Vec<Self::Message>> + 'static {
        let mut link: ComponentLink<COMP> = self.clone();
        let js_future = async move {
            let messages: Vec<COMP::Message> = future.await;
            link.send_message_batch(messages);
        };
        spawn_local(js_future);
    }
}

/// Trait that allows you to use `AgentLink`s to register futures.
pub trait AgentLinkFuture {
    type Message;
    /// This method processes a Future that returns a message and sends it back to the component's
    /// loop.
    ///
    /// # Panics
    /// If the future panics, then the promise will not resolve, and will leak.
    fn send_future<F>(&self, future: F) where F: Future<Output = Self::Message> + 'static;
}

impl <AGN: Agent> AgentLinkFuture for AgentLink<AGN> {
    type Message = AGN::Message;

    fn send_future<F>(&self, future: F) where F: Future<Output=Self::Message> + 'static {
        let link: AgentLink<AGN>  = self.clone();
        let js_future = async move{
            let message: AGN::Message = future.await;
            let cb = link.callback(|m: AGN::Message| m);
            cb.emit(message);
        };
        spawn_local(js_future);
    }
}
