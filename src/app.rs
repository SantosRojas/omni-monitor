use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{ParentRoute, Redirect, Route, Router, Routes};
use leptos_router::path;

use crate::components::layout::Layout;
use crate::pages::admin_machine_ips::AdminMachineIpsPage;
use crate::pages::admin_users::AdminUsersPage;
use crate::pages::login::LoginPage;
use crate::pages::patient_dashboard::PatientDashboardPage;
use crate::pages::patient_detail::PatientDetail;
use crate::pages::patient_history::PatientHistory;
use crate::pages::patients::PatientsPage;
use crate::pages::therapy_detail::TherapyDetailPage;
use crate::utils::auth::AuthContext;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let auth = AuthContext::new();
    provide_context(auth.clone());

    let global_error = RwSignal::new(None::<String>);
    provide_context(global_error);

    view! {
        <Html attr:lang="es" />
        <Title text="Monitor OMNI" />
        <Link rel="stylesheet" href="/styles/glassmorphism.css" />

        <Router>
            <Routes fallback=|| { view! { <div class="content-area"><div class="empty-state"><p>"Página no encontrada"</p></div></div> }.into_any() }>
                <Route path=path!("/login") view=LoginPage />
                <Route path=path!("") view=move || {
                    let auth = use_context::<AuthContext>().unwrap();
                    if auth.is_logged_in() {
                        view! { <Redirect path="/patients"/> }.into_any()
                    } else {
                        view! { <Redirect path="/login"/> }.into_any()
                    }
                } />
                <ParentRoute path=path!("") view=Layout>
                    <Route path=path!("patients") view=PatientsPage />
                    <Route path=path!("patients/:id") view=PatientDetail />
                    <Route path=path!("patients/:id/history") view=PatientHistory />
                    <Route path=path!("patients/:id/dashboard") view=PatientDashboardPage />
                    <Route path=path!("therapies/:id") view=TherapyDetailPage />
                    <Route path=path!("admin/machine-ips") view=AdminMachineIpsPage />
                    <Route path=path!("admin/users") view=AdminUsersPage />
                </ParentRoute>
            </Routes>
        </Router>
    }
}
