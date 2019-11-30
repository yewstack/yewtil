use crate::fetch::{FetchError, FetchAction};
use wasm_bindgen::JsValue;
use serde::{Serialize};
use serde::de::DeserializeOwned;
use web_sys::{Request, RequestInit, RequestMode, Response, Window};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// An enum representing what method to use for the request,
/// as well as a body if the method is able to have a body.
///
/// Connect, Options, Trace are omitted because they are unlikely to be used in this scenario.
/// Please open an issue if their absence is a problem for you.
pub enum MethodBody<'a, T> {
    Head,
    Get,
    Delete,
    Post(&'a T),
    Put(&'a T),
    Patch(&'a T),
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
    pub fn as_body<FORMAT: Format>(&self) -> Result<Option<JsValue>, FetchError> {
        let body: Option<String> = match self {
            MethodBody::Get
            | MethodBody::Delete
            | MethodBody::Head => None,
            MethodBody::Put(data)
            | MethodBody::Post(data)
            | MethodBody::Patch(data) => {
                let body = FORMAT::serialize(data)
                    .ok_or_else(|| FetchError::CouldNotSerializeRequestBody)?;
                Some(body)
            }
        };

        let body = body
            .map(|data| JsValue::from_str(data.as_str()));
        Ok(body)
    }
}


pub trait TransportMechanism {
    type SerOutput;
    type DeInput;
}

//pub struct Text;
//pub struct Bytes;
//impl TransportMechanism for Text {
//    type SerOutput = String;
//    type DeInput = str; // TODO &'static seems wrong.
//}
//
//impl TransportMechanism for Bytes {
//    type SerOutput = Vec<u8>;
//    type DeInput = [u8];
//}

/// Determines what format the data will be transmitted in.
pub trait Format {
//    type Transport: TransportMechanism;
    fn serialize<T: Serialize>(t: &T) -> Option<String>;
    fn deserialize<T: DeserializeOwned>(s: &str) -> Option<T>
//        where
//            I: AsRef<<Self::Transport as TransportMechanism>::DeInput>,
//            T: DeserializeOwned
    ;
}

/// Transport data using the JSON format
pub struct Json;
impl Format for Json {
//    type Transport = Text;

    fn serialize<T: Serialize>(t: &T) -> Option<String> {
        serde_json::to_string(t).ok()
    }

    fn deserialize<T: DeserializeOwned>(s: &str) -> Option<T> {
        serde_json::from_str(s).ok()
    }
}


/// Trait used to declare how a fetch request shall be made using a type.
pub trait FetchRequest {
    /// The Request Body (if any).
    type RequestBody: Serialize;
    /// The Response Body (if any).
    type ResponseBody: DeserializeOwned;

    /// What format to use for serialization and deserialization.
    ///
    /// Ideally default to serde_json::Deserializer once
    /// https://github.com/rust-lang/rust/issues/29661
    type Format: Format;

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

//    fn deserializer<E>(&self, response_text: &str) ->  impl Deserializer<Error=E> {
//        serde_json::Deserializer::from_str(response_text)
//    }
//    fn serializer(&self) -> Box<dyn Serializer> {
//        unimplemented!()
//    }
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
    opts.body(method.as_body::<T::Format>()?.as_ref());
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


    let deserialized = <T::Format>::deserialize(&text_string)
        .ok_or_else(|| {
            FetchError::DeserializeError{error: "".to_string(), content: text_string}
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

// TODO move this into the namespace of FetchAction.
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