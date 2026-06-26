use leptos::prelude::*;

#[component]
pub fn ErrorBanner(
    error: RwSignal<Option<String>>,
    #[prop(optional)] message: &'static str,
) -> impl IntoView {
    move || error.get().map(|msg| {
        let msg_clone = msg.clone();
        view! {
            <div class="login-error" style="display:flex;flex-direction:column;gap:4px;">
                <div style="display:flex;justify-content:space-between;align-items:center;">
                    <span>{message}</span>
                    <button on:click=move |_| error.set(None) style="background:none;border:none;color:var(--danger);cursor:pointer;font-size:1.2rem;padding:0 4px;">"×"</button>
                </div>
                <small style="opacity:0.7;font-size:0.75rem;">{msg_clone}</small>
            </div>
        }
    })
}
