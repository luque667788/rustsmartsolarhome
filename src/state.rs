use cfg_if::cfg_if;

use crate::models::ParamsJson;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use leptos::LeptosOptions;
        use axum::extract::FromRef;
        use tokio::sync::broadcast;
        use tokio::sync::mpsc;
        use crate::models::LogData;
        use crate::models::ActionMqtt;
        use crate::models::RelayMqtt;
        

        /// This takes advantage of Axum's SubStates feature by deriving FromRef. This is the only way to have more than one
        /// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
        #[derive(FromRef, Clone, Debug)]
        pub struct AppState {
            pub leptos_options: LeptosOptions,
            #[from_ref(skip)]
            pub currentpwget: broadcast::Sender<i64>,
            pub daypwget: broadcast::Sender<i64>,
            pub relayget: broadcast::Sender<RelayMqtt>,
            pub logdataget: broadcast::Sender<LogData>,
            pub relayset: mpsc::Sender<ActionMqtt>,
            pub paramsset: mpsc::Sender<ParamsJson>,
            #[from_ref(skip)]
            pub rebootget: broadcast::Sender<i64>,
            pub sqlpool: sqlx::MySqlPool,
        }
    }
}
