use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::dto::RuleResponse;

#[component]
pub fn Rules() -> impl IntoView {
    let rules = RwSignal::new(Vec::<RuleResponse>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(Option::<String>::None);

    let sym_input = RwSignal::new(String::new());
    let buy_input = RwSignal::new(String::new());
    let sell_input = RwSignal::new(String::new());
    let sl_input = RwSignal::new(String::new());
    let lot_input = RwSignal::new(String::new());
    let saving = RwSignal::new(false);

    let load_rules = move || {
        spawn_local(async move {
            loading.set(true);
            match api::fetch_rules().await {
                Ok(resp) => rules.set(resp.data),
                Err(e) => error.set(Some(e)),
            }
            loading.set(false);
        });
    };

    load_rules();

    let on_create = move |_| {
        let sym = sym_input.get_untracked();
        let buy: f64 = buy_input.get_untracked().parse().unwrap_or(0.0);
        let sell: f64 = sell_input.get_untracked().parse().unwrap_or(0.0);
        let sl: f64 = sl_input.get_untracked().parse().unwrap_or(0.0);
        let lot: u32 = lot_input.get_untracked().parse().unwrap_or(1);

        if sym.is_empty() || buy <= 0.0 || sell <= 0.0 {
            return;
        }
        saving.set(true);

        spawn_local(async move {
            match api::create_rule(&sym, buy, sell, sl, lot).await {
                Ok(_) => {
                    sym_input.set(String::new());
                    buy_input.set(String::new());
                    sell_input.set(String::new());
                    sl_input.set(String::new());
                    lot_input.set(String::new());
                    load_rules();
                }
                Err(e) => error.set(Some(e)),
            }
            saving.set(false);
        });
    };

    let on_delete = move |symbol: String| {
        spawn_local(async move {
            if api::delete_rule(&symbol).await.is_ok() {
                load_rules();
            }
        });
    };

    view! {
        <div>
            <div class="page-header">
                <h1>"Trade Rules"</h1>
                <p class="subtitle">
                    "Configure auto-trade buy/sell triggers"
                </p>
            </div>

            <section class="section">
                <span class="section-title">"Create Rule"</span>
                <div class="rule-form">
                    <input class="input" type="text"
                        placeholder="Symbol"
                        prop:value=move || sym_input.get()
                        on:input=move |ev| {
                            sym_input.set(event_target_value(&ev))
                        }
                    />
                    <input class="input" type="number"
                        placeholder="Buy below"
                        prop:value=move || buy_input.get()
                        on:input=move |ev| {
                            buy_input.set(event_target_value(&ev))
                        }
                    />
                    <input class="input" type="number"
                        placeholder="Sell above"
                        prop:value=move || sell_input.get()
                        on:input=move |ev| {
                            sell_input.set(event_target_value(&ev))
                        }
                    />
                    <input class="input" type="number"
                        placeholder="Stop loss"
                        prop:value=move || sl_input.get()
                        on:input=move |ev| {
                            sl_input.set(event_target_value(&ev))
                        }
                    />
                    <input class="input" type="number"
                        placeholder="Max lot"
                        prop:value=move || lot_input.get()
                        on:input=move |ev| {
                            lot_input.set(event_target_value(&ev))
                        }
                    />
                    <button
                        class="btn btn-primary"
                        on:click=on_create
                        disabled=move || saving.get()
                    >
                        {move || {
                            if saving.get() {
                                "Saving..."
                            } else {
                                "Add Rule"
                            }
                        }}
                    </button>
                </div>
            </section>

            <section class="section">
                <span class="section-title">"Active Rules"</span>
                {move || {
                    if loading.get() {
                        view! {
                            <div class="loading">
                                <div class="spinner"></div>
                                <span>"Loading rules..."</span>
                            </div>
                        }.into_any()
                    } else {
                        let items = rules.get();
                        if items.is_empty() {
                            view! {
                                <div class="placeholder">
                                    <p>"No trade rules configured"</p>
                                    <p style="font-size: 0.8rem; margin-top: 0.25rem; color: var(--text-dim)">
                                        "Add a rule above to get started"
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <table class="table">
                                    <thead>
                                        <tr>
                                            <th>"Symbol"</th>
                                            <th>"Buy Below"</th>
                                            <th>"Sell Above"</th>
                                            <th>"Stop Loss"</th>
                                            <th>"Max Lot"</th>
                                            <th></th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {items
                                            .into_iter()
                                            .map(|r| {
                                                let sym = r.symbol.clone();
                                                let del_sym = r.symbol.clone();
                                                view! {
                                                    <tr>
                                                        <td class="font-bold">
                                                            {sym}
                                                        </td>
                                                        <td class="text-green">
                                                            {format!(
                                                                "Rp {}",
                                                                r.buy_below,
                                                            )}
                                                        </td>
                                                        <td class="text-red">
                                                            {format!(
                                                                "Rp {}",
                                                                r.sell_above,
                                                            )}
                                                        </td>
                                                        <td>
                                                            {format!(
                                                                "Rp {}",
                                                                r.stop_loss,
                                                            )}
                                                        </td>
                                                        <td>{r.max_lot}</td>
                                                        <td>
                                                            <button
                                                                class="btn btn-danger btn-sm"
                                                                on:click=move |_| {
                                                                    on_delete(
                                                                        del_sym.clone(),
                                                                    )
                                                                }
                                                            >
                                                                "Remove"
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                            })
                                            .collect::<Vec<_>>()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }
                    }
                }}
            </section>
        </div>
    }
}
