#[cfg(feature = "frontend")]
use leptos::prelude::*;

#[component]
pub fn StatsCard(
    label: String,
    value: String,
    color: Option<String>,
) -> impl IntoView {
    let color = color.unwrap_or_else(|| "var(--accent)".to_string());
    view! {
        <div class="stat-card glass-sm">
            <div class="stat-value" style=format!("color: {}", color)>
                {value}
            </div>
            <div class="stat-label">{label}</div>
        </div>
    }
}
