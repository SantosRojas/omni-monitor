use std::sync::Arc;
use leptos::prelude::*;

#[component]
pub fn Modal(
    title: String,
    on_close: Arc<dyn Fn() + Send + Sync>,
    children: Children,
) -> impl IntoView {
    let on_close_overlay = on_close.clone();

    view! {
        <div class="modal-overlay" on:click=move |_| on_close_overlay()>
            <div class="modal glass" on:click=move |ev| ev.stop_propagation()>
                <div class="modal-title">{title}</div>
                {children()}
            </div>
        </div>
    }
}
