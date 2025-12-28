use std::net::IpAddr;

use crate::error_template::{AppError, ErrorTemplate};
use crate::{PendingCode, QuickUser};
use codee::string::JsonSerdeCodec;
use itertools::Itertools;
use js_sys::Date;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::hooks::use_params_map;
use leptos_router::path;
use leptos_use::use_cookie;
use leptos_ws::ReadOnlySignal;
use rand::distr::slice::Choose;
use rand::distr::{Distribution, Uniform};
use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};
use uuid::Uuid;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    leptos_ws::provide_websocket();

    view! {
        <Stylesheet id="leptos" href="/pkg/bankid-mock.css" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />
        <Html {..} lang="en" attr:data-bs-theme="dark" />
        <Link
            href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css"
            rel="stylesheet"
            integrity="sha384-QWTKZyjpPEjISv5WaRU9OFeRpok6YctnYmDr5pNlyT2bRjXh0JMhjY6hW+ALEwIH"
            crossorigin="anonymous"
        />

        // sets the document title
        <Title text="Bank-id mock" />

        // content for this welcome page
        <Router>
            <main>
                <Navbar />
                <Routes fallback=|| {
                    let mut outside_errors = Errors::default();
                    outside_errors.insert_with_default_key(AppError::NotFound);
                    view! { <ErrorTemplate outside_errors /> }.into_any()
                }>
                    <Route 
                    path=path!("") 
        view=ListAllPlacesWithActiveOrders />
                    <Route 
                    path=path!("by-ip/:ip") 
                    view=GetByIP />
                    <Route path=path!("by-alias/:alias") view=GetByAlias />
                </Routes>
            </main>
        </Router>
        <script
            src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js"
            integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz"
            crossorigin="anonymous"
        ></script>
    }
}

#[component]
fn Navbar() -> impl IntoView {
    let aliases = Resource::new(|| (), |_| async move { get_aliases().await });

    view! {
        <Transition>
            <nav class="navbar navbar-expand-lg ">
                <div class="container-fluid">
                    <A href="/" {..} class="navbar-brand">
                        Bank-id mock
                    </A>
                    <button
                        class="navbar-toggler"
                        type="button"
                        data-bs-toggle="collapse"
                        data-bs-target="#navbarScroll"
                        aria-controls="navbarScroll"
                        aria-expanded="false"
                        aria-label="Toggle navigation"
                    >
                        <span class="navbar-toggler-icon"></span>
                    </button>
                    <div class="collapse navbar-collapse" id="navbarScroll">
                        <ul class="navbar-nav me-auto my-2 my-lg-0">
                            {move || {
                                aliases
                                    .get()
                                    .map(|o| {
                                        o.unwrap()
                                            .into_iter()
                                            .map(|n| {
                                                view! {
                                                    <li class="nav-item">
                                                        <A href=format!("/by-alias/{}", n) {..} class="nav-link">
                                                            {n}
                                                        </A>
                                                    </li>
                                                }
                                            })
                                            .collect_view()
                                    })
                            }}

                        </ul>
                    </div>
                </div>
            </nav>
        </Transition>
    }
}

