use leptos::prelude::*;

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

#[component]
fn App() -> impl IntoView {
    let tab = RwSignal::new(Tab::Dashboard);

    let nav_item = move |t: Tab, label: &'static str| {
        let active = move || {
            if tab.get() == t { "nav-link active" } else { "nav-link" }
        };
        view! {
            <li>
                <a class=active on:click=move |_| tab.set(t)>
                    {label}
                </a>
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
