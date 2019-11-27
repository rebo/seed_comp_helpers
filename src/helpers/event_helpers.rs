use seed::{prelude::*, *};

/// Shortcut for mouse_ev(Ev::Click, ...)
/// with the exception that the defailt Message gets returned
pub fn on_click<Ms, F>(func: F) -> events::Listener<Ms>
where
    Ms: Default + Clone,
    F: FnOnce(web_sys::MouseEvent) -> () + 'static + Clone,
{
    mouse_ev(Ev::Click, |a| {
        func(a);
        Ms::default()
    })
}

/// Shortcut for input_ev(Ev::Input, ...)
/// with the exception that the defailt Message gets returned
pub fn on_input<Ms, F>(func: F) -> events::Listener<Ms>
where
    Ms: Default + Clone,
    F: FnOnce(String) -> () + 'static + Clone,
{
    input_ev(Ev::Input, |a| {
        func(a);
        Ms::default()
    })
}
