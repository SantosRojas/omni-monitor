use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos::task::spawn_local;
use crate::models::PatientDashboard;
use crate::utils::api;
use crate::components::dashboard_chart::DashboardChart;
use crate::components::stats_card::StatsCard;
use crate::components::loading_state::LoadingState;
use crate::components::empty_state::EmptyState;

#[component]
pub fn TherapyDetailPage() -> impl IntoView {
    let global_error = use_context::<RwSignal<Option<String>>>().expect("Global error context not provided");
    let params = use_params_map();
    let id = move || params.read().get("id").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);

    let dashboard = RwSignal::new(Option::<PatientDashboard>::None);
    let loading = RwSignal::new(true);

    spawn_local(async move {
        let tid = id();
        if tid > 0 {
            match api::get_therapy_dashboard(tid).await {
                Ok(d) => dashboard.set(Some(d)),
                Err(e) => global_error.set(Some(e)),
            }
        }
        loading.set(false);
    });

    view! {
        <div class="page-title">Terapia #{id()}</div>

        {move || {
            if loading.get() {
                return view! { <LoadingState message="Cargando..." /> }.into_any();
            }

            match dashboard.get() {
                None => view! { <EmptyState message="No hay datos de esta terapia" /> }.into_any(),
                Some(d) => {
                    let signals = d.signals.clone();
                    view! {
                        {signals.iter().map(|s| {
                            let signal = s.clone();
                            view! {
                                <div class="card glass" style="margin-bottom:24px;">
                                    <div class="card-title">{signal.display_name.clone().unwrap_or(signal.internal_name.clone())}</div>
                                    <div class="stats-grid">
                                        <StatsCard label="Promedio".to_string() value={signal.average.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "—".to_string())} color=Some("var(--accent)".to_string()) />
                                        <StatsCard label="Mínimo".to_string() value={signal.minimum.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "—".to_string())} color=Some("var(--success)".to_string()) />
                                        <StatsCard label="Máximo".to_string() value={signal.maximum.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "—".to_string())} color=Some("var(--danger)".to_string()) />
                                        <StatsCard label="Muestras".to_string() value={signal.count.to_string()} color=Some("var(--warning)".to_string()) />
                                    </div>
                                    <DashboardChart signal=signal width=800.0 height=250.0 />
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    }.into_any()
                }
            }
        }}
    }
}
