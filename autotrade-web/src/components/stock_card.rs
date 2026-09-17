use leptos::prelude::*;

use crate::components::signal_badge::SignalBadge;
use crate::dto::SuggestionDto;

#[component]
pub fn StockCard(suggestion: SuggestionDto) -> impl IntoView {
    let symbol = suggestion.symbol.clone();
    let name = suggestion.name.clone();
    let current_price = suggestion.current_price.to_string();
    let target_price = suggestion.target_price.to_string();
    let potential_return = format!("{}%", suggestion.potential_return);
    let reason = suggestion.reason.clone();

    view! {
        <div class="stock-card">
            <div class="stock-header">
                <h3>{symbol}</h3>
                <SignalBadge signal={suggestion.signal.clone()} />
            </div>
            <p class="stock-name">{name}</p>
            <div class="stock-metrics">
                <div>
                    <span class="label">"Price"</span>
                    <span class="value">{current_price}</span>
                </div>
                <div>
                    <span class="label">"Target"</span>
                    <span class="value">{target_price}</span>
                </div>
                <div>
                    <span class="label">"Return"</span>
                    <span class="value">{potential_return}</span>
                </div>
            </div>
            <p class="stock-reason">{reason}</p>
        </div>
    }
}
