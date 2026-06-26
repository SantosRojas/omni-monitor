use std::sync::Arc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::models::{CreateMachineIpRequest, UpdateMachineIpRequest, MachineIpWithSerial, Machine};
use crate::utils::api;
use crate::components::modal::Modal;

#[component]
pub fn MachineIpForm(
    edit_item: Option<MachineIpWithSerial>,
    on_close: Arc<dyn Fn() + Send + Sync>,
    on_saved: Arc<dyn Fn() + Send + Sync>,
) -> impl IntoView {
    let machines = RwSignal::new(Vec::<Machine>::new());
    let machine_id = RwSignal::new(edit_item.as_ref().map(|i| i.machine_id).unwrap_or(0));
    let ip_address = RwSignal::new(edit_item.as_ref().map(|i| i.ip_address.clone()).unwrap_or_default());
    let port = RwSignal::new(edit_item.as_ref().and_then(|i| i.port).map(|p| p.to_string()).unwrap_or_else(|| "9001".to_string()));
    let label = RwSignal::new(edit_item.as_ref().and_then(|i| i.label.clone()).unwrap_or_default());
    let saving = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let is_edit = edit_item.is_some();
    let title = if is_edit { "Editar IP de Máquina" } else { "Nueva IP de Máquina" };

    {
        spawn_local(async move {
            match api::list_machines().await {
                Ok(list) => machines.set(list),
                Err(e) => error.set(Some(e)),
            }
        });
    }

    let on_save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        saving.set(true);
        error.set(None);

        let mid = machine_id.get();
        let ip = ip_address.get();
        let p: i32 = port.get().parse().unwrap_or(9001);
        let lbl = label.get();

        if mid == 0 {
            error.set(Some("Seleccione una máquina".to_string()));
            saving.set(false);
            return;
        }

        if is_edit {
            let item_id = edit_item.as_ref().map(|i| i.id).unwrap_or(0);
            let on_saved = on_saved.clone();
            spawn_local(async move {
                let result = api::update_machine_ip(item_id, &UpdateMachineIpRequest {
                    ip_address: Some(ip),
                    port: Some(p),
                    label: Some(lbl),
                    is_active: None,
                }).await;
                match result {
                    Ok(_) => { on_saved(); }
                    Err(e) => { error.set(Some(e)); saving.set(false); }
                }
            });
        } else {
            let on_saved = on_saved.clone();
            spawn_local(async move {
                let result = api::create_machine_ip(&CreateMachineIpRequest {
                    machine_id: mid,
                    ip_address: ip,
                    port: Some(p),
                    label: Some(lbl),
                }).await;
                match result {
                    Ok(_) => { on_saved(); }
                    Err(e) => { error.set(Some(e)); saving.set(false); }
                }
            });
        }
    };

    let on_close_btn = on_close.clone();
    view! {
        <Modal title=title.to_string() on_close=on_close>
            {move || error.get().map(|e| view! { <div class="login-error">{e}</div> })}
            <form on:submit=on_save>
                <div class="form-group">
                    <label class="form-label">Máquina</label>
                    <select class="select"
                        prop:value=move || machine_id.get().to_string()
                        on:change=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<i64>() {
                                machine_id.set(v);
                            }
                        }
                    >
                        <option value="0">Seleccione una máquina...</option>
                        {move || machines.get().into_iter().map(|m| {
                            let sid = m.id;
                            view! {
                                <option value={sid.to_string()} selected=move || machine_id.get() == sid>
                                    {m.serial_number}
                                </option>
                            }
                        }).collect::<Vec<_>>()}
                    </select>
                </div>
                <div class="form-group">
                    <label class="form-label">Dirección IP</label>
                    <input class="input" type="text" placeholder="192.168.1.100"
                        prop:value=move || ip_address.get()
                        on:input=move |ev| ip_address.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label class="form-label">Puerto</label>
                    <input class="input" type="number" placeholder="9001"
                        prop:value=move || port.get()
                        on:input=move |ev| port.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label class="form-label">Etiqueta</label>
                    <input class="input" type="text" placeholder="Máquina principal"
                        prop:value=move || label.get()
                        on:input=move |ev| label.set(event_target_value(&ev))
                    />
                </div>
                <div class="modal-actions">
                    <button type="button" class="btn" on:click=move |_| on_close_btn()>Cancelar</button>
                    <button type="submit" class="btn btn-primary" disabled=move || saving.get()>
                        {if is_edit { "Guardar" } else { "Crear" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}
