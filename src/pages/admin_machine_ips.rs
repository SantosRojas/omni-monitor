use std::sync::Arc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::models::MachineIpWithSerial;
use crate::utils::api;
use crate::components::machine_ip_form::MachineIpForm;
use crate::components::loading_state::LoadingState;
use crate::components::empty_state::EmptyState;

#[component]
pub fn AdminMachineIpsPage() -> impl IntoView {
    let global_error = use_context::<RwSignal<Option<String>>>().expect("Global error context not provided");
    let items = RwSignal::new(Vec::<MachineIpWithSerial>::new());
    let loading = RwSignal::new(true);
    let show_form = RwSignal::new(false);
    let edit_item = RwSignal::new(Option::<MachineIpWithSerial>::None);

    let fetch = move || {
        loading.set(true);
        spawn_local(async move {
            match api::list_machine_ips().await {
                Ok(data) => items.set(data),
                Err(e) => global_error.set(Some(e)),
            }
            loading.set(false);
        });
    };
    fetch();

    let open_create = move || {
        edit_item.set(None);
        show_form.set(true);
    };

    let open_edit = move |item: MachineIpWithSerial| {
        edit_item.set(Some(item));
        show_form.set(true);
    };

    let close_form = move || {
        show_form.set(false);
        edit_item.set(None);
    };

    let on_saved = move || {
        close_form();
        fetch();
    };

    view! {
        <div class="page-title">IPs de Máquinas OMNI</div>

        <div class="action-bar">
            <button class="btn btn-primary" on:click=move |_| open_create()>"+ Nueva IP"</button>
        </div>

        {move || {
            if loading.get() {
                return view! { <LoadingState message="Cargando..." /> }.into_any();
            }
            let data = items.get();
            if data.is_empty() {
                return view! { <EmptyState message="No hay IPs registradas" /> }.into_any();
            }
            view! {
                <div class="table-container glass" style="padding:0;overflow:hidden;">
                    <table>
                        <thead>
                            <tr>
                                <th>ID</th>
                                <th>Máquina</th>
                                <th>IP</th>
                                <th>Puerto</th>
                                <th>Etiqueta</th>
                                <th>Activo</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {data.into_iter().map(|item| {
                                let item_clone = item.clone();
                                view! {
                                    <tr>
                                        <td>{item.id}</td>
                                        <td>{item.serial_number.clone().unwrap_or_default()}</td>
                                        <td>{item.ip_address}</td>
                                        <td>{item.port.map(|p| p.to_string()).unwrap_or_default()}</td>
                                        <td>{item.label.clone().unwrap_or_default()}</td>
                                        <td>
                                            <span class={if item.is_active { "badge badge-active" } else { "badge badge-inactive" }}>
                                                {if item.is_active { "Activo" } else { "Inactivo" }}
                                            </span>
                                        </td>
                                        <td>
                                            <button class="btn btn-sm" on:click=move |_| open_edit(item_clone.clone())>"Editar"</button>
                                        </td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()}
                        </tbody>
                    </table>
                </div>
            }.into_any()
        }}

        {move || show_form.get().then(|| {
            view! {
                <MachineIpForm
                    edit_item=edit_item.get()
                    on_close=Arc::new(close_form)
                    on_saved=Arc::new(on_saved)
                />
            }
        })}
    }
}
