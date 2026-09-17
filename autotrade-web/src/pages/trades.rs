use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::dto::OrderResponse;

#[component]
pub fn Trades() -> impl IntoView {
    let orders = RwSignal::new(Vec::<OrderResponse>::new());
    let running = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let executed = RwSignal::new(Option::<usize>::None);

    let on_trigger = move |_| {
        running.set(true);
        error.set(None);
        executed.set(None);
        orders.set(vec![]);

        spawn_local(async move {
            match api::trigger_autotrade().await {
                Ok(resp) => {
                    executed.set(resp.executed);
                    orders.set(resp.orders.unwrap_or_default());
                }
                Err(e) => error.set(Some(e)),
            }
            running.set(false);
        });
    };

    view! {
        <div>
            <div class="page-header">
                <h1>"Auto Trade"</h1>
                <p class="subtitle">
                    "Execute trades based on your active rules"
                </p>
            </div>

            <div class="stats-grid" style="margin-bottom: 1.5rem">
                <div class="stat-card">
                    <div class="stat-card-title">"How It Works"</div>
                    <div class="stat-item">
                        <span class="label">"Step 1"</span>
                        <span class="value"
                            style="font-size: 0.85rem; font-weight: 500">
                            "Set rules in the Rules tab"
                        </span>
                    </div>
                    <div class="stat-item">
                        <span class="label">"Step 2"</span>
                        <span class="value"
                            style="font-size: 0.85rem; font-weight: 500">
                            "Click Run AutoTrade"
                        </span>
                    </div>
                </div>
                <div class="stat-card">
                    <div class="stat-card-title">"Engine"</div>
                    <div class="stat-item">
                        <span class="label">"Data Source"</span>
                        <span class="value"
                            style="font-size: 0.85rem">"IDX Real-Time"
                        </span>
                    </div>
                    <div class="stat-item">
                        <span class="label">"AI Model"</span>
                        <span class="value"
                            style="font-size: 0.85rem">"DeepSeek"
                        </span>
                    </div>
                </div>
                <div class="stat-card">
                    <div class="stat-card-title">"Status"</div>
                    <div class="stat-item">
                        <span class="label">"State"</span>
                        <span class="value" style="font-size: 0.85rem">
                            {move || {
                                if running.get() {
                                    "Running..."
                                } else if executed.get().is_some() {
                                    "Completed"
                                } else {
                                    "Ready"
                                }
                            }}
                        </span>
                    </div>
                    <div class="stat-item">
                        <span class="label">"Last Result"</span>
                        <span class="value" style="font-size: 0.85rem">
                            {move || {
                                match executed.get() {
                                    Some(n) => format!("{} order(s)", n),
                                    None => "\u{2014}".to_string(),
                                }
                            }}
                        </span>
                    </div>
                </div>
            </div>

            <section class="section"
                style="display: flex; align-items: center; gap: 1rem">
                <button
                    class="btn btn-primary"
                    on:click=on_trigger
                    disabled=move || running.get()
                    style="min-width: 160px"
                >
                    {move || {
                        if running.get() {
                            "Executing..."
                        } else {
                            "Run AutoTrade"
                        }
                    }}
                </button>
                {move || {
                    if running.get() {
                        Some(view! {
                            <div style="display: flex; align-items: center; gap: 0.5rem">
                                <div class="spinner"></div>
                                <span style="color: var(--text-secondary); font-size: 0.875rem">
                                    "Checking rules against live market data..."
                                </span>
                            </div>
                        })
                    } else {
                        None
                    }
                }}
                {move || {
                    executed.get().map(|n| {
                        view! {
                            <span class="success-msg"
                                style="display: flex; align-items: center; gap: 0.35rem">
                                {format!("Executed {} order(s)", n)}
                            </span>
                        }
                    })
                }}
            </section>

            {move || {
                error.get().map(|e| {
                    view! {
                        <div class="error-msg">{e}</div>
                    }
                })
            }}

            {move || {
                let items = orders.get();
                if items.is_empty() {
                    None
                } else {
                    Some(view! {
                        <section class="section">
                            <span class="section-title">
                                "Executed Orders"
                            </span>
                            <table class="table">
                                <thead>
                                    <tr>
                                        <th>"Symbol"</th>
                                        <th>"Side"</th>
                                        <th>"Lot"</th>
                                        <th>"Price"</th>
                                        <th>"Status"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {items
                                        .into_iter()
                                        .map(|o| {
                                            let side_class = if o.side
                                                == "Buy"
                                            {
                                                "text-green font-bold"
                                            } else {
                                                "text-red font-bold"
                                            };
                                            view! {
                                                <tr>
                                                    <td class="font-bold">
                                                        {o.symbol}
                                                    </td>
                                                    <td class={side_class}>
                                                        {o.side}
                                                    </td>
                                                    <td>{o.lot}</td>
                                                    <td>{o.price}</td>
                                                    <td>{o.status}</td>
                                                </tr>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </section>
                    })
                }
            }}
        </div>
    }
}
