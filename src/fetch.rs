//! Feature to enable fetching using web_sys-based fetch requests.
//!
//! Bodies will be serialized to JSON.
//!
//! # Note
//! Because this makes use of futures, enabling this feature will require the use of a
//! wasm-pack build environment and will prevent you from using `cargo web`.

use serde::Serialize;
use serde::de::DeserializeOwned;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, Window};
use crate::NeqAssign; // requires "neq" feature.

/// Indicates that a change was caused by a set function.
pub type DidChange = bool;


/// A fetch type that is useful for when you don't hold any request directly.
///
/// This is useful for GET and DELETE requests where additional information needed to create the request object
/// can be provided by a closure.
pub type AcquireFetch<T> = Fetch<(), T>;

/// A fetch type that is useful for when the request type is the same as the response type.
///
/// This makes sense to use when the request and response bodies are exactly the same.
/// Some PUT requests are amenable to this arrangement.
pub type ModifyFetch<T> = Fetch<T, T>;


#[derive(Clone, Debug, PartialEq, Default)]
pub struct Fetch<REQ, RES> {
    request: REQ,
    response: FetchState<RES>
}

/// Holds the state of the request being made and response
/// (if any has been made successfully at any prior point).
#[derive(Clone, Debug, PartialEq)]
pub enum FetchState<RES> {
    NotFetching(Option<RES>),
    Fetching(Option<RES>),
    Fetched(RES),
    Failed(Option<RES>, FetchError)
}

impl <RES> Default for FetchState<RES> {
    fn default() -> Self {
        FetchState::NotFetching(None)
    }
}


impl <REQ: PartialEq, RES> Fetch<REQ, RES> {

    /// Sets the request without changing the variant.
    pub fn set_req(&mut self, request: REQ) -> DidChange {
        self.request.neq_assign(request)
    }
}

impl <REQ: Default, RES: PartialEq> Fetch<REQ, RES> {

    /// Sets the Fetch wrapper to indicate that a request was successfully fetched.
    pub fn set_fetched(&mut self, res: RES) -> DidChange {
        let will_change = match &self.response {
            FetchState::Fetched(old_res) => {
                &res == old_res
            },
            _ => true
        };

        // TODO replace this with std::mem::take when it stabilizes.
        let old = std::mem::replace(&mut self.response, FetchState::default());
        let new = old.fetched(res);
        std::mem::replace(&mut self.response, new);

        will_change
    }

    /// Apply a FetchAction to alter the Fetch wrapper to perform a state change.
    pub fn apply(&mut self, action: FetchAction<RES>) -> DidChange {
        match action {
            FetchAction::NotFetching => self.set_not_fetching(),
            FetchAction::Fetching => self.set_fetching(),
            FetchAction::Success(res) => self.set_fetched(res),
            FetchAction::Failed(err) => self.set_failed(err),
        }
    }
}


impl <REQ, RES> Fetch<REQ, RES> {

    /// Creates a new Fetch wrapper around the request.
    ///
    /// It will default the response field to be put in a NotFetching state.
    pub fn new(request: REQ) -> Self {
        Self {
            request,
            response: Default::default()
        }
    }

    /// Sets the response field to indicate that no fetch request is in flight.
    pub fn set_not_fetching(&mut self) -> DidChange {
        let will_change = self.response.discriminant_differs(&FetchState::NotFetching(None));

        let old = std::mem::replace(&mut self.response, FetchState::default());
        let new = old.not_fetching();
        std::mem::replace(&mut self.response, new);

        will_change
    }

    /// Sets the response field to indicate that a fetch request is currently being made.
    pub fn set_fetching(&mut self) -> DidChange {
        let will_change = self.response.discriminant_differs(&FetchState::Fetching(None));

        let old = std::mem::replace(&mut self.response, FetchState::default());
        let new = old.fetching();
        std::mem::replace(&mut self.response, new);

        will_change
    }

