use std::sync::Arc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use crate::models::{Patient, PaginatedResponse};
use crate::utils::api;
use crate::components::table::{DataTable, ColumnDef};
use crate::components::loading_state::LoadingState;
use crate::components::empty_state::EmptyState;

#[component]
pub fn PatientsPage() -> impl IntoView {
    let global_error = use_context::<RwSignal<Option<String>>>().expect("Global error context not provided");
    let page = RwSignal::new(1);
    let search = RwSignal::new(String::new());
    let data = RwSignal::new(Option::<PaginatedResponse<Patient>>::None);
    let loading = RwSignal::new(true);

    let fetch_loading = loading.clone();
    let fetch_search = search.clone();
    let fetch_data = data.clone();
    let fetch_error = global_error.clone();

    let fetch = Arc::new(move |p: i64| {
        fetch_loading.set(true);
        let s = fetch_search.get_untracked();
        let d = fetch_data.clone();
        let e = fetch_error.clone();
        let l = fetch_loading.clone();
        spawn_local(async move {
            let result = api::list_patients(p, 20, if s.is_empty() { None } else { Some(&s) }).await;
            match result {
                Ok(resp) => { d.set(Some(resp)); }
                Err(e) => { e.set(Some(e)); }
            }
            l.set(false);
        });
    });

    fetch(1);

    let fetch_for_search = fetch.clone();
    let page_for_search = page.clone();
    let on_search = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        page_for_search.set(1);
        fetch_for_search(1);
    };

    let fetch_for_page = fetch;
    let page_for_page = page.clone();
    let on_page_change = move |p: i64| {
        page_for_page.set(p);
        fetch_for_page(p);
    };

    let search_for_input = search.clone();
    let on_search_input = move |ev: leptos::ev::Event| {
        search_for_input.set(event_target_value(&ev));
    };

    let columns = vec![
        ColumnDef { header: "ID", filterable: false, responsive_hide: Some("hide-sm") },
        ColumnDef { header: "Identificador", filterable: true, responsive_hide: None },
        ColumnDef { header: "Registrado", filterable: false, responsive_hide: Some("hide-md") },
        ColumnDef { header: "Estado Terapia", filterable: false, responsive_hide: None },
        ColumnDef { header: "", filterable: false, responsive_hide: None },
    ];

    view! {
        <div class="page-title">Pacientes</div>

        <form on:submit=on_search class="search-bar">
            <input
                class="input"
                type="text"
                placeholder="Buscar por identificador..."
                prop:value=move || search.get()
                on:input=on_search_input
            />
            <button type="submit" class="btn btn-primary">Buscar</button>
        </form>

        {move || {
            if loading.get() {
                view! { <LoadingState message="Cargando pacientes..." /> }.into_any()
            } else if let Some(ref d) = data.get() {
                if d.data.is_empty() {
                    view! { <EmptyState message="No se encontraron pacientes" /> }.into_any()
                } else {
                    let cloned = d.clone();
                    view! {
                        <DataTable
                            columns=columns.clone()
                            page=page.clone()
                            total_pages=cloned.total_pages
                            total=cloned.total
                            on_page_change=Arc::new(on_page_change)
                            on_filter=None
                        >
                            <tbody>
                                {cloned.data.into_iter().map(|p| {
                                    let is_active = p.active_therapy_count.unwrap_or(0) > 0;
                                    view! {
                                    <tr>
                                        <td class="hide-sm">{p.id}</td>
                                        <td style="font-weight: 500; color: var(--text-primary);">{p.patient_id_str}</td>
                                        <td class="hide-md">{p.created_at.map(|t| t.format("%Y-%m-%d %H:%M").to_string())}</td>
                                        <td>
                                            <span class={format!("badge badge-{}", if is_active { "active" } else { "inactive" })}>
                                                {if is_active { "Activa" } else { "Inactiva" }}
                                            </span>
                                        </td>
                                        <td>
                                            <A href={format!("/patients/{}", p.id)} attr:class="btn btn-sm btn-primary">"Ver"</A>
                                        </td>
                                    </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </DataTable>
                    }.into_any()
                }
            } else {
                view! { <EmptyState message="Error al cargar pacientes" /> }.into_any()
            }
        }}
    }
}
