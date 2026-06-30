use std::sync::Arc;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos::task::spawn_local;
use crate::models::{TelemetryReading, PaginatedResponse};
use crate::utils::api;
use crate::utils::excel;
use crate::components::table::{DataTable, ColumnDef};
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

    let fetch_loading = loading.clone();
    let fetch_data = data.clone();
    let fetch_error = global_error.clone();

    let fetch_for_history = Arc::new(move |p: i64| {
        let pid = id();
        fetch_loading.set(true);
        let d = fetch_data.clone();
        let e = fetch_error.clone();
        let l = fetch_loading.clone();
        spawn_local(async move {
            if pid > 0 {
                match api::get_patient_history(pid, p, 50).await {
                    Ok(res) => d.set(Some(res)),
                    Err(err) => e.set(Some(err)),
                }
            }
            l.set(false);
        });
    });

    fetch_for_history(1);

    let page_for_page = page.clone();
    let on_page_change = move |p: i64| {
        page_for_page.set(p);
        fetch_for_history(p);
    };

    let columns = vec![
        ColumnDef { header: "Timestamp", filterable: false, responsive_hide: None },
        ColumnDef { header: "Señal ID", filterable: true, responsive_hide: Some("hide-sm") },
        ColumnDef { header: "Valor", filterable: false, responsive_hide: None },
        ColumnDef { header: "Unidad", filterable: false, responsive_hide: Some("hide-sm") },
    ];

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
                        <DataTable
                            columns=columns.clone()
                            page=page
                            total_pages=cloned.total_pages
                            total=cloned.total
                            on_page_change=Arc::new(on_page_change)
                            on_filter=None
                        >
                            <tbody>
                                {cloned.data.into_iter().map(|r| view! {
                                    <tr>
                                        <td>{r.timestamp.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())}</td>
                                        <td class="hide-sm">{r.signal_id.map(|s| s.to_string())}</td>
                                        <td>{r.physical_value.unwrap_or_default()}</td>
                                        <td class="hide-sm">{r.unit.unwrap_or_default()}</td>
                                    </tr>
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </DataTable>
                    }.into_any()
                }
                _ => view! { <EmptyState message="No hay datos de telemetría" /> }.into_any()
            }
        }}
    }
}
