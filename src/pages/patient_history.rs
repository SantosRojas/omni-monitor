use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos::task::spawn_local;
use crate::models::{TelemetryReading, PaginatedResponse};
use crate::utils::api;
use crate::utils::excel;
use crate::components::loading_state::LoadingState;
use crate::components::empty_state::EmptyState;

#[component]
pub fn PatientHistory() -> impl IntoView {
    let global_error = use_context::<RwSignal<Option<String>>>().expect("Global error context not provided");
    let params = use_params_map();
    let id = move || params.read().get("id").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);

    let page = RwSignal::new(1);
    let data = RwSignal::new(Option::<PaginatedResponse<TelemetryReading>>::None);
    let loading = RwSignal::new(true);

    let fetch = move |p: i64| {
        loading.set(true);
        let pid = id();
        spawn_local(async move {
            if pid > 0 {
                match api::get_patient_history(pid, p, 50).await {
                    Ok(d) => data.set(Some(d)),
                    Err(e) => global_error.set(Some(e)),
                }
            }
            loading.set(false);
        });
    };

    fetch(1);

    view! {
        <div class="page-title">Historial de Telemetría</div>

        <div class="action-bar">
            <button class="btn" on:click=move |_| { let pid = id(); if pid > 0 { excel::trigger_download(pid); } }>
                "📥 Exportar Excel"
            </button>
            <span style="font-size:0.85rem;color:var(--text-muted);">Datos crudos de telemetría</span>
        </div>

        {move || {
            if loading.get() {
                return view! { <LoadingState message="Cargando historial..." /> }.into_any();
            }

            match data.get() {
                Some(d) if !d.data.is_empty() => {
                    let cloned = d.clone();
                    view! {
                        <div class="table-container glass" style="padding:0;overflow:hidden;">
                            <table>
                                <thead>
                                    <tr>
                                        <th>Timestamp</th>
                                        <th>Señal ID</th>
                                        <th>Valor</th>
                                        <th>Unidad</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {cloned.data.into_iter().map(|r| view! {
                                        <tr>
                                            <td>{r.timestamp.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())}</td>
                                            <td>{r.signal_id.map(|s| s.to_string())}</td>
                                            <td>{r.physical_value.unwrap_or_default()}</td>
                                            <td>{r.unit.unwrap_or_default()}</td>
                                        </tr>
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </div>
                        <div class="pagination">
                            <button class="btn btn-sm" disabled=move || page.get() <= 1
                                on:click=move |_| { let p = page.get() - 1; page.set(p); fetch(p); }>
                                "◀"
                            </button>
                            <span style="font-size:0.85rem;color:var(--text-muted);">Página {page.get()} de {cloned.total_pages} ({cloned.total} registros)</span>
                            <button class="btn btn-sm" disabled=move || page.get() >= cloned.total_pages
                                on:click=move |_| { let p = page.get() + 1; page.set(p); fetch(p); }>
                                "▶"
                            </button>
                        </div>
                    }.into_any()
                }
                _ => view! { <EmptyState message="No hay datos de telemetría" /> }.into_any()
            }
        }}
    }
}