    /// Sets the response field to indicate that a fetch request failed to complete.
    pub fn set_failed(&mut self, err: FetchError) -> DidChange {
        let will_change = match &self.response {
            FetchState::Failed(_, old_err) => {
                &err == old_err
            }
            _ => true
        };

        let old = std::mem::replace(&mut self.response, FetchState::default());
        let new = old.failed(err);
        std::mem::replace(&mut self.response, new);

        will_change
    }



    // TODO need tests to make sure that this is ergonomic.
    /// Makes an asynchronous fetch request, which will produce a message that makes use of a
    /// `FetchAction` when it completes.
    pub async fn fetch_convert<T: FetchRequest, Msg>(
        &self,
        to_request: impl Fn(&Self) -> &T,
        to_msg: impl Fn(FetchAction<T::ResponseBody>) -> Msg
    ) -> Msg {
        let request = to_request(self);
        let fetch_state = match fetch_resource(request).await {
            Ok(response) => FetchAction::Success(response),
            Err(err) => FetchAction::Failed(err)
        };

        to_msg(fetch_state)
    }

    /// Transforms the type of the response held by the fetch state (if any exists).
    pub fn map<NewRes, F: Fn(Fetch<REQ, RES>) -> Fetch<REQ, NewRes> >(self, f:F) -> Fetch<REQ, NewRes> {
        f(self)
    }

    /// Unwraps the Fetch wrapper to produce the response it may contain.
    ///
    /// # Panics
    /// If the Fetch wrapper doesn't contain an instance of a response, this function will panic.
    pub fn unwrap(self) -> RES {
        // TODO, actually provide some diagnostic here.
        self.res().unwrap()
    }

    /// Gets the response body (if present).
    pub fn res(self) -> Option<RES> {
        match self.response {
            FetchState::NotFetching(res) => res,
            FetchState::Fetching(res) => res,
            FetchState::Fetched(res) => Some(res),
            FetchState::Failed(res, _) => res,
        }
    }

    /// Gets the request body.
    pub fn req(self) -> REQ {
        self.request
    }

    /// Converts the wrapped values to references.
    ///
    /// # Note
    /// This may be expensive if a Failed variant made into a reference, as the FetchError is cloned.
    pub fn as_ref(&self) -> Fetch<&REQ, &RES> {
        let response = match &self.response {
            FetchState::NotFetching(res) => FetchState::NotFetching(res.as_ref()),
            FetchState::Fetching(res) => FetchState::Fetching(res.as_ref()),
            FetchState::Fetched(res) => FetchState::Fetched(res),
            FetchState::Failed(res, err) => FetchState::Failed(res.as_ref(), err.clone()),
        };

        Fetch {
            request: &self.request,
            response
        }
    }

    /// Converts the wrapped values to mutable references.
    ///
    /// # Note
    /// This may be expensive if a Failed variant made into a reference, as the FetchError is cloned.
    pub fn as_mut(&mut self) -> Fetch<&mut REQ, &mut RES> {
        let response = match &mut self.response {
            FetchState::NotFetching(res) => FetchState::NotFetching(res.as_mut()),
            FetchState::Fetching(res) => FetchState::Fetching(res.as_mut()),
            FetchState::Fetched(res) => FetchState::Fetched(res),
            FetchState::Failed(res, err) => FetchState::Failed(res.as_mut(), err.clone()),
        };
        Fetch {
            request: &mut self.request,
            response
        }
    }
}

impl <REQ: FetchRequest> Fetch<REQ, REQ::ResponseBody>{

    /// Makes an asynchronous fetch request, which will produce a message that makes use of a
    /// `FetchAction` when it completes.
    pub async fn fetch<Msg>(
        &self,
        to_msg: impl Fn(FetchAction<REQ::ResponseBody>) -> Msg
    )-> Msg {
        let request = self.as_ref().req();
        let fetch_state = match fetch_resource(request).await {
            Ok(response) => FetchAction::Success(response),
            Err(err) => FetchAction::Failed(err)
        };

        to_msg(fetch_state)
    }
}

