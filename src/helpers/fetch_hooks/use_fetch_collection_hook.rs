use crate::get_app;
use comp_state::{topo, use_state, StateAccess};
use enclose::enclose;
use futures::Future;
use seed::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use wasm_bindgen_futures::spawn_local;

use super::{fetch_json, NextTick, UseFetchStatus};

type FetcherContainerItemJson<C, I, J> = StateAccess<UseFetchCollection<ArrayResponse<C, I>, J>>;

/// use_fetch just returns a single struct. This returns a collection
/// collection_name is used to ensure the json deserializes into ArrayResponse(data:DataResponseEnum::UseFetchCollectionItems)
pub fn use_fetch_collection<
    C: Clone + DeserializeOwned + 'static + IntoList<I>,
    I: Clone + DeserializeOwned + 'static,
    J: Serialize + Clone + 'static,
>(
    url: &str,
    method: Method,
    json: J,
) -> (
    Option<ArrayResponse<C, I>>,
    FetcherContainerItemJson<C, I, J>,
) {
    topo::call!({
        let (state, state_access) =
            use_state(|| UseFetchCollection::<ArrayResponse<C, I>, J>::new(url, json, method));

        let possible_type: Option<ArrayResponse<C, I>> = match (state.status, state.string_response)
        {
            (UseFetchStatus::Complete, Some(response)) => {
                let result: Result<ArrayResponse<C, I>, _> = serde_json::from_str(&response);
                let poss = result.unwrap();
                Some(poss)
            }
            _ => None,
        };
        (possible_type, state_access)
    })
}

#[derive(Clone, Debug)]
pub struct UseFetchCollection<T, J>
where
    J: Serialize,
{
    pub status: UseFetchStatus,
    pub json_body: J,
    pub string_response: Option<String>,
    pub url: String,
    pub method: Method,
    pub _phantom_data: PhantomData<T>,
}

impl<T, J> UseFetchCollection<T, J>
where
    J: Serialize,
{
    fn new(url: &str, json_body: J, method: Method) -> UseFetchCollection<T, J> {
        UseFetchCollection {
            status: UseFetchStatus::Initialized,
            string_response: None,
            url: url.to_string(),
            json_body,
            method,
            _phantom_data: PhantomData::default(),
        }
    }
}

pub trait UseFetchCollectionStatusTrait: Clone {
    fn status(&self) -> UseFetchStatus;
    fn dispatch<Ms: Default + Clone + 'static, Mdl: 'static>(&self);
}

impl<T, J> UseFetchCollectionStatusTrait for StateAccess<UseFetchCollection<T, J>>
where
    T: Clone + DeserializeOwned + 'static,
    J: Clone + Serialize + 'static,
{
    fn status(&self) -> UseFetchStatus {
        self.get().unwrap().status
    }

    fn dispatch<Ms: 'static + Clone + Default, Mdl: 'static>(&self) {
        let use_fetch = self.get().unwrap();
        self.update(|state| state.status = UseFetchStatus::Loading);
        let url = use_fetch.url.clone();
        let id = self.id;
        let boxed_fn = {
            Box::new(move || {
                if let Some(app) = get_app::<Ms, Mdl>() {
                    let lazy_schedule_cmd = enclose!((url, use_fetch) move |_| {
                        let url = url.clone();
                        spawn_local(
                                {fetch_json::<T,_,Ms,Mdl>(id, url, use_fetch.json_body, use_fetch.method,).then(move |_| {
                                // let msg_returned_from_effect = res.unwrap_or_else(|err_msg| err_msg);
                                // recursive call which can blow the call stack
                                // s.update(Ms::default());
                                Ok(()) })}
                            )

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

#[derive(Clone, Debug, Deserialize)]
pub struct ArrayResponse<C, I>
where
    C: IntoList<I>,
{
    pub data: C,
    #[serde(skip)]
    _phantom_data: PhantomData<I>,
}

impl<C, I> ArrayResponse<C, I>
where
    C: IntoList<I>,
{
    pub fn items(&self) -> Vec<I> {
        self.data.items()
    }
}

pub trait IntoList<I> {
    fn items(&self) -> Vec<I>;
}
