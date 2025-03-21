use bankid_mock::{app::App, ConfigState, DeviceCompletionData, Orders, PendingCode, UserCompletionData};
#[cfg(feature = "ssr")]
pub mod fileserv;
#[cfg(feature = "ssr")]
use crate::fileserv::file_and_error_handler;
#[cfg(feature = "ssr")]
use axum::response::Response as AxumResponse;
#[cfg(feature = "ssr")]
use axum::{
    extract::{Path, Request, State},
    response::IntoResponse,
};
#[cfg(feature = "ssr")]
use axum::{
    routing::{get, post},
    Router,
};
#[cfg(feature = "ssr")]
use bankid_mock::app::*;
#[cfg(feature = "ssr")]
use config::get_configuration;
#[cfg(feature = "ssr")]
use http::HeaderMap;
#[cfg(feature = "ssr")]
use leptos::*;
#[cfg(feature = "ssr")]
use leptos::{
    config::LeptosOptions,
    prelude::{provide_context, *},
};
#[cfg(feature = "ssr")]
use leptos_axum::{generate_route_list_with_exclusions_and_ssg_and_context, LeptosRoutes};
#[cfg(feature = "ssr")]
use leptos_axum::{handle_server_fns_with_context, AxumRouteListing};
#[cfg(feature = "ssr")]
use leptos_ws::server_signals::ServerSignals;
use leptos_ws::ServerSignal;
use bankid_mock::OrderData;
use bankid_mock::Config;
use leptos_meta::MetaTags;
 use serde::Serialize;