impl <RES> FetchState<RES> {

    /// Determines if there is a different discriminant between the fetch states.
    fn discriminant_differs(&self, other: &Self) -> bool {
        std::mem::discriminant(self) != std::mem::discriminant(other)
    }

    fn not_fetching(self) -> Self {
        match self {
            FetchState::NotFetching(res) => {
                FetchState::NotFetching( res)
            }
            FetchState::Fetching(res) => {
                FetchState::NotFetching( res)
            }
            FetchState::Fetched(res) => {
                FetchState::NotFetching( Some(res))
            }
            FetchState::Failed(res, _err) => {
                FetchState::NotFetching( res)
            }
        }
    }

    fn fetching(self) -> Self {
        match self {
            FetchState::NotFetching(res) => {
                FetchState::Fetching(res)
            }
            FetchState::Fetching(res) => {
                FetchState::Fetching(res)
            }
            FetchState::Fetched(res) => {
                FetchState::Fetching(Some(res))
            }
            FetchState::Failed(res, _err) => {
                FetchState::Fetching(res)
            }
        }
    }

    fn fetched(self, res: RES) -> Self {
        match self {
            FetchState::NotFetching(_res) => {
                FetchState::Fetched(res)
            }
            FetchState::Fetching(_res) => {
                FetchState::Fetched(res)
            }
            FetchState::Fetched(_res) => {
                FetchState::Fetched(res)
            }
            FetchState::Failed(_res, _err) => {
                FetchState::Fetched(res)
            }
        }
    }

    fn failed(self, err: FetchError) -> Self {
        match self {
            FetchState::NotFetching(res) => {
                FetchState::Failed(res, err)
            }
            FetchState::Fetching(res) => {
                FetchState::Failed(res, err)
            }
            FetchState::Fetched(res) => {
                FetchState::Failed(Some(res), err)
            }
            FetchState::Failed(res, _err) => {
                FetchState::Failed(res, err)
            }
        }
    }
}


/// Represents a state change to Fetch wrapper.
#[derive(Clone, PartialEq, Debug)]
pub enum FetchAction<T> {
    NotFetching,
    Fetching,
    Success(T), // TODO rename to Fetched(T)
    Failed(FetchError),
}

impl <T> Default for FetchAction<T> {
    fn default() -> Self {
        FetchAction::NotFetching
    }
}

impl <T> FetchAction<T> {
    /// Returns a reference to the Success case
    pub fn success(&self) -> Option<&T> {
        match self {
            FetchAction::Success(value) => Some(value),
            _ => None
        }
    }

    /// Gets the value out of the fetch state if it is a `Success` variant.
    pub fn unwrap(self) -> T {
        if let FetchAction::Success(value) = self {
            value
        } else {
            panic!("Could not unwrap value of FetchState");
        }
    }

    /// Transforms the FetchState into another FetchState using the given function.
    pub fn map<U, F: Fn(T)-> U>(self, f: F ) -> FetchAction<U> {
        match self {
            FetchAction::NotFetching => FetchAction::NotFetching,
            FetchAction::Fetching => FetchAction::NotFetching,
            FetchAction::Success(t) => FetchAction::Success(f(t)),
            FetchAction::Failed(e) => FetchAction::Failed(e)
        }
    }

    /// Applies a function that mutates the response if the Action is the success case.
    pub fn alter<F: Fn(&mut T)>(&mut self, f: F) {
        match self {
            FetchAction::Success(t) => f(t),
            _ => {}
        }
    }

    /// Converts the FetchAction to contain a reference to the success case.
    pub fn as_ref(&self) -> FetchAction<&T>  {
        match self {
            FetchAction::NotFetching => FetchAction::NotFetching,
            FetchAction::Fetching => FetchAction::NotFetching,
            FetchAction::Success(t) => FetchAction::Success(t),
            FetchAction::Failed(e) => FetchAction::Failed(e.clone())
        }
    }
}

