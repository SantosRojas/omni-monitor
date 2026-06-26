use leptos::prelude::*;
use leptos_router::components::{A, Outlet};
use leptos_router::hooks::use_location;
use crate::utils::auth::use_auth;
use crate::components::error_banner::ErrorBanner;

#[component]
pub fn Layout() -> impl IntoView {
    let auth = use_auth();
    let location = use_location();
    let global_error = use_context::<RwSignal<Option<String>>>().expect("Global error context not provided");

    let is_active = move |path: &'static str| -> &'static str {
        if location.pathname.get().starts_with(path) { "active" } else { "" }
    };

    let auth2 = auth.clone();
    let auth3 = auth.clone();
    let logout = move |_: leptos::ev::MouseEvent| { auth3.logout(); leptos_router::hooks::use_navigate()("/login", Default::default()); };

    view! {
        <div class="app-layout">
            <nav class="sidebar">
                <div class="sidebar-logo">Monitor <span>OMNI</span></div>
                <A href="/patients" attr:class={move || format!("sidebar-link {}", is_active("/patients"))}>
                    <span>"📋"</span>
                    <span>"Pacientes"</span>
                </A>
                {move || (auth2.is_admin()).then(|| view! {
                    <>
                        <A href="/admin/machine-ips" attr:class={move || format!("sidebar-link {}", is_active("/admin/machine-ips"))}>
                            <span>"🔧"</span>
                            <span>"IPs de Máquinas"</span>
                        </A>
                        <A href="/admin/users" attr:class={move || format!("sidebar-link {}", is_active("/admin/users"))}>
                            <span>"👥"</span>
                            <span>"Usuarios"</span>
                        </A>
                    </>
                })}
                <div style="flex:1"></div>
                <div class="sidebar-link" on:click=logout>
                    <span>"🚪"</span>
                    <span>"Salir"</span>
                </div>
            </nav>

            <div class="main-area">
                <header class="topbar">
                    <div class="topbar-title">
                        {move || match location.pathname.get().as_str() {
                            p if p == "/patients" || p == "/" => "Pacientes",
                            p if p.starts_with("/patients/") => "Detalle del Paciente",
                            p if p.starts_with("/admin/machine-ips") => "IPs de Máquinas",
                            p if p.starts_with("/admin/users") => "Usuarios",
                            _ => "Monitor OMNI",
                        }}
                    </div>
                    <div class="topbar-user">
                        {move || auth.user.get().map(|u| view! { <>{u.full_name.clone()} ({u.role.clone()})</> })}
                    </div>
                </header>
                <div class="content-area">
                    <ErrorBanner error=global_error message="Error de conexión con el servidor. Verifique su conexión e intente nuevamente." />
                    <Outlet />
                </div>
            </div>
        </div>
    }
}
