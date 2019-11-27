#[macro_use]
extern crate seed;
pub mod helpers;

use anymap::any::Any;
pub use helpers::event_helpers::{on_click, on_input};
pub use helpers::fetch_hooks;
pub use helpers::form_state;
pub use helpers::graphql_list;
use seed::prelude::*;
use std::cell::RefCell;

thread_local! {
    static INSTANCE: RefCell<anymap::Map<dyn Any>> = RefCell::new(anymap::Map::<dyn Any>::new());
}

///  Init function to store a clone of the seed app
pub fn init<Ms: 'static, Mdl: 'static, O: Orders<Ms>>(orders: &mut O) {
    INSTANCE.with(|map_refcell| {
        let mut map = map_refcell.borrow_mut();
        map.insert(orders.clone_app());
    });
}

/// Retrieves a clone of seed::app . This enables app::update to be retrieved f
///     from anywhere for instance a timeout callback.
pub fn get_app<Ms: 'static, Mdl: 'static>() -> Option<seed::App<Ms, Mdl, Node<Ms>>> {
    INSTANCE.with(|map_refcell| {
        map_refcell
            .borrow()
            .get::<seed::App<Ms, Mdl, Node<Ms>>>()
            .cloned()
    })
}

/// Shedule an update with an arbitary message
/// Useful if you want to send a message to the app from a view component.
/// Ensure you do this in a do_once black or equivalent otherwise you will
/// get an infinite loop
pub fn schedule_update<Ms: Clone + 'static, Mdl: 'static>(msg: Ms) {
    let boxed_fn = {
        Box::new(move || {
            if let Some(app) = get_app::<Ms, Mdl>() {
                app.update(msg.clone());
            }
        })
    };
    seed::set_timeout(boxed_fn, 0);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
