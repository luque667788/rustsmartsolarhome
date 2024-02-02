//the signal change but protected rout do not update!!!

use cfg_if::cfg_if;

use crate::error_template::{ AppError, ErrorTemplate };
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use leptos_server_signal::create_server_signal;
use crate::auth;
use crate::models::*;

#[cfg(feature = "ssr")]
use tokio::sync::mpsc;
#[cfg(feature = "ssr")]
use std::sync::Arc;
#[cfg(feature = "ssr")]
use tokio::sync::Mutex;

#[server(GetReboot)]
pub async fn get_reboothour() -> Result<String, ServerFnError> {
    //
    if let Some(req) = leptos::use_context::<leptos_axum::RequestParts>() {
        if auth::isloged_fn(&req.headers).await {
            let hour = use_context::<Arc<Mutex<String>>>().ok_or_else(||
                ServerFnError::ServerError("could not get lasthour".into())
            );
            return match hour {
                Ok(h) => {
                     Ok(h.lock().await.clone())
                }
                Err(e) =>{
                     Err(e)
                }
            }
            
            //refer to counter isomorphic example for specifiing return type or error
        } else {
            eprintln!("must be loged in");
        }
    } else {
        eprintln!("error fecthing headers cookie");
    }

    Ok("could not get login info".to_string())
}

#[server(TurnOn)]
pub async fn set_pool(action: ActionMqtt) -> Result<(), ServerFnError> {
    //
    if let Some(req) = leptos::use_context::<leptos_axum::RequestParts>() {
        if auth::isloged_fn(&req.headers).await {
            let tx = use_context::<mpsc::Sender<ActionMqtt>>()
                .ok_or_else(|| ServerFnError::ServerError("sender channel is missing!!".into()))
                .unwrap();
            if let Err(e) = tx.send(action).await {
                eprintln!("sender chan error (when sending){}", e);
            }

            //refer to counter isomorphic example for specifiing return type or error
        } else {
            eprintln!("must be loged in");
        }
    } else {
        eprintln!("error fecthing headers cookie");
    }

    Ok(())
}

