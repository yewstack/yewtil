use crate::Msg::SetMarkdownFetchState;
use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yewtil::fetch::{FetchAction, FetchRequest, MethodBody};
use yewtil::fetch::fetch_to_state_msg;
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_app() {
    yew::start_app::<Model>();
}

struct Model {
    markdown: FetchAction<Vec<Employee>>,
    link: ComponentLink<Self>,
}

pub struct Request;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    id: String,
    employee_name: String,
    employee_salary: String,
    employee_age: String,
    profile_image: String
}

impl FetchRequest for Request {
    type RequestBody = ();
    type ResponseBody = Vec<Employee>;

    fn url(&self) -> String {
        // Given that this is an external resource, this may fail sometime in the future.
        // Please report any regressions related to this.
        "http://dummy.restapiexample.com/api/v1/employees".to_string()
    }

    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Get
    }

    fn headers(&self) -> Vec<(String, String)> {
        vec![]
    }

    fn use_cors(&self) -> bool {
        true
    }
}


enum Msg {
    SetMarkdownFetchState(FetchAction<Vec<Employee>>),
    GetMarkdown,
}

impl Component for Model {
    // Some details omitted. Explore the examples to see more.

    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model {
            markdown: FetchAction::NotFetching,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetMarkdownFetchState(fetch_state) => {
                self.markdown = fetch_state;
                true
            }
            Msg::GetMarkdown => {
                let request = Request;
                self.link.send_future(fetch_to_state_msg(request, Msg::SetMarkdownFetchState));
                self.link.send_self(SetMarkdownFetchState(FetchAction::Fetching));
                false
            }
        }
    }

    fn view(&self) -> Html<Self> {
        match &self.markdown {
            FetchAction::NotFetching => {
                html! {<button onclick=|_| Msg::GetMarkdown>{"Get employees"}</button>}
            }
            FetchAction::Fetching => html! {"Fetching"},
            FetchAction::Success(data) => data.iter().map(render_employee).collect(),
            FetchAction::Failed(err) => html! {&err},
        }
    }
}

fn render_employee(e: &Employee) -> Html<Model> {
    html! {
        <div>
            <div>
                {"Name: "}
                {&e.employee_name}
            </div>
            <div>
                {"Salary: "}
                {&e.employee_salary}
            </div>

            <div>
                {"Age: "}
                {&e.employee_age}
            </div>
            <br/>
        </div>
    }
}


