use axum::{
    body::Body,
    extract::{Path, Request, State},
    response::{IntoResponse, Response},
    Json,
};
use bankid_mock::{app::App, ConfigState, DeviceCompletionData, Orders, UserCompletionData};
use core::net::SocketAddr;
use leptos::provide_context;
use leptos_axum::handle_server_fns_with_context;
use leptos_router::RouteListing;
use serde::{Deserialize, Serialize};

async fn leptos_routes_handler(State(app_state): State<AppState>, req: Request<Body>) -> Response {
    let handler = leptos_axum::render_route_with_context(
        app_state.leptos_options.clone(),
        app_state.routes.clone(),
        move || {
            provide_context(app_state.orders.clone());
            provide_context(app_state.config.clone());
            provide_context(app_state.tx.clone());
        },
        App,
    );
    handler(req).await.into_response()
}

async fn server_fn_handler(
    State(app_state): State<AppState>,
    _path: Path<String>,
    request: Request<Body>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(app_state.orders.clone());
            provide_context(app_state.config.clone());
            provide_context(app_state.tx.clone());
        },
        request,
    )
    .await
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::time::Duration;

    use axum::routing::get;
    use axum::Router;
    use bankid_mock::fileserv::file_and_error_handler;
    use bankid_mock::{app::*, Config, OrderData};
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    let toml_str = std::fs::read_to_string("config.toml").unwrap_or(String::default());

    let decoded: Config = toml::from_str(&toml_str).unwrap();
    let (tx, _rx) = tokio::sync::broadcast::channel(100);
    let orders = Orders::new(OrderData::new());
    let app_state = AppState {
        leptos_options,
        routes: routes.clone(),
        orders: orders.clone(),
        config: ConfigState::new(decoded),
        tx: tx.clone(),
    };

    // build our application with a route
    let app = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .route("/rp/v6.0/auth", axum::routing::post(auth))
        .route("/rp/v6.0/collect", axum::routing::post(collect))
        .route("/ws", get(websocket))
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    tokio::spawn(async move {
        let orders = orders.clone();
        let tx = tx.clone();
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            let mut guard = orders.lock().unwrap();
            let count = guard.remove_old();
            drop(guard);
            if count != 0 {
                tx.send(()).unwrap();
            }
        }
    });
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

#[cfg(feature = "ssr")]
async fn auth(
    axum::extract::State(state): axum::extract::State<AppState>,
    insecure_ip: axum_client_ip::InsecureClientIp,
) -> Json<AuthResponse> {
    let uid = uuid::Uuid::new_v4();
    let mut guard = state.orders.lock().unwrap();

    let ip = insecure_ip.0;
    guard.insert_empty(uid, ip);
    drop(guard);
    state.tx.send(()).unwrap();
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
            hint_code: Some(HintCode::Started),
            completion_data: None,
        }),
        Some(OrderEnum::Pending) => Json(CollectResponse {
            order_ref: options.order_ref.clone().into(),
            status: StatusEnum::Pending,
            hint_code: Some(HintCode::Started),
            completion_data: None,
        }),
        None => Json(CollectResponse {
            order_ref: options.order_ref.clone().into(),
            status: StatusEnum::Pending,
            hint_code: Some(HintCode::Started),
            completion_data: None,
        }),
    }
}
use axum::extract::FromRef;
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: leptos::LeptosOptions,
    pub routes: Vec<RouteListing>,
    pub orders: Orders,
    pub config: ConfigState,
    tx: tokio::sync::broadcast::Sender<()>,
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
pub struct CollectResponse {
    order_ref: String,
    status: StatusEnum,
    hint_code: Option<HintCode>,
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
pub enum HintCode {
    Started,
    ExpiredTransaction,
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
#[cfg(feature = "ssr")]
pub async fn websocket(
    ws: axum::extract::WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::response::Response {
    let rec = state.tx.subscribe();
    ws.on_upgrade(|socket| handle_socket(socket, rec))
}

#[cfg(feature = "ssr")]
async fn handle_socket(
    mut socket: axum::extract::ws::WebSocket,
    mut rec: tokio::sync::broadcast::Receiver<()>,
) {
    use bankid_mock::app::Count;
    use leptos_server_signal::ServerSignal;

    let mut count = ServerSignal::<Count>::new("counter").unwrap();

    loop {
        rec.recv().await.unwrap();
        let result = count.with(&mut socket, |count| count.value += 1).await;
        if result.is_err() {
            break;
        }
    }
}
