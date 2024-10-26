use gloo::net::http::Request;
use serde::{Deserialize, Serialize};
use yew::platform::spawn_local;
use yew::{function_component, html, Callback, Html, InputEvent, TargetCast};

#[function_component]
fn App() -> Html {
    html! {
        <Frame />
    }
}

#[function_component]
fn Frame() -> Html {
    let short_url_state = yew::use_state(|| String::new());
    let long_url_state = yew::use_state(|| String::new());

    let long_url_listener = {
        let long_url_state = long_url_state.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                long_url_state.set(input.value())
            }
        })
    };

    let get_short_url = {
        let short_url_state = short_url_state.clone();
        let long_url_state = long_url_state.clone();

        Callback::from(move |_| {
            let short_url_state = short_url_state.clone();
            let long_url_state = long_url_state.clone();

            spawn_local(async move {
                let short_url_state = short_url_state.clone();
                let long_url_state = long_url_state.clone();

                let payload = ShortenRequest {
                    url: (*long_url_state).clone(),
                    domain: "localhost:8080".to_string(),
                };

                let response = Request::post("http://127.0.0.1:8080/api/v1/shorten")
                    .json(&payload)
                    .unwrap()
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        let short_url: ResponseContainer = resp.json().await.unwrap();

                        short_url_state.set(short_url.data.short_url);
                    }
                    Err(err) => gloo::console::info!("failed:", err.to_string()),
                }
            });
        })
    };

    html! {
        <div>
           <div>
                <label for="url">{ "url: " }</label>
                <input type="text" id="url" oninput={long_url_listener} />
            </div>
            <div>
                <label for="domain">{ "Choose a domain:" }</label>
                <select id="domain">
                    <option value="1" selected=true>{ "localhost:8080" }</option>
                    <option value="2">{ "localhost:8081" }</option>
                    <option value="3">{ "localhost:8082" }</option>
                </select>
            </div>
            <div>
                <button onclick={get_short_url}>{ "Shorten" }</button>
            </div>
            <div id="hide-me" hidden=false>
                <label for="shorten">{ "Short url: " }</label>
                <input type="text" id="shorten" value={(*short_url_state).clone()} readonly=true/>
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[derive(Deserialize, Serialize)]
struct ShortenRequest {
    pub domain: String,
    pub url: String,
}

#[derive(Deserialize, Serialize)]
struct ResponseContainer {
    pub data: ShortenResponse,
}

#[derive(Deserialize, Serialize)]
struct ShortenResponse {
    pub domain: String,
    pub alias: String,
    pub short_url: String,
}
