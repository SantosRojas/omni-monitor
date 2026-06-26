use std::sync::Arc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::utils::auth::use_auth;
use crate::utils::api;

#[component]
pub fn LoginForm(on_success: Arc<dyn Fn() + Send + Sync>) -> impl IntoView {
    let auth = use_auth();
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let loading = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        loading.set(true);
        error.set(None);
        let u = username.get();
        let p = password.get();
        let auth = auth.clone();
        let on_success = on_success.clone();
        spawn_local(async move {
            match api::login(&u, &p).await {
                Ok(resp) => {
                    auth.login(&resp.token, resp.user);
                    on_success();
                }
                Err(e) => {
                    error.set(Some(e));
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="login-page">
            <div class="login-card glass">
                <div class="login-title">Monitor OMNI</div>
                <div class="login-subtitle">Ingrese sus credenciales</div>

                {move || error.get().map(|e| view! { <div class="login-error">{e}</div> })}

                <form on:submit=on_submit>
                    <div class="form-group">
                        <label class="form-label">Usuario</label>
                        <input
                            class="input"
                            type="text"
                            placeholder="Nombre de usuario"
                            prop:value=move || username.get()
                            on:input=move |ev| username.set(event_target_value(&ev))
                            disabled=move || loading.get()
                        />
                    </div>
                    <div class="form-group">
                        <label class="form-label">Contraseña</label>
                        <input
                            class="input"
                            type="password"
                            placeholder="Contraseña"
                            prop:value=move || password.get()
                            on:input=move |ev| password.set(event_target_value(&ev))
                            disabled=move || loading.get()
                        />
                    </div>
                    <button
                        type="submit"
                        class="btn btn-primary w-full"
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Ingresando..." } else { "Ingresar" }}
                    </button>
                </form>
            </div>
        </div>
    }
}