impl <T: PartialEq> FetchAction<T> {
    /// Sets the fetch state to be fetching.
    /// If it wasn't already in a fetch state, it will return `true`,
    /// to indicate that the component should re-render.
    pub fn set_fetching(&mut self) -> bool {
        self.neq_assign(FetchAction::Fetching)
    }
}


// TODO add remaining HTTP methods.
/// An enum representing what method to use for the request,
/// as well as a body if the method is able to have a body.
pub enum MethodBody<'a, T> {
    Head,
    Get,
    Delete,
    Post(&'a T),
    Put(&'a T),
    Patch(&'a T)
}

impl <'a, T> MethodBody<'a, T> {
    pub fn as_method(&self) -> &'static str {
        match self {
            MethodBody::Get => "GET",
            MethodBody::Delete => "DELETE",
            MethodBody::Post(_) => "POST",
            MethodBody::Put(_) => "PUT",
            MethodBody::Patch(_) => "PATCH",
            MethodBody::Head => "HEAD",
        }
    }
}

impl <'a, T: Serialize> MethodBody<'a, T> {
    pub fn as_body(&self) -> Result<Option<JsValue>, FetchError> {
        let body: Option<String> = match self {
            MethodBody::Get
            | MethodBody::Delete
            | MethodBody::Head => None,
            MethodBody::Put(data)
            | MethodBody::Post(data)
            | MethodBody::Patch(data) => {
                let body = serde_json::to_string(data)
                    .map_err(|_| FetchError::CouldNotSerializeRequestBody)?;
                Some(body)
            }
        };

        let body = body
            .map(|data| JsValue::from_str(data.as_str()));
        Ok(body)
    }
}

/// A representation of an error that may occur when making a fetch request.
#[derive(Debug, PartialEq, Clone)]
pub enum FetchError {
    /// The response could not be deserialized.
    DeserializeError{error: String, content: String},
    /// The response had an error code.
    ResponseError{status_code: u16, response_body: String},
    /// Text was not available on the response.
    // TODO, this might get thrown in unexpected circumstances.
    TextNotAvailable,
    /// The Fetch Future could not be created due to a misconfiguration.
    CouldNotCreateFetchFuture,
    /// The request could cont be created due to a misconfiguration.
    CouldNotCreateRequest(JsValue), // TODO, convert this to a string or more structured error - implement Hash on this and related structs.
    /// Could not serialize the request body.
    CouldNotSerializeRequestBody
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::DeserializeError {error, content} => {
                f.write_str(&format!("Could not deserialize a successful request. With error: {}, and content: {}", error, content))
            }
            FetchError::ResponseError { status_code, response_body} => {
                f.write_str(&format!("The server returned a response with code: {}, and body: {}", status_code, response_body))
            }
            FetchError::TextNotAvailable => {
                f.write_str("The text could not be extracted from the response.")
            }
            FetchError::CouldNotCreateFetchFuture => {
                f.write_str("Could not create a fetch future.")
            }
            FetchError::CouldNotCreateRequest(_) => {
                f.write_str("Could not create a fetch request.")
            }
            FetchError::CouldNotSerializeRequestBody => {
                f.write_str("Could not serialize the body in the fetch request.")
            }
        }
    }
}

impl std::error::Error for FetchError {
}

/// Trait used to declare how a fetch request shall be made using a type.
pub trait FetchRequest {
    /// The Request Body (if any).
    type RequestBody: Serialize;
    /// The Response Body (if any).
    type ResponseBody: DeserializeOwned;

    /// The URL of the resource to fetch.
    fn url(&self) -> String;

    /// The HTTP method and body (if any) to be used in constructing the request.
    fn method(&self) -> MethodBody<Self::RequestBody>;

    /// The headers to attach to the request .
    fn headers(&self) -> Vec<(String, String)>;

