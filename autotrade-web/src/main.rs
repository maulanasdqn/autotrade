use leptos::prelude::*;
use wasm_bindgen::prelude::*;

mod api;
mod components;
mod dto;
mod pages;

use pages::analysis::Analysis;
use pages::dashboard::Dashboard;
use pages::rules::Rules;
use pages::trades::Trades;

fn main() {
    leptos::mount::mount_to_body(App);
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Dashboard,
    Analysis,
    Trades,
    Rules,
}

fn path_to_tab(path: &str) -> Tab {
    match path.trim_start_matches('/') {
        "analysis" => Tab::Analysis,
        "trades" => Tab::Trades,
        "rules" => Tab::Rules,
        _ => Tab::Dashboard,
    }
}

fn tab_to_path(t: Tab) -> &'static str {
    match t {
        Tab::Dashboard => "/",
        Tab::Analysis => "/analysis",
        Tab::Trades => "/trades",
        Tab::Rules => "/rules",
    }
}

fn current_path() -> String {
    web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default()
}

fn push_path(path: &str) {
    if let Some(w) = web_sys::window() {
        if let Ok(h) = w.history() {
            let _ = h.push_state_with_url(&JsValue::NULL, "", Some(path));
        }
    }
}

#[component]
fn App() -> impl IntoView {
    let tab = RwSignal::new(path_to_tab(&current_path()));

    let cb = Closure::<dyn Fn()>::new(move || {
        tab.set(path_to_tab(&current_path()));
    });
    if let Some(w) = web_sys::window() {
        let _ = w.add_event_listener_with_callback(
            "popstate",
            cb.as_ref().unchecked_ref(),
        );
    }
    cb.forget();

    let nav_item = move |t: Tab, label: &'static str| {
        let active = move || {
            if tab.get() == t { "nav-link active" } else { "nav-link" }
        };
        view! {
            <li>
                <a class=active href=tab_to_path(t) on:click=move |ev| {
                    ev.prevent_default();
                    push_path(tab_to_path(t));
                    tab.set(t);
                }>{label}</a>
            </li>
        }
    };

    view! {
        <main class="app">
            <nav class="topnav">
                <div class="topnav-brand">
                    <div class="topnav-brand-icon">"AT"</div>
                    "AutoTrade"
                </div>
                <ul>
                    {nav_item(Tab::Dashboard, "Dashboard")}
                    {nav_item(Tab::Analysis, "Analysis")}
                    {nav_item(Tab::Trades, "Trades")}
                    {nav_item(Tab::Rules, "Rules")}
                </ul>
            </nav>
            <section class="content">
                {move || match tab.get() {
                    Tab::Dashboard => view! { <Dashboard /> }.into_any(),
                    Tab::Analysis => view! { <Analysis /> }.into_any(),
                    Tab::Trades => view! { <Trades /> }.into_any(),
                    Tab::Rules => view! { <Rules /> }.into_any(),
                }}
            </section>
        </main>
    }
}
