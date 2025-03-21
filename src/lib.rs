use core::panic;
use std::borrow::Borrow;
use std::ops::Deref;
use std::{collections::HashMap, net::IpAddr, sync::Mutex};

use serde::Deserialize;
use serde::Serialize;

use strum::EnumIter;
use time::Duration;
use time::OffsetDateTime;
use uuid::Uuid;

pub mod app;
use leptos::mount::mount_to_body;
pub mod error_template;


#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
leptos::mount::hydrate_body(App);
}

#[derive(Debug)]
pub struct Orders(std::sync::Arc<Mutex<OrderData>>);

impl Deref for Orders {
    type Target = Mutex<OrderData>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Orders {
    pub fn new(order_data: OrderData) -> Self {
        Self(std::sync::Arc::new(Mutex::new(order_data)))
    }
}

#[derive(Debug)]
pub struct ConfigState(std::sync::Arc<Config>);

impl ConfigState {
    pub fn new(config: Config) -> Self {
        ConfigState(std::sync::Arc::new(config))
    }
}

impl Deref for ConfigState {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(PartialEq, Debug)]
pub struct Order {
    ip: IpAddr,
    order_time: OffsetDateTime,
    data: OrderEnum,
}
#[derive(PartialEq, Debug)]
pub enum OrderEnum {
    Pending(PendingData),
    Completed(UserCompletionData),
    Expired,
}

#[derive(Debug)]
pub struct OrderData {
    data: HashMap<uuid::Uuid, Order>,
}
use itertools::Itertools;
impl OrderData {
    pub fn new() -> Self {
        OrderData {
            data: HashMap::new(),
        }
    }

    pub fn insert_empty(&mut self, id: uuid::Uuid, ip: IpAddr) {
        self.data.insert(
            id,
            Order {
                ip,
                order_time: OffsetDateTime::now_utc(),
                data: OrderEnum::Pending(PendingData {
                    status: PendingCode::Started,
                }),
            },
        );
    }

    pub fn upgrade(&mut self, id: uuid::Uuid, data: UserCompletionData) {
        let slot = self.data.get_mut(&id).unwrap();
        slot.data = OrderEnum::Completed(data);
    }

    pub fn set_pending_status(&mut self, id: uuid::Uuid, status: PendingCode) {
        let slot = self.data.get_mut(&id).unwrap();
        match &mut slot.data {
            OrderEnum::Pending(ref mut o) => o.status = status,
            _ => panic!("Crash"),
        }
    }

    pub fn get(&self, id: &uuid::Uuid) -> Option<&OrderEnum> {
        self.data.get(&id).map(|p| &p.data)
    }
    pub fn get_ips(&self) -> Vec<IpAddr> {
        self.data
            .iter()
            .filter(|o| matches!(o.1.data, OrderEnum::Pending(_)))
            .map(|o| &o.1.ip)
            .cloned()
            .unique()
            .collect()
    }

    pub fn remove_old(&mut self) -> u32 {
        let now = OffsetDateTime::now_utc();
        let condition = |a: (&Uuid, &Order)| {
            matches!(a.1.data, OrderEnum::Pending(_))
                && a.1.order_time.saturating_add(Duration::seconds(50)) < now
        };
        let old_count = self.data.iter().filter(|(a, b)| condition((a, b))).count();

        self.data
            .iter_mut()
            .filter(|(a, b)| condition((a, b.borrow())))
            .for_each(|f| f.1.data = OrderEnum::Expired);

        let new_count = self.data.iter().filter(|(a, b)| condition((a, b))).count();

        return (old_count - new_count).try_into().unwrap();
    }
    pub fn get_all(&self, ip: &IpAddr) -> Vec<(OffsetDateTime, Uuid)> {
        self.data
            .iter()
            .filter(|o| matches!(o.1.data, OrderEnum::Pending(_)))
            .filter(|o| &o.1.ip == ip)
            .map(|o| (o.1.order_time.clone(), o.0.clone()))
            .collect()
    }
}

impl Clone for Orders {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl Clone for ConfigState {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingData {
    pub status: PendingCode,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, EnumIter)]
#[serde(rename_all = "camelCase")]
pub enum PendingCode {
    Started,
    UserCallConfirm,
    UserSign,
    UserMrtd,
    NoClient,
    OutstandingTransaction,
}
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserCompletionData {
    pub personal_number: String,
    pub name: String,
    pub given_name: String,
    pub sur_name: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCompletionData {
    pub ip_adress: String,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub aliases: Option<Vec<Alias>>,
    pub quick_users: Option<Vec<QuickUser>>,
    pub first_names: Option<Vec<String>>,
    pub last_names: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Alias {
    pub ip: IpAddr,
    pub name: String,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct QuickUser {
    pub label: String,
    pub ssn: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct MyDateTime(pub OffsetDateTime);

impl Default for MyDateTime {
    fn default() -> Self {
        Self(OffsetDateTime::now_utc())
    }
}
