use leptos::prelude::*;

#[component]
pub fn LoadingState(
    #[prop(optional)]
    #[prop(default = "Cargando...")]
    message: &'static str,
) -> impl IntoView {
    view! {
        <div class="loading">
            <div class="spinner"></div>
            <p>{message}</p>
        </div>
    }
}
