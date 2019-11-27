# Some helpers and custom hooks for seed

* on_click and on_input - shortcus for mouse_ev(Ev::Click, |f| Msg) and  input_ev(Ev::Input, |f| Msg) 
* use_fetch and use_fetch_colleciton - custom hooks to manage json requets
* graphql_list - example of a custom hook using the above to make a graphql query and keep a list up to date
* form_state - proof of concept form helpers, very incomplete
* schedule_update - schedule an app update from anywhere
* init and get_app - access the app from anywhere