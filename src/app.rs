use std::fmt;

use crate::error_template::{ AppError, ErrorTemplate };
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{ Deserialize, Serialize };
use leptos_server_signal::create_server_signal;

#[cfg(feature = "ssr")]
use tokio::sync::mpsc;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum ActionMqtt {
    State(bool),
    Get,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DayPw {
    pub value: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LogData {
    pub timeon: String,
    pub totaltimeon: String,
    pub currenttimehours: String,
}

impl fmt::Display for LogData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "timeon: {}\n totaltimeon: {} \n current time: {}",
            self.timeon,
            self.totaltimeon,
            self.currenttimehours
        )
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RelayMqtt {
    pub value: String,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RebootMqtt {
    pub value: i64,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CurrentPw {
    pub value: String,
}

#[server(TurnOn, "/api")]
pub async fn set_pool(action: ActionMqtt) -> Result<(), ServerFnError> {
    //
    let tx = use_context::<mpsc::Sender<ActionMqtt>>()
        .ok_or_else(|| ServerFnError::ServerError("sender channel is missing!!".into()))
        .unwrap();
    if let Err(e) = tx.send(action).await {
        eprintln!("sender chan error (when sending){}", e);
    }

    //refer to counter isomorphic example for specifiing return type or error
    Ok(())
}

// the receive mqtt will be a server signal implemented by websockects
//no need for server funcinto

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    #[cfg(feature = "hydrate")]
    let window = web_sys::window().unwrap();
    #[cfg(feature = "hydrate")]
    let url = window.location().host().unwrap();
    #[cfg(feature = "hydrate")]
    leptos_server_signal
        ::provide_websocket(format!("ws://{}/ws", url).as_str())
        .unwrap();

    create_local_resource(|| (), |_|async move {
        spawn_local(async {
            let _ = set_pool(ActionMqtt::Get).await;
        });
    });
    view! {


        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/solar-frontend.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let get = move |_| {
        spawn_local(async {
            let _ = set_pool(ActionMqtt::Get).await;
        });
    };

    let on = move |_| {
        spawn_local(async {
            let _ = set_pool(ActionMqtt::State(true)).await;
        });
    };
    let off = move |_| {
        spawn_local(async {
            let _ = set_pool(ActionMqtt::State(false)).await;
        });
    };

    // Create server signal
    // or just create another signall for other things and rename this one
    let currentpwsignal = create_server_signal::<CurrentPw>("currentpwmqtt");
    let relaysignal = create_server_signal::<RelayMqtt>("relaymqtt");
    let daypwsignal = create_server_signal::<DayPw>("daypwmqtt");
    let logdatasignal = create_server_signal::<LogData>("logdatamqtt");
    let rebootsignal = create_server_signal::<RebootMqtt>("rebootmqtt");

    view! {
        <h1>"Relay State: "{move || relaysignal().value}</h1>
        <button on:click=on>"on pool"</button>
        <button on:click=off>"off pool"</button>

        <p>"Current power: "{move || currentpwsignal().value}</p>
        <p>"Day power: "{move || daypwsignal().value}</p>

        <h3>"Logs:"</h3>
        <p>"Current hour: "{move || logdatasignal().currenttimehours}</p>
        <p>"time on: "{move || logdatasignal().timeon}</p>
        <p>"total time on: "{move || logdatasignal().totaltimeon}</p>

        <p>"last reboot hour: "{move || rebootsignal().value}</p>






        <button on:click=get>"Reload values"</button>
    }
}