    /// Use CORS for the request. By default, it will not.
    fn use_cors(&self) -> bool {
        false
    }
}

/// Fetch a resource, returning a result of the expected response,
/// or an error indicating what went wrong.
pub async fn fetch_resource<T: FetchRequest>(request: &T) -> Result<T::ResponseBody, FetchError> {
    let method = request.method();
    let headers = request.headers();
    let headers = JsValue::from_serde(&headers).expect("Convert Headers to Tuple");

    // configure options for the request
    let mut opts = RequestInit::new();
    opts.method(method.as_method());
    opts.body(method.as_body()?.as_ref());
    opts.headers(&headers);

    // TODO, see if there are more options that can be specified.
    if request.use_cors() {
        opts.mode(RequestMode::Cors);
    }

    // Create the request
    let request = Request::new_with_str_and_init(
        &request.url(),
        &opts,
    )
        .map_err(|e| FetchError::CouldNotCreateRequest(e))?; // TODO make this a Rust value instead.


    // Send the request, resolving it to a response.
    let window: Window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| FetchError::CouldNotCreateFetchFuture)?;
    debug_assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();


    // Process the response
    let text = JsFuture::from(resp.text().map_err(|_| FetchError::TextNotAvailable)?)
        .await
        .map_err(|_| FetchError::TextNotAvailable)?;

    let text_string = text.as_string().unwrap();

    // If the response isn't ok, then return an error without trying to deserialize.
    if !resp.ok() {
        return Err(FetchError::ResponseError {status_code: resp.status(), response_body: text_string})
    }

    let deserialized = serde_json::from_str(&text_string)
        .map_err(|e| {
            FetchError::DeserializeError{error: e.to_string(), content: text_string}
        })?;

    Ok(deserialized)
}

/// Performs a fetch and then resolves the fetch to a message by way of using two provided Fns to
/// convert the success and failure cases.
///
/// This is useful if you want to handle the success case and failure case separately.
pub async fn fetch_to_msg<T: FetchRequest, Msg>(request: &T, success: impl Fn(T::ResponseBody) -> Msg, failure: impl Fn(FetchError) -> Msg) -> Msg {
    fetch_resource(request)
        .await
        .map(success)
        .unwrap_or_else(failure)
}

/// Performs a fetch and resolves the fetch to a message by converting a FetchState into the Message
/// by way of a provided closure.
///
/// This is useful if you just want to update a FetchState in your model based on the result of your request.
pub async fn fetch_to_state_msg<T: FetchRequest, Msg>(request: &T, to_msg: impl Fn(FetchAction<T::ResponseBody>) -> Msg) -> Msg {
    let fetch_state = match fetch_resource(request).await {
        Ok(response) => FetchAction::Success(response),
        Err(err) => FetchAction::Failed(err)
    };

    to_msg(fetch_state)
}


#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn setting_fetching_state_doesnt_change_strong_count() {
        // This is done to detect if a leak occurred.
        let data: Arc<i32> = Arc::new(22);
        let cloned_data: Arc<i32> = data.clone();
        assert_eq!(Arc::strong_count(&data), 2);
        let mut fs: Fetch<Arc<i32>, ()> = Fetch::new(cloned_data);
        fs.set_fetching();

        assert_eq!(Arc::strong_count(&data), 2);
        assert_eq!(Fetch::Fetching(Arc::new(22), None), fs)
    }

    #[test]
    fn setting_fetched_state() {
        let mut fs = Fetch::Fetching((), None);
        assert!(fs.set_fetched("SomeValue".to_string()));
        assert_eq!(fs, Fetch::Fetched((), "SomeValue".to_string()));
    }

    #[test]
    fn setting_fetching_from_fetched() {
        let mut fs = Fetch::Fetched((), "Lorem".to_string());
        assert!(fs.set_fetching());
        assert_eq!(fs, Fetch::Fetching((), Some("Lorem".to_string())));
    }
}
