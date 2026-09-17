use leptos::prelude::*;

#[component]
pub fn SignalBadge(signal: String) -> impl IntoView {
    let class = match signal.as_str() {
        "StrongBuy" => "badge badge-strong-buy",
        "Buy" => "badge badge-buy",
        "Hold" => "badge badge-hold",
        "Sell" => "badge badge-sell",
        "StrongSell" => "badge badge-strong-sell",
        _ => "badge",
    };

    view! {
        <span class={class}>{signal}</span>
    }
}
