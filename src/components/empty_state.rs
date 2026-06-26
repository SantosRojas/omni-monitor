use leptos::prelude::*;

#[component]
pub fn EmptyState(
    #[prop(optional)]
    #[prop(default = "Sin datos")]
    message: &'static str,
) -> impl IntoView {
    view! {
        <div class="empty-state">
            <p>{message}</p>
        </div>
    }
}