use axum::Json;
use serde::Deserialize;
use leptos_axum::generate_route_list;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::{net::SocketAddr, sync::Arc, time::Duration};
    use tokio::time::sleep;

    pub fn shell(options: LeptosOptions) -> impl IntoView {
        view! {
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                    <link rel="stylesheet" href="pkg/axum_example.css"/>

                    <MetaTags/>
                    <AutoReload options=options.clone()/>
                    <HydrationScripts options=options />
                </head>
                <body>
                    <App/>
                </body>
            </html>
        }
    }

    async fn leptos_routes_handler(state: State<AppState>, req: Request) -> AxumResponse {
        let state1 = state.0.clone();
        let options2 = state.clone().0.options.clone();
        let handler = leptos_axum::render_route_with_context(
            state.routes.clone().unwrap(),
            move || {
                provide_context(state1.options.clone());
                provide_context(state1.config.clone());
                provide_context(state1.orders.clone());
                provide_context(state1.server_signals.clone());
            },
            move || shell(options2.clone()),
        );
        handler(state, req).await.into_response()
    }
    async fn server_fn_handler(
        State(state): State<AppState>,
        _path: Path<String>,
        _headers: HeaderMap,
        _query: axum::extract::RawQuery,
        request: Request,
    ) -> impl IntoResponse {
        handle_server_fns_with_context(
            move || {
                provide_context(state.options.clone());
                provide_context(state.config.clone());
                provide_context(state.orders.clone());
                provide_context(state.server_signals.clone());
            },
            request,
        )
        .await
    }

    let server_signals = ServerSignals::new();
    //let signal = ServerSignal::new("counter".to_string(), 1);
    // build our application with a route
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;

    let toml_str = std::fs::read_to_string("config.toml").unwrap_or(String::default());

    let orders = Orders::new(OrderData::new());
    let decoded: Config = toml::from_str(&toml_str).unwrap();
    let mut state = AppState {
        options: leptos_options.clone(),
        routes: None,
        orders: orders.clone(),
        config: ConfigState::new(decoded),
        server_signals: server_signals.clone(),
    };
    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let addr = leptos_options.site_addr;
    let state2 = state.clone();
    let routes = generate_route_list(
        App,
    );    state.routes = Some(routes.clone());
    let app = Router::new()
        .route("/api/*fn_name", get(server_fn_handler).post(server_fn_handler))
        .route("/rp/v6.0/auth", axum::routing::post(auth))
        .route("/rp/v6.0/collect", axum::routing::post(collect))
        .route(
            "/ws",
            get(leptos_ws::axum::websocket(state.server_signals.clone())),
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .with_state(state);
    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    leptos::logging::log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
// Old
//#[cfg(feature = "ssr")]
//#[tokio::main]
//async fn main() {
//    use std::time::Duration;
//
//    use axum::routing::get;
//    use axum::Router;
//    use bankid_mock::fileserv::file_and_error_handler;
//    use bankid_mock::{app::*, Config, OrderData};
//    use leptos::*;
//    use leptos_axum::{generate_route_list, LeptosRoutes};
//
//    let conf = get_configuration(None).await.unwrap();
//    let leptos_options = conf.leptos_options;
//    let addr = leptos_options.site_addr;
//    let routes = generate_route_list(App);
//    let toml_str = std::fs::read_to_string("config.toml").unwrap_or(String::default());
//
//    let decoded: Config = toml::from_str(&toml_str).unwrap();
//    let (tx, _rx) = tokio::sync::broadcast::channel(100);
//    let orders = Orders::new(OrderData::new());
//    let app_state = AppState {
//        leptos_options,
//        routes: routes.clone(),
//        orders: orders.clone(),
//        config: ConfigState::new(decoded),
//        tx: tx.clone(),
//    };
//
//    // build our application with a route
//    let app = Router::new()
//        .route(
//            "/api/*fn_name",
//            get(server_fn_handler).post(server_fn_handler),
//        )
//        .route("/rp/v6.0/auth", axum::routing::post(auth))
//        .route("/rp/v6.0/collect", axum::routing::post(collect))
//        .route("/ws", get(websocket))
//        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
//        .fallback(file_and_error_handler)
//        .with_state(app_state);
//
//    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
//    logging::log!("listening on http://{}", &addr);
//    tokio::spawn(async move {
//        let orders = orders.clone();
//        let tx = tx.clone();
//        loop {
//            tokio::time::sleep(Duration::from_secs(10)).await;
//            let mut guard = orders.lock().unwrap();
//            let count = guard.remove_old();
//            drop(guard);
//            if count != 0 {
//                tx.send(()).unwrap();
//            }
//        }
//    });
//    axum::serve(
//        listener,
//        app.into_make_service_with_connect_info::<SocketAddr>(),
//    )
//    .await
//    .unwrap();
//}
#[axum::debug_handler]
#[cfg(feature = "ssr")]
async fn auth(
    axum::extract::State(state): axum::extract::State<AppState>,
    insecure_ip: axum_client_ip::InsecureClientIp,
) -> Json<AuthResponse> {
    let uid = uuid::Uuid::new_v4();
    let ip = insecure_ip.0;
{
    let mut guard = state.orders.lock().unwrap();

    guard.insert_empty(uid, ip);
    drop(guard);
    }
    let mut server_signals = state.server_signals.clone();
    let signal  = server_signals.get_signal::<ServerSignal<i32>>("counter".to_string()).await.unwrap();
    signal.update(|x| {
        *x += 1;
    });
    Json(AuthResponse {
        order_ref: uid.into(),
        auto_start_token: "7c40b5c9-fa74-49cf-b98c-bfe651f9a7c6".into(),
        qr_start_token: "67df3917-fa0d-44e5-b327-edcc928297f8".into(),
        qr_start_secret: "d28db9a7-4cde-429e-a983-359be676944c".into(),
    })
}

#[cfg(feature = "ssr")]
async fn collect(
    axum::extract::State(state): axum::extract::State<AppState>,
    options: Json<CollectOptions>,
) -> Json<CollectResponse> {
    use bankid_mock::OrderEnum;

    let guard = state.orders.lock().unwrap();
    match guard.get(&options.order_ref) {
        Some(OrderEnum::Completed(o)) => Json(CollectResponse {
            order_ref: options.order_ref.clone().into(),
            status: StatusEnum::Complete,
            hint_code: None,
            completion_data: Some(CompletionData {
                user: o.clone(),
                device: DeviceCompletionData {
                    ip_adress: "192.168.1.1".to_string(),
                },
                bank_id_issue_date: "2023-01-01".to_string(),
                signature: "".to_string(),
                ocsp_response: "".to_string(),
            }),
        }),
        Some(OrderEnum::Expired) => Json(CollectResponse {
            order_ref: options.order_ref.clone().into(),
            status: StatusEnum::Failed,
            hint_code: Some(HintCodes::Failed(FailedHintCodes::ExpiredTransaction)),
            completion_data: None,
        }),
        Some(OrderEnum::Pending(o)) => Json(CollectResponse {
            order_ref: options.order_ref.clone().into(),
            status: StatusEnum::Pending,
            hint_code: Some(HintCodes::Pending(o.status.clone())),
            completion_data: None,
        }),
        None => Json(CollectResponse {
            order_ref: options.order_ref.clone().into(),
            status: StatusEnum::Pending,
            hint_code: Some(HintCodes::Pending(PendingCode::Started)),
            completion_data: None,
        }),
    }
}
use axum::extract::FromRef;
#[cfg(feature = "ssr")]
#[derive(FromRef, Clone)]
pub struct AppState {
    pub options: LeptosOptions,
    pub server_signals: ServerSignals,
    pub routes: Option<Vec<AxumRouteListing>>,
    pub orders: Orders,
    pub config: ConfigState,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    order_ref: String,
    auto_start_token: String,
    qr_start_token: String,
    qr_start_secret: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum HintCodes {
    Pending(PendingCode),
    Failed(FailedHintCodes) 
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectResponse {
    order_ref: String,
    status: StatusEnum,
    hint_code: Option<HintCodes>,
    completion_data: Option<CompletionData>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionData {
    user: UserCompletionData,
    device: DeviceCompletionData,
    bank_id_issue_date: String,
    signature: String,
    ocsp_response: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FailedHintCodes {
ExpiredTransaction

}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StatusEnum {
    Pending,
    Complete,
    Failed,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CollectOptions {
    order_ref: uuid::Uuid,
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}

