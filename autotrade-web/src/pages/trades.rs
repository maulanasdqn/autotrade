use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::dto::OrderResponse;

#[component]
pub fn Trades() -> impl IntoView {
    let orders = RwSignal::new(Vec::<OrderResponse>::new());
    let running = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let executed = RwSignal::new(Option::<usize>::None);
    let auto_on = RwSignal::new(false);
    let timer_id = RwSignal::new(Option::<i32>::None);

    let run_trade = move || {
        if running.get_untracked() {
            return;
        }
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

    let on_trigger = move |_| run_trade();

    let on_toggle_auto = move |_| {
        if auto_on.get_untracked() {
            auto_on.set(false);
            if let Some(id) = timer_id.get_untracked() {
                if let Some(w) = web_sys::window() {
                    w.clear_interval_with_handle(id);
                }
                timer_id.set(None);
            }
        } else {
            auto_on.set(true);
            let cb = Closure::<dyn Fn()>::new(move || run_trade());
            if let Some(w) = web_sys::window() {
                if let Ok(id) =
                    w.set_interval_with_callback_and_timeout_and_arguments_0(
                        cb.as_ref().unchecked_ref(),
                        300_000,
                    )
                {
                    timer_id.set(Some(id));
                }
            }
            cb.forget();
            run_trade();
        }
    };

    on_cleanup(move || {
        if let Some(id) = timer_id.get_untracked() {
            if let Some(w) = web_sys::window() {
                w.clear_interval_with_handle(id);
            }
        }
    });

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
                            "Click Run or enable Auto-Run"
                        </span>
                    </div>
                </div>
                <div class="stat-card">
                    <div class="stat-card-title">"Engine"</div>
                    <div class="stat-item">
                        <span class="label">"Data Source"</span>
                        <span class="value"
                            style="font-size: 0.85rem">"Yahoo Finance"
                        </span>
                    </div>
                    <div class="stat-item">
                        <span class="label">"Auto-Run"</span>
                        <span class="value" style="font-size: 0.85rem">
                            {move || {
                                if auto_on.get() {
                                    "Every 5 min"
                                } else {
                                    "Off"
                                }
                            }}
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
                style="display: flex; align-items: center; gap: 1rem; flex-wrap: wrap">
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
                <button
                    on:click=on_toggle_auto
                    style=move || {
                        if auto_on.get() {
                            "min-width: 160px; padding: 0.6rem 1.2rem; border-radius: 8px; font-weight: 600; cursor: pointer; border: none; background: #dc2626; color: white;"
                        } else {
                            "min-width: 160px; padding: 0.6rem 1.2rem; border-radius: 8px; font-weight: 600; cursor: pointer; border: 2px solid #2563eb; background: white; color: #2563eb;"
                        }
                    }
                >
                    {move || {
                        if auto_on.get() {
                            "Stop Auto-Run"
                        } else {
                            "Auto-Run (5m)"
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
