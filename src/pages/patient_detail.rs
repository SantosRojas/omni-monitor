use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos_router::components::A;
use leptos::task::spawn_local;
use crate::models::{Patient, TherapyWithMachine, ActiveDevice};
use crate::utils::api;
use crate::components::loading_state::LoadingState;
use crate::components::empty_state::EmptyState;

#[component]
pub fn PatientDetail() -> impl IntoView {
    let global_error = use_context::<RwSignal<Option<String>>>().expect("Global error context not provided");
    let params = use_params_map();
    let id = move || params.read().get("id").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);

    let patient = RwSignal::new(Option::<Patient>::None);
    let therapies = RwSignal::new(Option::<Vec<TherapyWithMachine>>::None);
    let active_device = RwSignal::new(Option::<ActiveDevice>::None);
    let loading = RwSignal::new(true);

    spawn_local(async move {
        let pid = id();
        if pid > 0 {
            let (p, t, d) = futures::join!(
                api::get_patient(pid),
                api::get_patient_therapies(pid),
                api::get_active_device(pid),
            );
            if let Err(e) = &p { global_error.set(Some(e.clone())); }
            if let Ok(p) = p { patient.set(Some(p)); }
            if let Ok(t) = t { therapies.set(Some(t)); }
            if let Ok(d) = d { active_device.set(Some(d)); }
        }
        loading.set(false);
    });

    let redirect_to_device = move |url: String| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().assign(&url);
        }
    };

    view! {
        {move || {
            if loading.get() {
                return view! { <LoadingState message="Cargando..." /> }.into_any();
            }

            let p = match patient.get() {
                Some(p) => p,
                None => return view! { <EmptyState message="Paciente no encontrado" /> }.into_any(),
            };

            view! {
                <div class="page-title">Paciente: {p.patient_id_str.clone()}</div>

                <div class="card glass">
                    <div class="card-title">Información del Paciente</div>
                    <div class="grid-2">
                        <div class="form-group">
                            <div class="form-label">ID</div>
                            <div>{p.id}</div>
                        </div>
                        <div class="form-group">
                            <div class="form-label">Identificador</div>
                            <div>{p.patient_id_str}</div>
                        </div>
                        <div class="form-group">
                            <div class="form-label">Registrado</div>
                            <div>{p.created_at.map(|t| t.format("%Y-%m-%d %H:%M").to_string())}</div>
                        </div>
                        <div class="form-group">
                            <div class="form-label">Inicio Terapia</div>
                            <div>{p.therapy_start.map(|t| t.format("%Y-%m-%d %H:%M").to_string())}</div>
                        </div>
                    </div>
                </div>

                <div class="action-bar">
                    <A href={format!("/patients/{}/history", p.id)} attr:class="btn">
                        "📊 Ver Historial"
                    </A>
                    <A href={format!("/patients/{}/dashboard", p.id)} attr:class="btn btn-primary">
                        "📈 Dashboard"
                    </A>
                    {move || active_device.get().map(|d| {
                        view! {
                            <button class="btn" on:click=move |_| redirect_to_device(d.url.clone())>
                                "🔗 Ir a Máquina OMNI"
                            </button>
                        }
                    })}
                </div>

                <div class="card glass">
                    <div class="card-title">Terapias</div>
                    {move || match therapies.get() {
                        None => view! { <div class="loading"><div class="spinner"></div></div> }.into_any(),
                        Some(t) if t.is_empty() => view! { <EmptyState message="Sin terapias registradas" /> }.into_any(),
                        Some(therapies) => {
                            view! {
                                <div class="table-container">
                                    <table>
                                        <thead>
                                            <tr>
                                                <th>ID</th>
                                                <th>Inicio</th>
                                                <th>Fin</th>
                                                <th>Estado</th>
                                                <th>Máquina</th>
                                                <th></th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {therapies.into_iter().map(|t| view! {
                                                <tr>
                                                    <td>{t.id}</td>
                                                    <td>{t.started_at.map(|t| t.format("%Y-%m-%d %H:%M").to_string())}</td>
                                                    <td>{t.ended_at.map(|t| t.format("%Y-%m-%d %H:%M").to_string())}</td>
                                                    <td>
                                                        <span class={format!("badge badge-{}", if t.status.as_deref() == Some("active") { "active" } else { "inactive" })}>
                                                            {t.status.clone().unwrap_or_default()}
                                                        </span>
                                                    </td>
                                                    <td style="font-size:0.8rem">{t.serial_number.unwrap_or_default()}</td>
                                                    <td>
                                                        <A href={format!("/therapies/{}", t.id)} attr:class="btn btn-sm">"Ver"</A>
                                                    </td>
                                                </tr>
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            }.into_any()
        }}
    }
}
