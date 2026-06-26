#[cfg(feature = "frontend")]
use leptos::prelude::*;
use std::sync::Arc;
use leptos_router::hooks::use_navigate;
use crate::components::login_form::LoginForm;

#[component]
pub fn LoginPage() -> impl IntoView {
    let navigate = use_navigate();

    let on_success = Arc::new(move || {
        navigate("/patients", Default::default());
    });

    view! {
        <LoginForm on_success=on_success />
    }
}
