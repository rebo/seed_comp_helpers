use serde::Serialize;
// A Partially complete graphQL backed list
// Can execute a graphQL query and populate a list
// Does a No Op on fetch fail

use crate::fetch_hooks;

use fetch_hooks::use_fetch_collection_hook::{
    ArrayResponse, UseFetchCollection, UseFetchCollectionStatusTrait,
};

pub use fetch_hooks::{IntoList, UseFetchStatus};

// use crate::use_fetch_hooks::IntoList;
use comp_state::list::{use_list, List, ListControl};
use comp_state::{topo, StateAccess};
use seed::*;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, Default, Serialize)]
pub struct SendMessageRequestBody {
    pub query: String,
}

#[derive(Clone)]
pub struct GraphQLListControl<C, I>
where
    C: Clone + 'static + DeserializeOwned + IntoList<I>,
    I: Clone + 'static + DeserializeOwned,
{
    pub list: ListControl<I>,
    pub fetcher: StateAccess<UseFetchCollection<ArrayResponse<C, I>, SendMessageRequestBody>>,
}

impl<C, I> GraphQLListControl<C, I>
where
    C: Clone + DeserializeOwned + 'static + IntoList<I>,
    I: Clone + DeserializeOwned + 'static,
    StateAccess<UseFetchCollection<I, SendMessageRequestBody>>: UseFetchCollectionStatusTrait,
{
    pub fn get_list(&self) -> List<I> {
        self.list.get_list()
    }

    pub fn dispatch<Ms, Mdl>(&self)
    where
        Ms: Clone + Default + 'static,
        Mdl: 'static,
    {
        self.fetcher.dispatch::<Ms, Mdl>();
    }

    pub fn status(&self) -> UseFetchStatus {
        self.fetcher.hard_get().status
    }
}

pub fn use_graphql_list<
    C: Clone + DeserializeOwned + 'static + IntoList<I>,
    I: Clone + DeserializeOwned,
>(
    query: &str,
    url: &str,
) -> (List<I>, GraphQLListControl<C, I>) {
    topo::call!({
        //create a blank list to be used later
        let (_list, list_control) = use_list(|| vec![]);

        // intialize fetch objects and control

        let json_request = SendMessageRequestBody {
            query: query.to_string(),
        };

        let (fetched, fetch_control) = fetch_hooks::use_fetch_collection::<
            C,
            I,
            SendMessageRequestBody,
        >(url, Method::Post, json_request);
        // if fetched is returned as Some then
        // load list_control
        if let Some(fetched) = fetched {
            comp_state::do_once({
                || {
                    for item in fetched.items() {
                        list_control.push(item.clone());
                    }
                }
            })
        }

        let graphql_list_control = GraphQLListControl::<C, I> {
            list: list_control,
            fetcher: fetch_control,
        };
        // return list, list_c otnrol, and fetch_cotnrol
        (graphql_list_control.get_list(), graphql_list_control)
    })
}

// no method named `items` found for type `helpers::use_fetch_hooks::use_fetch_collection_hook::ArrayResponse<C, I>` in the current scope

// method not found in `helpers::use_fetch_hooks::use_fetch_collection_hook::ArrayResponse<C, I>`

// help: items from traits can only be used if the trait is implemented and in scope
// note: the following trait defines an item `items`, perhaps you need to implement it:
//       candidate #1: `helpers::use_fetch_hooks::use_fetch_collection_hook::IntoList`
