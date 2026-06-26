#[cfg(feature = "frontend")]
use std::sync::Arc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::models::{Patient, PaginatedResponse};
use crate::utils::api;
use crate::components::patient_table::PatientTable;
use crate::components::loading_state::LoadingState;
use crate::components::empty_state::EmptyState;

#[component]
pub fn PatientsPage() -> impl IntoView {
    let global_error = use_context::<RwSignal<Option<String>>>().expect("Global error context not provided");
    let page = RwSignal::new(1);
    let search = RwSignal::new(String::new());
    let data = RwSignal::new(Option::<PaginatedResponse<Patient>>::None);
    let loading = RwSignal::new(true);

    let fetch = move |p: i64| {
        loading.set(true);
        let s = search.get_untracked();
        spawn_local(async move {
            let result = api::list_patients(p, 20, if s.is_empty() { None } else { Some(&s) }).await;
            match result {
                Ok(resp) => { data.set(Some(resp)); }
                Err(e) => { global_error.set(Some(e)); }
            }
            loading.set(false);
        });
    };

    fetch(1);

    let on_search = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        page.set(1);
        fetch(1);
    };

    let on_page_change = move |p: i64| {
        page.set(p);
        fetch(p);
    };

    let on_search_input = move |ev: leptos::ev::Event| {
        search.set(event_target_value(&ev));
    };

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
                    view! {
                        <PatientTable
                            data=d.clone()
                            page=page
                            on_page_change=Arc::new(on_page_change)
                        />
                    }.into_any()
                }
            } else {
                view! { <EmptyState message="Error al cargar pacientes" /> }.into_any()
            }
        }}
    }
}
