use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use leptos::*;
        use leptos::{ logging::log, provide_context, get_configuration };
        use axum::{
            routing::{ get },
            extract::{ State, Path, RawQuery },
            http::{ Request, header::HeaderMap },
            response::{ IntoResponse, Response },
            Router,
            body::Body as AxumBody,
        };
        use serde_json::{ Value };

        use leptos_axum::{ generate_route_list, LeptosRoutes, handle_server_fns_with_context };
        use rumqttc::{
            AsyncClient,
            Event,
            Incoming,
            MqttOptions,
            QoS,
            TlsConfiguration,
            Transport,
        };
        pub mod app;
        pub mod error_template;
        pub mod fileserv;
        pub mod state;
        pub mod auth;
        pub mod models;

        use crate::fileserv::file_and_error_handler;
        use crate::state::AppState;
        use crate::models::*;
        use crate::app::App;
        
        use tokio::sync::broadcast;
        use tokio::sync::mpsc;
        use tokio::sync::mpsc::Receiver as mReceiver;
        use tokio::sync::mpsc::Sender as mSender;
        use tokio::time::{ Duration };

        #[cfg(feature = "ssr")]
        pub async fn websocket(
            State(app_state): State<AppState>,
            ws: axum::extract::WebSocketUpgrade,
            headers: HeaderMap
        ) -> axum::response::Response {
            if auth::isloged_fn(&headers).await {
                ws.on_upgrade(|ws| handle_socket(app_state, ws))
            } else {
                eprintln!("this should not normally happen ");
                axum::response::Redirect::to("/login").into_response()
            }
        }

        #[cfg(feature = "ssr")]
        async fn handle_socket(app_state: AppState, mut socket: axum::extract::ws::WebSocket) {
            use leptos_server_signal::ServerSignal;
            let mut currentpwget = app_state.currentpwget.subscribe();
            let mut daypwget = app_state.daypwget.subscribe();
            let mut relayget = app_state.relayget.subscribe();
            let mut logdataget = app_state.logdataget.subscribe();
            let mut rebootget = app_state.rebootget.subscribe();
            let mut reboot = ServerSignal::<RebootMqtt>::new("rebootmqtt").unwrap();

            let mut logdata = ServerSignal::<LogData>::new("logdatamqtt").unwrap();
            let mut currentpw = ServerSignal::<CurrentPw>::new("currentpwmqtt").unwrap();
            let mut relay = ServerSignal::<RelayMqtt>::new("relaymqtt").unwrap();
            let mut daypw = ServerSignal::<DayPw>::new("daypwmqtt").unwrap();
            /*
            implement the server signal for relay state changes
             */
            loop {
                tokio::select! {
                    Ok(info) = currentpwget.recv() => {
                        // In any websocket error, break loop.
                        
                        let result = currentpw.with(
                            &mut socket,
                            |count| count.value = info.to_string()
                        ).await;
                        if result.is_err() {
                            break;
                        }
                    },
                    Ok(value) = daypwget.recv() => {
                        
                        let result = daypw.with(
                            &mut socket,
                            |count| count.value = value.to_string()
                        ).await;
                        if result.is_err() {
                            break;
                        }
                    },
                    Ok(state) = relayget.recv() => {
                        
                        let result = relay.with(
                            &mut socket,
                            |count| count.value = state.to_string()
                        ).await;
                        if result.is_err() {
                            break;
                        }
                    },
                    Ok(log) = logdataget.recv() => {
                        let result = logdata.with(
                            &mut socket,
                            |count| *count = log
                        ).await;
                        if result.is_err() {
                            break;
                        }
                    },
                    Ok(log) = rebootget.recv() => {
                        let result = reboot.with(
                            &mut socket,
                            |count| count.value = log
                        ).await;
                        if result.is_err() {
                            break;
                        }
                    },
        
                    else => break,
                }
            }
        }

        async fn server_fn_handler(
            State(app_state): State<AppState>,
            path: Path<String>,
            headers: HeaderMap,
            raw_query: RawQuery,
            request: Request<AxumBody>
        ) -> impl IntoResponse {
            log!("{:?}", path);

            handle_server_fns_with_context(
                path,
                headers,
                raw_query,
                move || {
                    provide_context(app_state.relayset.clone());
                    //provide_context(app_state.txrx.clone());
                },
                request
            ).await
        }

        async fn leptos_routes_handler(
            State(app_state): State<AppState>,
            req: Request<AxumBody>
        ) -> Response {
            let handler = leptos_axum::render_app_to_stream_with_context(
                app_state.leptos_options.clone(),
                move || {
                    // provide_context(app_state.txrx.clone());
                },
                || view! { <App/> }
            );
            handler(req).await.into_response()
        }

        #[tokio::main]
        async fn main() {
            simple_logger::init_with_level(log::Level::Error).expect("couldn't initialize logging");

            let mut mqqt_opts = MqttOptions::new(
                "test-1",
                "w39b31e7.ala.us-east-1.emqxsl.com",
                8883
            );
            mqqt_opts.set_credentials("test", "qwer1234");
            mqqt_opts.set_keep_alive(Duration::from_secs(12 * 60 * 60));

            let ca = include_bytes!("emqx.crt").to_vec();
            let transport = Transport::Tls(TlsConfiguration::Simple {
                ca,
                alpn: None,
                client_auth: None,
            });

            mqqt_opts.set_transport(transport);

            let (client, mut eventloop) = AsyncClient::new(mqqt_opts, 10);

            client
                .subscribe("esp32/sendsundata", QoS::AtMostOnce).await
                .expect("subscribe chan sendsundata ERROR");
            client
                .subscribe("esp32/sendrelaystate", QoS::AtMostOnce).await
                .expect("subscribe chan sendrelaystate ERROR");
            client
                .subscribe("esp32/sendlogdata", QoS::AtMostOnce).await
                .expect("subscribe chan sendlogdata ERROR");
            client
                .subscribe("esp32/reboot", QoS::AtMostOnce).await
                .expect("subscribe chan esp32/reboot ERROR");

            //receiver channel
            let (currentpwget, _rx): (
                broadcast::Sender<i64>,
                broadcast::Receiver<i64>,
            ) = broadcast::channel(100);
            let (logdataget, _rx12): (
                broadcast::Sender<LogData>,
                broadcast::Receiver<LogData>,
            ) = broadcast::channel(100);
            let (reboot, _rx1df2): (
                broadcast::Sender<i64>,
                broadcast::Receiver<i64>,
            ) = broadcast::channel(100);
            let (daypwget, _rx2): (
                broadcast::Sender<i64>,
                broadcast::Receiver<i64>,
            ) = broadcast::channel(100);
            let (relayget, _rx1): (
                broadcast::Sender<bool>,
                broadcast::Receiver<bool>,
            ) = broadcast::channel(100);
            //sender relay channel
            let (relayset, mut relaysetrx): (
                mSender<ActionMqtt>,
                mReceiver<ActionMqtt>,
            ) = mpsc::channel(100);

            let currentpwgetq = currentpwget.clone();
            let daypwgetq = daypwget.clone();
            let relaygetq = relayget.clone();
            let logdatagetq = logdataget.clone();
            let rebootq = reboot.clone();
            // receiver channel (will be implemented with server signals)
            let client2 = client.clone();
            tokio::task::spawn(async move {
                let mut lasttimeon = 0.0;
                let mut lastcurrenttime = 0;
                'mqqttloop: loop {
                    let event = eventloop.poll().await;
                    match &event {
                        Ok(_) => {
                            if let Ok(Event::Incoming(Incoming::Publish(packet))) = event {
                                let message: Value = match
                                    serde_json::from_slice(packet.payload.as_ref())
                                {
                                    Ok(v) => v,
                                    Err(e) => {
                                        eprintln!("ERROR parsing json (mqtt receive){}", e);
                                        serde_json::json!({"parsing ERROR": e.to_string(),})
                                    }
                                };
                                match packet.topic.as_str() {
                                    "esp32/sendsundata" => {
                                        let currentpw: i64 = match message["currentpw"].as_i64() {
                                            Some(v) => v,
                                            None => {
                                                eprintln!("json ERROR attr currentpw not found: {}", message);
                                                continue 'mqqttloop;
                                                // find a better way to deal with json parse error
                                            }
                                        };
                                        if currentpwgetq.receiver_count() > 1 {
                                            if let Err(e) = currentpwgetq.send(currentpw) {
                                                eprintln!("currenpwget chan ERROR (when sending){}", e);
                                            } else {
                                                println!("recevied message mqtt and sent to chan currentpwget: {}", message);
                                            }
                                        } else {
                                            let currentpwgetqq = currentpwgetq.clone();
                                            println!(
                                                "no receivers for currentpwget chan so waiting..."
                                            );
                                            let message1 = message.clone();
                                            tokio::task::spawn_blocking(move || {
                                                loop {
                                                    if currentpwgetqq.receiver_count() > 1 {
                                                        if
                                                            let Err(e) =
                                                                currentpwgetqq.send(currentpw)
                                                        {
                                                            eprintln!("currentpwget chan ERROR (when sending){}", e);
                                                        } else {
                                                            println!("recevied message mqtt and sent to chan currentpwget: {}", message1);
                                                        }
                                                        break;
                                                    }
                                                }
                                            });
                                        }

                                        let daypw: i64 = match message["daypw"].as_i64() {
                                            Some(v) => v,
                                            None => {
                                                eprintln!("json ERROR attr daypw not found: {}", message);
                                                continue 'mqqttloop;
                                                // find a better way to deal with json parse error
                                            }
                                        };
                                        if daypwgetq.receiver_count() > 1 {
                                            if let Err(e) = daypwgetq.send(daypw) {
                                                eprintln!("daypwget chan ERROR (when sending){}", e);
                                            } else {
                                                println!("recevied message mqtt and sent to chan daypwget: {}", message);
                                            }
                                        } else {
                                            let daypwgetqq = daypwgetq.clone();
                                            println!(
                                                "no receivers for daypwget chan so waiting..."
                                            );
                                            let message1 = message.clone();
                                            tokio::task::spawn_blocking(move || {
                                                loop {
                                                    if daypwgetqq.receiver_count() > 1 {
                                                        if let Err(e) = daypwgetqq.send(daypw) {
                                                            eprintln!("daypwget chan ERROR (when sending){}", e);
                                                        } else {
                                                            println!("recevied message mqtt and sent to chan daypwget: {}", message1);
                                                        }
                                                        break;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    "esp32/sendlogdata" => {
                                        /*
                                        
                                        implement last curenttime on and etc 
                                        
                                        
                                         */
                                        let message: LogData = serde_json
                                            ::from_value(message)
                                            .unwrap_or(LogData {
                                                totaltimeon: "error parsing json".into(),
                                                timeon: "error parsing json".into(),
                                                currenttimehours: "error parsing json".into(),
                                            });
                                        lastcurrenttime = message.currenttimehours
                                            .parse::<i64>()
                                            .unwrap_or(0);
                                        lasttimeon = message.timeon.parse::<f64>().unwrap_or(0.0);
                                        if logdatagetq.receiver_count() > 1 {
                                            if let Err(e) = logdatagetq.send(message.clone()) {
                                                eprintln!("logdataget chan ERROR (when sending){}", e);
                                            } else {
                                                println!("recevied message mqtt and sent to chan logdataget: {}", message);
                                            }
                                        } else {
                                            let logdatagetqq = logdatagetq.clone();
                                            println!(
                                                "no receivers for logdataget chan so waiting..."
                                            );
                                            tokio::task::spawn_blocking(move || {
                                                loop {
                                                    if logdatagetqq.receiver_count() > 1 {
                                                        if
                                                            let Err(e) = logdatagetqq.send(
                                                                message.clone()
                                                            )
                                                        {
                                                            eprintln!("logdataget chan ERROR (when sending){}", e);
                                                        } else {
                                                            println!("recevied message mqtt and sent to chan logdataget: {}", message);
                                                        }
                                                        break;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    "esp32/sendrelaystate" => {
                                        let state: bool = match message["state"].as_bool() {
                                            Some(v) => v,
                                            None => {
                                                eprintln!("json ERROR attr state not found: {}", message);
                                                continue 'mqqttloop;
                                            }
                                        };
                                        if relaygetq.receiver_count() > 1 {
                                            if let Err(e) = relaygetq.send(state) {
                                                eprintln!("relayget chan ERROR (when sending){}", e);
                                            } else {
                                                println!("recevied message mqtt and sent to chan relayget: {}", message);
                                            }
                                        } else {
                                            let relaygetqq = relaygetq.clone();
                                            println!(
                                                "no receivers for relayget chan so waiting..."
                                            );
                                            tokio::task::spawn_blocking(move || {
                                                loop {
                                                    if relaygetqq.receiver_count() > 1 {
                                                        if let Err(e) = relaygetqq.send(state) {
                                                            eprintln!("relayget chan ERROR (when sending){}", e);
                                                        } else {
                                                            println!("recevied message mqtt and sent to chan relayget: {}", message);
                                                        }
                                                        break;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    "esp32/reboot" => {
                                        //publish last log data recieved
                                        let a =
                                            serde_json::json!({
                                                //find a way to convert from int to string
                                              "timehour": lastcurrenttime,
                                                   "timeon": lasttimeon,
                                                                });
                                        client2
                                            .publish(
                                                "esp32/setrebootinfo",
                                                QoS::AtLeastOnce,
                                                false,
                                                a.to_string()
                                            ).await
                                            .unwrap_or_else(|e|
                                                eprintln!("publish chan ERROR: {}", e)
                                            );

                                        let time: String = String::from(
                                            message["currenttimehours"]
                                                .as_str()
                                                .unwrap_or("error reboot time json parsing")
                                        );

                                        if rebootq.receiver_count() > 1 {
                                            if
                                                let Err(e) = rebootq.send(
                                                    time.parse::<i64>().unwrap_or(0)
                                                )
                                            {
                                                eprintln!("reboot chan ERROR (when sending){}", e);
                                            } else {
                                                println!("recevied message mqtt and sent to chan reboot: {}", message);
                                            }
                                        } else {
                                            let rebootqq = rebootq.clone();
                                            println!(
                                                "no receivers for relayget chan so waiting..."
                                            );
                                            tokio::task::spawn_blocking(move || {
                                                loop {
                                                    if rebootqq.receiver_count() > 1 {
                                                        if
                                                            let Err(e) = rebootqq.send(
                                                                time.parse::<i64>().unwrap_or(0)
                                                            )
                                                        {
                                                            eprintln!("reboot chan ERROR (when sending){}", e);
                                                        } else {
                                                            println!("recevied message mqtt and sent to chan reboot: {}", message);
                                                        }
                                                        break;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    e => {
                                        eprintln!("unknow mqtt channel ERROR: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("Got event mqtt Error = {e:?}");

                            return Ok::<(), rumqttc::ConnectionError>(());
                        }
                    }
                }
            });

            //sender channels

            tokio::spawn(async move {
                while let Some(msg) = relaysetrx.recv().await {
                    // In any websocket error, break loop.
                    match msg {
                        ActionMqtt::State(on) => {
                            let on = || {
                                if on { "on" } else { "off" }
                            };

                            let a =
                                serde_json::json!({
                                "state": on(),
                            });
                            client
                                .publish(
                                    "esp32/relay",
                                    QoS::AtLeastOnce,
                                    false,
                                    a.to_string()
                                ).await
                                .unwrap_or_else(|e| eprintln!("publish chan ERROR: {}", e));
                        }
                        ActionMqtt::Get => {
                            println!("received request to get data");
                            client
                                .publish(
                                    "esp32/get",
                                    QoS::AtLeastOnce,
                                    false,
                                    "uninportant message"
                                ).await
                                .unwrap_or_else(|e| eprintln!("publish chan ERROR: {}", e));
                        }
                    }
                }
            });

            // *** spawn thread that listens to eventloop and publish to channel tx
            // and takes care of also sending messages

            // Setting get_configuration(None) means we'll be using cargo-leptos's env values
            // For deployment these variables are:
            // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
            // Alternately a file can be specified such as Some("Cargo.toml")
            // The file would need to be included with the executable when moved to deployment
            let conf = get_configuration(None).await.unwrap();
            let leptos_options = conf.leptos_options;
            let addr = leptos_options.site_addr;
            let routes = generate_route_list(App);

            // implement reboot chan and reboot info vector etc...
            let app_state = AppState {
                leptos_options,
                relayset,
                relayget,
                currentpwget,
                daypwget,
                logdataget,
                rebootget: reboot,
            };

            // build our application with a route
            let app = Router::new()
                .route("/api/*fn_name", get(server_fn_handler).post(server_fn_handler))
                .route("/ws", get(websocket))
                .leptos_routes_with_handler(routes, get(leptos_routes_handler))
                .fallback(file_and_error_handler)
                .with_state(app_state);

            // run our app with hyper
            // `axum::Server` is a re-export of `hyper::Server`
            log::info!("listening on http://{}", &addr);
            axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
        }
    } else {
        pub fn main() {}
    }
    // This example cannot be built as a trunk standalone CSR-only app.
    // Only the server may directly connect to the database.
}
