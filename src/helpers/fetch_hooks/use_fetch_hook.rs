use super::NextTick;
use super::UseFetchMsgTrait;
use super::UseFetchStatus;
use crate::get_app;
use comp_state::update_state_with_topo_id;
use comp_state::StateAccess;
use comp_state::{topo, use_state};
use enclose::enclose;
use futures::Future;
use futures_util::FutureExt;
use seed::*;
use serde::de::DeserializeOwned;
use wasm_bindgen_futures::spawn_local;

/// return a maybe_fetched and fetcher. Maybe_fetched is None until the fetch request returns deserialised json
/// into type T. fetcher.dispatch() needs to be called to actually start the fetching. It is done this way so you can
/// use fetcher in a callback.
///
#[topo::nested]
pub fn use_fetch<T: Clone + DeserializeOwned>(
    url: String,
    method: Method,
) -> (Option<T>, impl UseFetchStatusTrait) {
    let (state, fetcher) = use_state(|| UseFetch::new(url, method));

    let maybe_fetched: Option<T> = match (state.status, state.string_response) {
        (UseFetchStatus::Complete, Some(response)) => {
            let result: Result<T, _> = serde_json::from_str(&response);
            let poss = result.unwrap();
            Some(poss)
        }
        _ => None,
    };
    (maybe_fetched, fetcher)
}

use std::default::Default;

#[derive(Clone)]
pub struct UseFetch {
    pub status: UseFetchStatus,
    pub string_response: Option<String>,
    pub fail_reason: Option<seed::fetch::FailReason<String>>,
    pub url: String,
    pub method: Method,
}

impl UseFetch {
    fn new(url: String, method: Method) -> UseFetch {
        UseFetch {
            status: UseFetchStatus::Initialized,
            string_response: None,
            fail_reason: None,
            url,
            method,
        }
    }
}

pub trait UseFetchStatusTrait: Clone {
    fn status(&self) -> UseFetchStatus;
    fn dispatch<Ms: Default + 'static, Mdl: 'static>(&self);
    fn dispatch_with_seed<Ms: UseFetchMsgTrait + 'static, Mdl: 'static>(&self);
}

impl UseFetchStatusTrait for StateAccess<UseFetch> {
    fn status(&self) -> UseFetchStatus {
        self.get().unwrap().status
    }

    fn dispatch_with_seed<Ms: UseFetchMsgTrait + 'static, Mdl: 'static>(&self) {
        let use_fetch = self.get().unwrap();
        self.update(|state| state.status = UseFetchStatus::Loading);
        let url = use_fetch.url.clone();
        let method = use_fetch.method;
        let id = self.id;
        let boxed_fn = {
            Box::new(move || {
                if let Some(app) = get_app::<Ms, Mdl>() {
                    app.update(Ms::fetch_message(id, url.clone(), method));
                }
            })
        };

        seed::set_timeout(boxed_fn, 0);
    }

    fn dispatch<Ms: 'static + Default, Mdl: 'static>(&self) {
        let use_fetch = self.get().unwrap();
        self.update(|state| state.status = UseFetchStatus::Loading);
        let url = use_fetch.url.clone();
        let method = use_fetch.method;
        let id = self.id;
        let boxed_fn = {
            Box::new(move || {
                if let Some(app) = get_app::<Ms, Mdl>() {
                    let lazy_schedule_cmd = enclose!((app => _s, url) move |_| {
                        let url = url.clone();
                        spawn_local( fetch_string::<Ms>(id, url, method).map( |_| () ))
                    });
                    // we need to clear the call stack by NextTick so we don't exceed it's capacity
                    spawn_local(NextTick::new().map(lazy_schedule_cmd));

                    app.update(Ms::default());
                }
            })
        };

        seed::set_timeout(boxed_fn, 0);
    }
}

pub fn fetch_string<Ms: Default + 'static>(
    id: topo::Id,
    url: String,
    method: Method,
) -> impl Future<Output = Result<Ms, Ms>> {
    seed::fetch::Request::new(url)
        .method(method)
        .fetch_string(move |f| {
            match f.response() {
                Ok(response) => update_state_with_topo_id::<UseFetch, _>(id, |u| {
                    u.status = UseFetchStatus::Complete;
                    u.string_response = Some(response.data.clone());
                }),
                Err(fail_reason) => update_state_with_topo_id::<UseFetch, _>(id, |u| {
                    u.status = UseFetchStatus::Failed;
                    u.fail_reason = Some(fail_reason);
                }),
            }
            Ms::default()
        })
}
