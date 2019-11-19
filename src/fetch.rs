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
use crate::NeqAssign;


#[derive(Clone, PartialEq, Debug)]
pub enum FetchState<T> {
    NotFetching,
    Fetching,
    Success(T),
    Failed(FetchError),
}

impl <T> Default for FetchState<T> {
    fn default() -> Self {
        FetchState::NotFetching
    }
}

impl <T> FetchState<T> {
    pub fn success(&self) -> Option<&T> {
        match self {
            FetchState::Success(value) => Some(value),
            _ => None
        }
    }

    /// Gets the value out of the fetch state if it is a `Success` variant.
    pub fn unwrap(self) -> T {
        if let FetchState::Success(value) = self {
            value
        } else {
            panic!("Could not unwrap value of FetchState");
        }
    }

    /// Transforms the FetchState into another FetchState using the given function.
    pub fn map<U, F: Fn(T)-> U>(self, f: F ) -> FetchState<U> {
        match self {
            FetchState::NotFetching => FetchState::NotFetching,
            FetchState::Fetching => FetchState::NotFetching,
            FetchState::Success(t) => FetchState::Success(f(t)),
            FetchState::Failed(e) => FetchState::Failed(e)
        }
    }

    pub fn alter<F: Fn(&mut T)>(&mut self, f: F) {
        match self {
            FetchState::Success(t) => f(t),
            _ => {}
        }
    }

    pub fn as_ref(&self) -> FetchState<&T>  {
        match self {
            FetchState::NotFetching => FetchState::NotFetching,
            FetchState::Fetching => FetchState::NotFetching,
            FetchState::Success(t) => FetchState::Success(t),
            FetchState::Failed(e) => FetchState::Failed(e.clone())
        }
    }
}

impl <T: PartialEq> FetchState<T> {
    /// Sets the fetch state to be fetching.
    /// If it wasn't already in a fetch state, it will return `true`,
    /// to indicate that the component should re-render.
    pub fn set_fetching(&mut self) -> bool {
        self.neq_assign(FetchState::Fetching)
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
    CouldNotCreateRequest(JsValue),
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
pub async fn fetch_resource<T: FetchRequest>(request: T) -> Result<T::ResponseBody, FetchError> {
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
pub async fn fetch_to_msg<T: FetchRequest, Msg>(request: T, success: impl Fn(T::ResponseBody) -> Msg, failure: impl Fn(FetchError) -> Msg) -> Msg {
    fetch_resource(request)
        .await
        .map(success)
        .unwrap_or_else(failure)
}

/// Performs a fetch and resolves the fetch to a message by converting a FetchState into the Message
/// by way of a provided closure.
///
/// This is useful if you just want to update a FetchState in your model based on the result of your request.
pub async fn fetch_to_state_msg<T: FetchRequest, Msg>(request: T, to_msg: impl Fn(FetchState<T::ResponseBody>) -> Msg) -> Msg {
    let fetch_state = match fetch_resource(request).await {
        Ok(response) => FetchState::Success(response),
        Err(err) => FetchState::Failed(err)
    };

    to_msg(fetch_state)
}