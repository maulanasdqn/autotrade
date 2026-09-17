use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::stock_card::StockCard;
use crate::dto::{PortfolioResponse, SuggestionDto, TokenStatus};

#[component]
pub fn Dashboard() -> impl IntoView {
    let suggestions = RwSignal::new(Vec::<SuggestionDto>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(Option::<String>::None);
    let portfolio = RwSignal::new(Option::<PortfolioResponse>::None);
    let token_status = RwSignal::new(Option::<TokenStatus>::None);
    let show_token_input = RwSignal::new(false);
    let token_input = RwSignal::new(String::new());
    let token_updating = RwSignal::new(false);

    spawn_local(async move {
        if let Ok(ts) = api::fetch_token_status().await {
            token_status.set(Some(ts));
        }
        if let Ok(p) = api::fetch_portfolio().await {
            portfolio.set(Some(p));
        }
        match api::fetch_suggestions(5).await {
            Ok(data) => suggestions.set(data),
            Err(e) => error.set(Some(e)),
        }
        loading.set(false);
    });

    view! {
        <div>
            {move || {
                token_status.get().map(|ts| {
                    let (status_text, status_class) = if ts.valid {
                        (
                            format!(
                                "Token valid — expires in {}",
                                ts.ttl_human.unwrap_or_default(),
                            ),
                            "token-bar token-valid",
                        )
                    } else {
                        (
                            "Token expired — update required".to_string(),
                            "token-bar token-expired",
                        )
                    };

                    view! {
                        <div class={status_class}
                            style="margin-bottom: 1rem">
                            <div style="display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; flex-wrap: wrap">
                                <span style="font-size: 0.8rem">
                                    {status_text}
                                </span>
                                <button
                                    class="btn btn-sm"
                                    on:click=move |_| {
                                        show_token_input
                                            .set(!show_token_input.get());
                                    }
                                >
                                    {move || {
                                        if show_token_input.get() {
                                            "Cancel"
                                        } else {
                                            "Update Token"
                                        }
                                    }}
                                </button>
                            </div>
                            {move || {
                                show_token_input.get().then(|| {
                                    view! {
                                        <div style="margin-top: 0.5rem; display: flex; gap: 0.5rem">
                                            <input
                                                type="text"
                                                class="input"
                                                placeholder="Paste new Bearer token here..."
                                                style="flex: 1; font-size: 0.75rem; padding: 0.35rem 0.5rem"
                                                prop:value=move || token_input.get()
                                                on:input=move |ev| {
                                                    token_input.set(
                                                        event_target_value(&ev),
                                                    );
                                                }
                                            />
                                            <button
                                                class="btn btn-sm btn-primary"
                                                prop:disabled=move || {
                                                    token_updating.get()
                                                        || token_input.get().trim().is_empty()
                                                }
                                                on:click=move |_| {
                                                    let val = token_input.get();
                                                    let t = val
                                                        .trim()
                                                        .trim_start_matches("Bearer ")
                                                        .to_string();
                                                    if t.is_empty() {
                                                        return;
                                                    }
                                                    token_updating.set(true);
                                                    spawn_local(async move {
                                                        match api::update_token(&t).await {
                                                            Ok(new_status) => {
                                                                token_status
                                                                    .set(Some(new_status));
                                                                show_token_input.set(false);
                                                                token_input
                                                                    .set(String::new());
                                                            }
                                                            Err(_e) => {}
                                                        }
                                                        token_updating.set(false);
                                                    });
                                                }
                                            >
                                                {move || {
                                                    if token_updating.get() {
                                                        "Updating..."
                                                    } else {
                                                        "Save"
                                                    }
                                                }}
                                            </button>
                                        </div>
                                    }
                                })
                            }}
                        </div>
                    }
                })
            }}

            <div class="page-header">
                <h1>"Dashboard"</h1>
                <p class="subtitle">"AI-powered stock insights from IDX"</p>
            </div>

            {move || {
                portfolio.get().map(|p| {
                    let total_equity: f64 =
                        p.total_equity.parse().unwrap_or(0.0);
                    let cash: f64 =
                        p.cash_balance.parse().unwrap_or(0.0);
                    let invested = total_equity - cash;
                    let total_pnl: f64 = p
                        .positions
                        .iter()
                        .map(|pos| {
                            pos.unrealized_pnl.parse::<f64>().unwrap_or(0.0)
                        })
                        .sum();
                    let pnl_pct = if invested > 0.0 {
                        (total_pnl / invested) * 100.0
                    } else {
                        0.0
                    };

                    view! {
                        <div class="stats-grid" style="margin-bottom: 1.5rem">
                            <div class="stat-card">
                                <div class="stat-card-title">
                                    "Portfolio Value"
                                </div>
                                <div class="stat-item">
                                    <span class="label">"Total Equity"</span>
                                    <span class="value">
                                        {format_rp(total_equity)}
                                    </span>
                                </div>
                                <div class="stat-item">
                                    <span class="label">"Cash Balance"</span>
                                    <span class="value">
                                        {format_rp(cash)}
                                    </span>
                                </div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-card-title">
                                    "Profit & Loss"
                                </div>
                                <div class="stat-item">
                                    <span class="label">
                                        "Unrealized P&L"
                                    </span>
                                    <span class={pnl_class(total_pnl)}>
                                        {format_pnl(total_pnl)}
                                    </span>
                                </div>
                                <div class="stat-item">
                                    <span class="label">"Return"</span>
                                    <span class={pnl_class(pnl_pct)}>
                                        {format!("{:+.2}%", pnl_pct)}
                                    </span>
                                </div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-card-title">
                                    "Positions"
                                </div>
                                {if p.positions.is_empty() {
                                    view! {
                                        <div class="stat-item">
                                            <span class="label">
                                                "Status"
                                            </span>
                                            <span class="value"
                                                style="font-size: 0.85rem">
                                                "No open positions"
                                            </span>
                                        </div>
                                    }
                                    .into_any()
                                } else {
                                    view! {
                                        <div>
                                            {p.positions
                                                .iter()
                                                .map(|pos| {
                                                    let pnl: f64 = pos
                                                        .unrealized_pnl
                                                        .parse()
                                                        .unwrap_or(0.0);
                                                    let avg: f64 = pos
                                                        .avg_price
                                                        .parse()
                                                        .unwrap_or(0.0);
                                                    let cur: f64 = pos
                                                        .current_price
                                                        .parse()
                                                        .unwrap_or(0.0);
                                                    let pos_pct = if avg > 0.0 {
                                                        ((cur - avg) / avg) * 100.0
                                                    } else {
                                                        0.0
                                                    };
                                                    let sym = pos.symbol.clone();
                                                    view! {
                                                        <div class="stat-item"
                                                            style="display: flex; justify-content: space-between; align-items: center">
                                                            <div>
                                                                <span
                                                                    style="font-weight: 700; font-size: 0.85rem">
                                                                    {sym}
                                                                </span>
                                                                <span
                                                                    style="color: var(--text-dim); font-size: 0.75rem; margin-left: 0.35rem">
                                                                    {format!(
                                                                        "{} lot",
                                                                        pos.lot,
                                                                    )}
                                                                </span>
                                                            </div>
                                                            <span class={pnl_class(
                                                                pnl,
                                                            )}>
                                                                {format_pnl(pnl)}
                                                                " "
                                                                {format!(
                                                                    "({:+.1}%)",
                                                                    pos_pct,
                                                                )}
                                                            </span>
                                                        </div>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                    }
                                    .into_any()
                                }}
                            </div>
                        </div>
                    }
                })
            }}

            <section class="section">
                <span class="section-title">"AI Stock Suggestions"</span>
                {move || {
                    if loading.get() {
                        view! {
                            <div class="loading">
                                <div class="spinner"></div>
                                <span>"Fetching AI suggestions..."</span>
                            </div>
                        }.into_any()
                    } else if let Some(err) = error.get() {
                        view! {
                            <div class="error-msg">{err}</div>
                        }.into_any()
                    } else {
                        let items = suggestions.get();
                        if items.is_empty() {
                            view! {
                                <p class="placeholder">
                                    "No suggestions available"
                                </p>
                            }.into_any()
                        } else {
                            view! {
                                <div class="card-grid">
                                    {items
                                        .into_iter()
                                        .map(|s| {
                                            view! { <StockCard suggestion={s} /> }
                                        })
                                        .collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </section>
        </div>
    }
}

fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push('.');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn format_rp(val: f64) -> String {
    let abs = val.abs() as u64;
    let formatted = if abs >= 1_000_000_000 {
        format!("{:.1}B", val / 1_000_000_000.0)
    } else {
        thousands(abs)
    };
    if val < 0.0 {
        format!("Rp -{formatted}")
    } else {
        format!("Rp {formatted}")
    }
}

fn format_pnl(val: f64) -> String {
    let prefix = if val >= 0.0 { "+" } else { "-" };
    let abs = val.abs() as u64;
    let formatted = if abs >= 1_000_000_000 {
        format!("{:.1}B", val.abs() / 1_000_000_000.0)
    } else {
        thousands(abs)
    };
    format!("Rp {prefix}{formatted}")
}

fn pnl_class(val: f64) -> &'static str {
    if val > 0.0 {
        "value value-green"
    } else if val < 0.0 {
        "value value-red"
    } else {
        "value"
    }
}
