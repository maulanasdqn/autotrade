use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::dto::AnalysisResponse;

#[component]
pub fn Analysis() -> impl IntoView {
    let symbol = RwSignal::new(String::new());
    let result = RwSignal::new(Option::<AnalysisResponse>::None);
    let loading = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let do_analyze = move || {
        let sym = symbol.get_untracked();
        if sym.is_empty() {
            return;
        }
        loading.set(true);
        error.set(None);
        result.set(None);

        spawn_local(async move {
            match api::analyze_stock(&sym.to_uppercase()).await {
                Ok(data) => result.set(Some(data)),
                Err(e) => error.set(Some(e)),
            }
            loading.set(false);
        });
    };

    view! {
        <div>
            <div class="page-header">
                <h1>"Stock Analysis"</h1>
                <p class="subtitle">
                    "IDX market data + DeepSeek AI insights"
                </p>
            </div>

            <section class="section">
                <div class="analyze-form">
                    <input
                        class="input"
                        type="text"
                        placeholder="Enter stock symbol (e.g. BBCA)"
                        on:input=move |ev| {
                            symbol.set(event_target_value(&ev));
                        }
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                do_analyze();
                            }
                        }
                    />
                    <button
                        class="btn btn-primary"
                        on:click=move |_| do_analyze()
                        disabled=move || loading.get()
                    >
                        {move || {
                            if loading.get() {
                                "Analyzing..."
                            } else {
                                "Analyze"
                            }
                        }}
                    </button>
                </div>
            </section>

            {move || {
                error.get().map(|e| {
                    view! {
                        <div class="error-msg">{e}</div>
                    }
                })
            }}

            {move || {
                if loading.get() {
                    Some(view! {
                        <div class="loading">
                            <div class="spinner"></div>
                            <span>"Analyzing with DeepSeek AI..."</span>
                        </div>
                    })
                } else {
                    None
                }
            }}

            {move || {
                result.get().map(|a| {
                    let signal_class = match a.signal.as_str() {
                        "StrongBuy" => "badge badge-strong-buy",
                        "Buy" => "badge badge-buy",
                        "Hold" => "badge badge-hold",
                        "Sell" => "badge badge-sell",
                        "StrongSell" => "badge badge-strong-sell",
                        _ => "badge",
                    };
                    let risks = a.risk_factors.clone().unwrap_or_default();
                    let conf_num: f64 = a.confidence.parse().unwrap_or(0.0);
                    let conf_class = if conf_num >= 70.0 {
                        "value value-green"
                    } else if conf_num >= 50.0 {
                        "value"
                    } else {
                        "value value-red"
                    };
                    let confidence = a.confidence;

                    view! {
                        <section class="section analysis-result">
                            <div class="analysis-header">
                                <h2>{a.symbol.clone()}</h2>
                                <span class={signal_class}>
                                    {a.signal}
                                </span>
                            </div>

                            <div class="stats-grid">
                                <div class="stat-card">
                                    <div class="stat-card-title">
                                        "Trading Insights"
                                    </div>
                                    <div class="stat-item">
                                        <span class="label">"Trend"</span>
                                        <span class="value">{a.trend}</span>
                                    </div>
                                    <div class="stat-item">
                                        <span class="label">
                                            "Confidence"
                                        </span>
                                        <span class={conf_class}>
                                            {confidence}"%"
                                        </span>
                                    </div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-card-title">
                                        "Price Targets"
                                    </div>
                                    <div class="stat-item">
                                        <span class="label">
                                            "Entry Price"
                                        </span>
                                        <span class="value">
                                            {format!("Rp {}", a.entry)}
                                        </span>
                                    </div>
                                    <div class="stat-item">
                                        <span class="label">
                                            "Take Profit"
                                        </span>
                                        <span class="value value-green">
                                            {format!("Rp {}", a.take_profit)}
                                        </span>
                                    </div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-card-title">
                                        "Risk Management"
                                    </div>
                                    <div class="stat-item">
                                        <span class="label">
                                            "Stop Loss"
                                        </span>
                                        <span class="value value-red">
                                            {format!("Rp {}", a.stop_loss)}
                                        </span>
                                    </div>
                                    <div class="stat-item">
                                        <span class="label">
                                            "Risk Factors"
                                        </span>
                                        <span class="value">
                                            {risks.len()}" identified"
                                        </span>
                                    </div>
                                </div>
                            </div>

                            {if !risks.is_empty() {
                                Some(view! {
                                    <div class="risk-section">
                                        <div class="risk-section-title">
                                            "Risk Factors"
                                        </div>
                                        <div>
                                            {risks
                                                .into_iter()
                                                .map(|r| {
                                                    view! {
                                                        <span class="risk-tag">
                                                            {r}
                                                        </span>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                })
                            } else {
                                None
                            }}

                            <div class="reasoning-card">
                                <div class="reasoning-card-header">
                                    <div class="reasoning-dot"></div>
                                    <span>"AI Insights"</span>
                                    <span class="reasoning-label">
                                        " \u{2014} DeepSeek Analysis"
                                    </span>
                                </div>
                                <p>{a.reasoning}</p>
                            </div>
                        </section>
                    }
                })
            }}
        </div>
    }
}