#[server(Settings)]
pub async fn set_params(
    mininterval: Option<i32>,
    mintimeon: Option<i32>,
    minpw: Option<i32>
) -> Result<(), ServerFnError> {
    //
    if let Some(req) = leptos::use_context::<leptos_axum::RequestParts>() {
        if auth::isloged_fn(&req.headers).await {
            let params = ParamsJson {
                mintimeon: mintimeon,
                mininterval: mininterval,
                minpw: minpw,
            };
            println!("setting mintimeon: {:#?}", mintimeon);
            println!("setting mininterval: {:#?}", mininterval);
            println!("setting minpw: {:#?}", minpw);
            let tx = use_context::<mpsc::Sender<ParamsJson>>()
                .ok_or_else(||
                    ServerFnError::ServerError("sender params channel is missing!!".into())
                )
                .unwrap();
            if let Err(e) = tx.send(params).await {
                eprintln!("sender params chan error (when sending){}", e);
            }
        } else {
            eprintln!("must be loged in");
        }
    } else {
        eprintln!("error fecthing headers cookie");
    }

    Ok(())
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let login = create_server_action::<auth::Login>();
    let logout = create_server_action::<auth::Logout>();

    let (login_status, set_login_status) = create_signal(false);

    let logout_fn = move |_| {
        logout.dispatch(auth::Logout {});
    };

    let user = create_resource(
        move || { (logout.version().get(), login.version().get()) },
        move |_| async move {
            let a = auth::current_user().await;
            match a {
                Ok(_) => set_login_status(true),
                Err(_) => set_login_status(false),
            }
            a
        }
    );

    create_local_resource(
        || (),
        move |_| async move {
            spawn_local(async {
                let _ = set_pool(ActionMqtt::Get).await;
            });
            let a = auth::current_user().await;
            match a {
                Ok(_) => set_login_status(true),
                Err(_) => set_login_status(false),
            }
        }
    );
    view! {
        <Stylesheet id="leptos" href="/pkg/solar-frontend.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <header>
                <div class="h-32 mx-auto bg-slate-500 w-5/6 p-6 pt-2 mb-20 border-solid border-2 border-sky-500 rounded">
                    <Show
                        when=move || { !login_status() }
                        fallback=move || {
                            view! {
                                <h1 class="font-semibold m-2 mt-0">Logged in</h1>

                                <button
                                    class="mx-4 bg-red-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                                    on:click=logout_fn
                                >

                                    logout
                                </button>
                                <A href="/settings">
                                <button
                                    class="mx-4 bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                                    
                                >

                                Settings
                                </button>
                                </A> 
                            }
                        }
                    >

                        <h1 class="font-semibold m-2 mt-0">Not Logged in</h1>

                    </Show>
                </div>

            </header>
            <main>
                <Routes>
                    <Route
                        path="/"
                        view=move || {
                            view! {
                                <ProtectedContentWrapper
                                    fallback=move || view! { <Redirect path="/login"/> }
                                    condition=login_status
                                >

                                    <Dashboard/>
                                </ProtectedContentWrapper>
                            }
                        }
                    />
                    <Route
                        path="/settings"
                        view=move || {
                            view! {
                                <ProtectedContentWrapper
                                    fallback=move || view! { <Redirect path="/login"/> }
                                    condition=login_status
                                >

                                    <Settings/>
                                </ProtectedContentWrapper>
                            }
                        }
                    />
                    <Route
                        path="/login"
                        view=move || {
                            view! {
                                <ProtectedContentWrapper
                                    fallback=move || view! { <Redirect path="/"/> }
                                    condition=move || !login_status()
                                >

                                    <Login login login_status/>
                                </ProtectedContentWrapper>
                            }
                        }
                    />

                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn Dashboard() -> impl IntoView {
    cfg_if! {
        if #[cfg(feature = "hydrate")] {
            let window = web_sys::window().unwrap();
            let url = window.location().host().unwrap();

            let protocol = {
                if window.location().protocol().unwrap() == "https:" { "wss://" } else { "ws://" }
            };
            leptos_server_signal
                ::provide_websocket(format!("{}{}/ws", protocol, url).as_str())
                .unwrap();


        }
    }    

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


    let lasthour = create_resource(rebootsignal, |_| async move { match get_reboothour().await {
        Ok(a) => {
            a
        }
        Err(e)=> {
            e.to_string()
        }
    } });

    lasthour.refetch();

    view! {
        <div class="flex items-stretch">
            <div class="relayinfo flex-1">

            
                <h3>"Relay State: " <span class=("text-red-700", move || {relaysignal().value == "true"})> {move || relaysignal().value.to_uppercase()}</span></h3>
                <button
                    class="m-4 bg-red-500 hover:bg-red-700 text-white font-bold py-2 px-4 rounded-full"
                    on:click=on
                >
                    "on pool"
                </button>
                <button
                    class="m-4 bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full"
                    on:click=off
                >
                    "off pool"
                </button>
                <div class="m-4">
                <label class="relative inline-flex items-center cursor-pointer">
                    <input
                        type="checkbox"
                        class="sr-only peer"

                        prop:checked=move || {
                            relaysignal().mode
                        } 

                        on:change=move |ev| {
                            let x:bool = event_target_checked(&ev);
                            log::debug!("Value: {}",x);
                            spawn_local(async move {
                                let _ = set_pool(ActionMqtt::setmanualmode(x)).await;
                            });
                        }
                        
                    />
                    <div class="w-11 h-6 bg-gray-200 rounded-full peer peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
                    <span class="ms-3 text-sm font-medium text-gray-900">
                    Manual
                    Mode
                    </span>
                </label>
                </div>

                <p>"Current power: " {move || currentpwsignal().value}</p>
                <p>"Day power: " {move || daypwsignal().value}</p>
            </div>
            <div class="loginfo flex-1">
                <p>"Current hour: " {move || logdatasignal().currenttimehours}</p>
                <p>"time on: " {move || logdatasignal().timeon}</p>

                <p>"last reboot hour: " {move || lasthour.get().unwrap_or("no defined".into())}</p>
            </div>
        </div>

        <button
            class="mt-20 bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded "
            on:click=get
        >
            "Reload values"
        </button>
    }
}

#[component]
fn Login(
    login: Action<auth::Login, Result<(), ServerFnError>>,
    login_status: ReadSignal<bool>
) -> impl IntoView {
    let result_of_call = login.value();

    let error = move || {
        if login_status() {
            "Login success!".to_owned()
        } else {
            result_of_call.with(|msg| {
                msg.clone()
                    .map(|inner| {
                        match inner {
                            Ok(()) => "loged out".to_owned(),

                            Err(x) => { format!("login error: {}", x) }
                        }
                    })
                    .unwrap_or_default()
            })
        }
    };

    view! {
        <Title text="Login"/>
        <div class="">
            <div class="">
                <div class="">
                    <div class="">

                        <p class="">{error}</p>

                        <ActionForm action=login>
                            <fieldset class="">
                                <input
                                    name="username"
                                    class=""
                                    type="text"
                                    placeholder="Your Username"
                                />
                            </fieldset>
                            <fieldset class="">
                                <input
                                    name="password"
                                    class=""
                                    type="password"
                                    placeholder="Password"
                                />
                            </fieldset>
                            <button class="">"Log in"</button>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Settings() -> impl IntoView {
    let settings = create_server_action::<Settings>();

    let _dismiss_enter_with_keyboard = window_event_listener(ev::keydown, move |ev| {
        if ev.key() == "Enter" {
            ev.prevent_default();
        }
    });
    view! {
        <Title text="Settings"/>

        <div class="">
            <ActionForm action=settings>
                <fieldset class="mb-5">
                <label for="number-input" class="block mb-2 text-sm font-medium text-gray-900 ">Type the minimum interval (in Minutes) for the pool to be turned on/off automatically</label>
                    <input
                        name="mininterval"
                        
                        type="number" id="number-input"  class="bg-green-300 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" 
                        placeholder="Please ONLY type integers, leave blank to maintain the previous value"
                    />
                </fieldset>
                <fieldset class="mb-5">
                <label for="number-input2" class="block mb-2 text-sm font-medium text-gray-900 ">Type the minimum time (in Hours) the pool should be turned on in a day</label>

                    <input
                        name="mintimeon"
                        type="number" id="number-input2"  class="bg-green-300 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" 

                        placeholder="Please ONLY type integers, leave blank to maintain the previous value"
                    />
                </fieldset>
                <fieldset class="mb-5">
                <label for="number-input3" class="block mb-2 text-sm font-medium text-gray-900 ">Type the minimum power (in Watss) desired for the pool to be turned on automatically</label>

                    <input
                        name="minpw"
                        type="number" id="number-input3"  class="bg-green-300 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" 

                        placeholder="Please ONLY type integers, leave blank to maintain the previous value"
                    />
                </fieldset>
                <button class="mx-4 bg-red-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">"Save"</button>
            </ActionForm>
                    
        </div>
    }
}
#[component]
pub fn ProtectedContentWrapper<F, IV, W>(
    fallback: F,
    children: ChildrenFn,
    condition: W
)
    -> impl IntoView
    where F: Fn() -> IV + 'static, IV: IntoView, W: Fn() -> bool + 'static
{
    let fallback = store_value(fallback);
    let children = store_value(children);
    let memoized_when = create_memo(move |_| condition());
    // add when conditional here or pass through as signal (above)

    view! {
        <Suspense fallback=|| ()>
            <Show when=memoized_when fallback=move || fallback.with_value(|fallback| fallback())>
                {children.with_value(|children| children())}
            </Show>
        </Suspense>
    }
}
