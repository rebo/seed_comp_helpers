pub mod use_fetch_collection_hook;
pub mod use_fetch_hook;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub use use_fetch_collection_hook::{
    use_fetch_collection, IntoList, UseFetchCollection, UseFetchCollectionStatusTrait,
};
pub use use_fetch_hook::{use_fetch, UseFetch, UseFetchStatusTrait};

use comp_state::{topo, update_state_with_topo_id};

use seed::*;
use serde::de::{DeserializeOwned, Deserializer};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

// Code + docs: https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen_futures/

#[derive(Clone, Debug)]
pub enum UseFetchStatus {
    Initialized,
    Loading,
    Failed,
    Complete,
}

impl Default for UseFetchStatus {
    fn default() -> Self {
        UseFetchStatus::Initialized
    }
}

// Needs to be implemented if you want Seed to be fully informed about fetching
pub trait UseFetchMsgTrait {
    fn fetch_message(id: topo::Id, url: String, method: Method) -> Self;
    fn fetched_message(id: topo::Id, response: String) -> Self;
}

pub(crate) fn fetch_json<
    T: DeserializeOwned + 'static + Clone,
    J: Serialize + Clone + 'static,
    Ms: Default + Clone + 'static,
    Mdl: 'static,
>(
    id: topo::Id,
    url: String,
    json: J,
    method: Method,
) -> impl Future<Output = Result<Ms, Ms>> {
    seed::fetch::Request::new(url)
        .method(method)
        .send_json(&json)
        .fetch_string(move |f| {
            let data = f.response().unwrap().data;
            update_state_with_topo_id::<UseFetchCollection<T, J>, _>(id, |u| {
                u.status = UseFetchStatus::Complete;
                u.string_response = Some(data.clone());
                crate::schedule_update::<Ms, Mdl>(Ms::default());
            });
            Ms::default()
        })
}

// Some of the NextTick code is copied from seed source, this is becuase it is needed
// to ensure the fetch future is properly executed.

/// A future that becomes ready after a tick of the micro task queue.
// A future that becomes ready after a tick of the micro task queue.
pub struct NextTick {
    inner: JsFuture,
}

impl NextTick {
    /// Construct a new `NextTick` future.
    pub fn new() -> NextTick {
        // Create a resolved promise that will run its callbacks on the next
        // tick of the micro task queue.
        let promise = js_sys::Promise::resolve(&JsValue::NULL);
        // Convert the promise into a `JsFuture`.
        let inner = JsFuture::from(promise);
        NextTick { inner }
    }
}

impl Default for NextTick {
    fn default() -> Self {
        Self::new()
    }
}
impl Future for NextTick {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<()> {
        // Polling a `NextTick` just forwards to polling if the inner promise is
        // ready.
        match Pin::new(&mut self.get_mut().inner).poll(ctx) {
            Poll::Ready(_) => Poll::Ready(()),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}

// pub(crate) fn fetch_string_with_seed_msg<Ms: UseFetchMsgTrait + 'static>(
//     id: topo::Id,
//     url: String,
//     method: Method,
// ) -> impl Future<Item = Ms, Error = Ms> {
//     seed::fetch::Request::new(url)
//         .method(method)
//         .fetch_string(move |f| Ms::fetched_message(id, f.response().unwrap().data))
// }

// pub(crate) fn update_fetch<Ms: UseFetchMsgTrait + 'static>(
//     orders: &mut impl Orders<Ms>,
//     id: topo::Id,
//     url: String,
//     method: Method,
// ) {
//     orders.perform_cmd(fetch_string_with_seed_msg::<Ms>(id, url, method));
// }

// pub(crate) fn update_fetched(id: topo::Id, string_response: String) {
//     update_state_with_topo_id::<UseFetch, _>(id, |u| {
//         u.status = UseFetchStatus::Complete;
//         u.string_response = Some(string_response.clone());
//     })
// }