#[component]
fn ListAllPlacesWithActiveOrders() -> impl IntoView {
    let count = ReadOnlySignal::new("counter", 1).unwrap();

    let count = move || count.get();
    let ips = Resource::new(count, |_| get_ips());

    view! {
        <Suspense>
            <div>
                {move || {
                    ips.get()
                        .map(|o| {
                            view! {
                                <table class="table">
                                    <thead>
                                        <tr>
                                            <th>Location</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {o
                                            .unwrap()
                                            .into_iter()
                                            .map(|n| match n {
                                                IpEntry::JustIp(n) => {
                                                    view! {
                                                        <tr>
                                                            <td>
                                                                <A href=format!(
                                                                    "/by-ip/{}",
                                                                    n.to_string(),
                                                                )>{n.to_string()}</A>
                                                            </td>
                                                        </tr>
                                                    }
                                                }
                                                IpEntry::Alias(alias) => {
                                                    view! {
                                                        <tr>
                                                            <td>
                                                                <A href=format!("/by-alias/{}", alias)>{alias}</A>
                                                            </td>
                                                        </tr>
                                                    }
                                                }
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            }
                        })
                }}

            </div>
        </Suspense>
    }
}
/// Renders the home page of your application.
#[component]
fn GetByAlias() -> impl IntoView {
    leptos_ws::provide_websocket();
    let count = ReadOnlySignal::new("counter", 1).unwrap();
    let params = use_params_map();
    let alias = move || params.with(|params| params.get("alias").unwrap_or_default());
    let count = move || count.get();
    let orders = Resource::new(
        move || (alias(), count()),
        |(alias, _count)| get_orders_by_alias(alias),
    );
    let first_and_lastnames = Resource::new(move || (), |_| get_first_and_lastname_options());

    view! {
        <Suspense>
            <div>
                {move || {
                    let firstnames = Signal::derive(move || {
                        first_and_lastnames
                            .get()
                            .map(|p| p)
                            .map(|k| k.unwrap().0)
                            .unwrap_or_default()
                    });
                    let lastnames = Signal::derive(move || {
                        first_and_lastnames
                            .get()
                            .map(|p| p)
                            .map(|k| k.unwrap().1)
                            .unwrap_or_default()
                    });
                    orders
                        .get()
                        .map(|order| {
                            match order {
                                Ok(o) => {
                                    let quick_users = Signal::derive(move || o.0.clone());
                                    view! {
                                        <table class="table">

                                            <thead>
                                                <tr>
                                                    <th>Id</th>
                                                    <th>status</th>
                                                    <th>time</th>
                                                    <th>Actions</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {o
                                                    .1
                                                    .iter()
                                                    .sorted_by(|a, b| Ord::cmp(&a.0, &b.0).reverse())
                                                    .map(|n| {
                                                        view! {
                                                            <RenderOrder
                                                                id=n.1
                                                                time=n.0
                                                                quick_users=quick_users
                                                                first_names=firstnames
                                                                last_names=lastnames
                                                            />
                                                        }
                                                    })
                                                    .collect_view()}
                                            </tbody>
                                        </table>
                                    }
                                        .into_any()
                                }
                                Err(o) => view! { <p>{o.to_string()}</p> }.into_any(),
                            }
                        })
                }}

            </div>
        </Suspense>
    }
}

/// Renders the home page of your application.
#[component]
fn GetByIP() -> impl IntoView {
    let count = ReadOnlySignal::new("counter", 0).unwrap();
    let params = use_params_map();
    let ip = move || {
        params.with(|params| {
            params
                .get("ip")
                .unwrap_or_default()
                .parse::<IpAddr>()
                .unwrap()
        })
    };
    let count = move || count.get();
    let orders_resource = Resource::new(move || (ip(), count()), |(ip, _count)| get_orders(ip));
    let first_and_lastnames = Resource::new(
        || (),
        |_| async move { get_first_and_lastname_options().await },
    );

    view! {
        <Suspense>
            <div>
                {move || {
                    let firstnames = Signal::derive(move || {
                        first_and_lastnames
                            .get()
                            .map(|p| p)
                            .map(|k| k.unwrap().0)
                            .unwrap_or_default()
                    });
                    let lastnames = Signal::derive(move || {
                        first_and_lastnames
                            .get()
                            .map(|p| p)
                            .map(|k| k.unwrap().1)
                            .unwrap_or_default()
                    });
                    orders_resource
                        .get()
                        .map(|orders| {
                            match orders {
                                Ok(o) => {
                                    let quick_users = Signal::derive(move || o.0.clone());
                                    view! {
                                        <table class="table">

                                            <thead>
                                                <tr>
                                                    <th>Id</th>
                                                    <th>status</th>
                                                    <th>time</th>
                                                    <th>Actions</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {o
                                                    .1
                                                    .iter()
                                                    .sorted_by(|a, b| Ord::cmp(&a.0, &b.0).reverse())
                                                    .map(|n| {
                                                        view! {
                                                            <RenderOrder
                                                                id=n.1
                                                                time=n.0
                                                                quick_users=quick_users
                                                                first_names=firstnames
                                                                last_names=lastnames
                                                            />
                                                        }
                                                    })
                                                    .collect_view()}
                                            </tbody>
                                        </table>
                                    }
                                        .into_any()
                                }
                                Err(o) => view! { <p>{o.to_string()}</p> }.into_any(),
                            }
                        })
                }}

            </div>
        </Suspense>
    }
}

#[component]
fn RenderOrder(
    time: OffsetDateTime,
    id: Uuid,
    quick_users: Signal<Vec<QuickUser>>,
    first_names: Signal<Vec<String>>,
    last_names: Signal<Vec<String>>,
) -> impl IntoView {
    let complete_order = ServerAction::<CompleteOrder>::new();
    let (id, _) = signal(id);
    let (ssn, set_ssn) = signal("".to_string());
    let (name, set_name) = signal("".to_string());
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let (offset, set_offset) = use_cookie::<UtcOffset, JsonSerdeCodec>("offset");
    if offset.get().is_none() {
        (move || set_offset.set(Some(UtcOffset::from_hms(0, 0, 0).unwrap())))();
    }
    Effect::new(move |_| {
        let date = Date::new_0();
        let offset = date.get_timezone_offset();
        set_offset.set(Some(
            UtcOffset::from_whole_seconds(-(offset.round() as i32 * 60)).unwrap(),
        ));
    });
    view! {
        <tr>
            <td>{move || id.get().to_string()}</td>
            <td>
                <form>
                    <select class="form-select" aria-label="Default select example">
                        {move || {
                            PendingCode::iter()
                                .map(|i| {
                                    let i2 = i.clone();
                                    let id = id.get().clone();
                                    view! {
                                        <option on:click=move |_p| {
                                            let id = id.clone();
                                            let i2 = i2.clone();
                                            spawn_local(async move {
                                                update_pending_status(id, i2.clone()).await.unwrap();
                                            });
                                        }>{format!("{:?}", &i)}</option>
                                    }
                                })
                                .collect_view()
                        }}

                    </select>
                </form>
            </td>
            <td>{move || time.to_offset(offset.get().unwrap()).format(&format).unwrap()}</td>
            <td>
                <ActionForm action=complete_order>
                    // {..} class="row row-cols-lg-auto g-3 align-items-center"
                    <input type="text" name="id" value=move || id.get().to_string() hidden />
                    <div class="col-12">
                        <label class="visually-hidden" for=move || format!("ssn-{}", id.get())>
                            Ssn
                        </label>
                        <div class="input-group">
                            <button
                                class="btn btn-outline-secondary"
                                type="button"
                                on:click=move |_| {
                                    set_ssn(generate_random_ssn());
                                }
                            >

                                "Randomize"
                            </button>
                            <input
                                type="text"
                                name="ssn"
                                class="form-control"
                                id=move || format!("ssn-{}", id.get())
                                placeholder="Ssn"
                                on:input=move |ev| {
                                    set_ssn(event_target_value(&ev));
                                }

                                prop:value=ssn
                            />
                        </div>
                    </div>
                    <div class="col-12">
                        <label class="visually-hidden" for=move || format!("name-{}", id.get())>
                            Name
                        </label>
                        <div class="input-group">

                            {move || {
                                let has_any = first_names.get().iter().count() > 0
                                    && last_names.get().iter().count() > 0;
                                if has_any {
                                    view! {
                                        <button
                                            class="btn btn-outline-secondary"
                                            type="button"
                                            on:click=move |_| {
                                                let first_names = first_names.get();
                                                let last_names = last_names.get();
                                                let first_names: Vec<_> = first_names
                                                    .iter()
                                                    .map(|s| s.as_str())
                                                    .collect();
                                                let last_names: Vec<_> = last_names
                                                    .iter()
                                                    .map(|s| s.as_str())
                                                    .collect();
                                                let name = generate_random_name(
                                                    first_names.as_slice(),
                                                    last_names.as_slice(),
                                                );
                                                set_name(name);
                                            }
                                        >

                                            "Randomize"
                                        </button>
                                    }
                                        .into_any()
                                } else {
                                    view! {}.into_any()
                                }
                            }}
                            <input
                                type="text"
                                name="name"
                                class="form-control"
                                id=move || format!("name-{}", id.get())
                                placeholder="Name"
                                on:input=move |ev| {
                                    set_name(event_target_value(&ev));
                                }

                                prop:value=name
                            />
                        </div>
                    </div>

                    <div class="col-12">
                        <input
                            type="submit"
                            class="btn btn-primary"
                            value="Submit"
                            disabled=move || {
                                complete_order.pending().get()
                                    || complete_order.value().get().is_some()
                            }
                        />

                    </div>

                </ActionForm>

                {quick_users
                    .get()
                    .into_iter()
                    .map(|p| {
                        view! {
                            <ActionForm action=complete_order>
                                // {..} class="d-inline-block"

                                <input
                                    type="text"
                                    name="id"
                                    value=move || id.get().to_string()
                                    hidden
                                />
                                <input name="ssn" value=p.ssn.to_string() hidden />
                                <input name="name" value=p.name.to_string() hidden />
                                <input
                                    type="submit"
                                    class="btn btn-link"
                                    value=p.label.clone()
                                    disabled=move || {
                                        complete_order.pending().get()
                                            || complete_order.value().get().is_some()
                                    }
                                />

                            </ActionForm>
                        }
                    })
                    .collect_view()}

            </td>
        </tr>
    }
}

#[server(GetOrders, "/api")]
pub async fn get_orders(
    ip: IpAddr,
) -> Result<(Vec<QuickUser>, Vec<(OffsetDateTime, Uuid)>), ServerFnError> {
    let orders = use_context::<crate::Orders>().ok_or_else(|| {
        ServerFnError::<server_fn::error::NoCustomError>::ServerError("Orders missing.".into())
    })?;
    let config = use_context::<crate::ConfigState>().ok_or_else(|| {
        ServerFnError::<server_fn::error::NoCustomError>::ServerError("config missing.".into())
    })?;

    let quick_users = &config.0.quick_users.clone().unwrap_or_default();
    let ord = orders.0.lock().unwrap();
    Ok((quick_users.to_vec(), ord.get_all(&ip)))
}

#[server(GetOrdersByAlias, "/api")]
pub async fn get_orders_by_alias(
    alias: String,
) -> Result<(Vec<QuickUser>, Vec<(OffsetDateTime, Uuid)>), ServerFnError> {
    let orders = use_context::<crate::Orders>().ok_or_else(|| {
        ServerFnError::<server_fn::error::NoCustomError>::ServerError("Orders missing.".into())
    })?;
    let config = use_context::<crate::ConfigState>().ok_or_else(|| {
        ServerFnError::<server_fn::error::NoCustomError>::ServerError("config missing.".into())
    })?;

    let quick_users = &config.quick_users.clone().unwrap_or_default();
    let ord = orders.lock().unwrap();

    let orders: Vec<_> = config
        .aliases
        .as_ref()
        .map(|q| {
            q.iter()
                .find(|p| p.name == alias)
                .iter()
                .flat_map(|p| ord.get_all(&p.ip))
                .collect()
        })
        .unwrap();

    Ok((quick_users.to_vec(), orders))
}

#[server]
pub async fn get_first_and_lastname_options() -> Result<(Vec<String>, Vec<String>), ServerFnError> {
    let config = use_context::<crate::ConfigState>().ok_or_else(|| {
        ServerFnError::<server_fn::error::NoCustomError>::ServerError("config missing.".into())
    })?;

    let first_names = config.first_names.clone().unwrap_or_default().to_vec();
    let last_names = config.last_names.clone().unwrap_or_default().to_vec();

    Ok((first_names, last_names))
}
#[server]
pub async fn get_aliases() -> Result<Vec<String>, ServerFnError> {
    let config = use_context::<crate::ConfigState>().ok_or_else(|| {
        ServerFnError::<server_fn::error::NoCustomError>::ServerError("config missing.".into())
    })?;

    let aliases = config.aliases.as_ref().cloned().unwrap_or_default();

    let alias_names = aliases.iter().map(|f| f.name.clone()).collect();
    Ok(alias_names)
}
#[server]
pub async fn get_ips() -> Result<Vec<IpEntry>, ServerFnError> {
    let orders = use_context::<crate::Orders>().ok_or_else(|| {
        ServerFnError::<server_fn::error::NoCustomError>::ServerError("Orders missing.".into())
    })?;
    let config = use_context::<crate::ConfigState>().ok_or_else(|| {
        ServerFnError::<server_fn::error::NoCustomError>::ServerError("config missing.".into())
    })?;

    let ord = orders.lock().unwrap();
    let ips = ord.get_ips();
    let aliases = config.aliases.as_ref().cloned().unwrap_or_default();

    let ips = ips
        .iter()
        .map(|f| {
            let alias = aliases.iter().find(|p| &p.ip == f);
            if let Some(alias) = alias {
                IpEntry::Alias(alias.name.clone())
            } else {
                IpEntry::JustIp(f.clone())
            }
        })
        .collect();
    drop(ord);
    Ok(ips)
}

#[derive(Deserialize, Serialize, Clone)]
pub enum IpEntry {
    JustIp(IpAddr),
    Alias(String),
}

#[server]
pub async fn complete_order(id: Uuid, ssn: String, name: String) -> Result<(), ServerFnError> {
    let orders = use_context::<crate::Orders>().ok_or_else(|| {
        ServerFnError::<server_fn::error::NoCustomError>::ServerError("Orders missing.".into())
    })?;

    let count = ReadOnlySignal::new("counter", 9).unwrap();

    let mut ord = orders.lock().unwrap();
    let new_name = name.clone();
    let mut split = new_name.split_whitespace();
    let given = split.nth(0).unwrap_or("");
    let surname = split.last().unwrap_or("");
    ord.upgrade(
        id,
        crate::UserCompletionData {
            personal_number: ssn,
            name,
            given_name: given.to_string(),
            sur_name: surname.to_string(),
        },
    );
    count.update(|x| *x += 1);

    Ok(())
}
#[server]
pub async fn update_pending_status(id: Uuid, status: PendingCode) -> Result<(), ServerFnError> {
    let orders = use_context::<crate::Orders>().ok_or_else(|| {
        ServerFnError::<server_fn::error::NoCustomError>::ServerError("Orders missing.".into())
    })?;
    let mut ord = orders.lock().unwrap();
    ord.set_pending_status(id, status);

    Ok(())
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Count {
    pub value: i32,
}

fn generate_random_ssn() -> String {
    let mut rng = rand::rng();
    let between = Uniform::new(1850, 2000).unwrap();
    let month = Uniform::new(1, 12).unwrap();
    let day = Uniform::new(1, 25).unwrap();
    let number = Uniform::new(0, 999).unwrap();
    let year = between.sample(&mut rng);
    let month = month.sample(&mut rng);
    let day = day.sample(&mut rng);
    let number = number.sample(&mut rng);

    let whole = format!("{}{:0>2}{:0>2}{:0>3}", year, month, day, number);

    let check_didgit = luhn(&whole[2..]);

    format!("{}{}", whole, check_didgit)
}
fn generate_random_name(first_names: &[&str], last_names: &[&str]) -> String {
    let mut rng = rand::rng();
    let first_names = Choose::new(first_names).unwrap();
    let first_name = first_names.sample(&mut rng);

    let last_names = Choose::new(last_names).unwrap();
    let last_name = last_names.sample(&mut rng);
    format!("{} {}", first_name, last_name)
}

/// https://en.wikipedia.org/wiki/Luhn_algorithm.
fn luhn(value: &str) -> u8 {
    let checksum = value
        .chars()
        .map(|c| c.to_digit(10).unwrap_or(0))
        .enumerate()
        .fold(0, |acc, (idx, v)| {
            let value = if idx % 2 == 0 { v * 2 } else { v };
            acc + if value > 9 { value - 9 } else { value }
        });

    (10 - (checksum as u8 % 10)) % 10
}
