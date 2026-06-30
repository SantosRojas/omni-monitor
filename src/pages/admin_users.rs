use std::sync::Arc;
use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::models::{UserResponse, CreateUserRequest, UpdateUserRequest};
use crate::utils::api;
use crate::components::table::{DataTable, ColumnDef};
use crate::components::loading_state::LoadingState;
use crate::components::empty_state::EmptyState;
use crate::components::modal::Modal;

#[component]
pub fn AdminUsersPage() -> impl IntoView {
    let global_error = use_context::<RwSignal<Option<String>>>().expect("Global error context not provided");
    let users = RwSignal::new(Vec::<UserResponse>::new());
    let loading = RwSignal::new(true);
    let show_create = RwSignal::new(false);
    let editing_id = RwSignal::new(Option::<i64>::None);

    let fetch = move || {
        loading.set(true);
        spawn_local(async move {
            match api::list_users().await {
                Ok(data) => users.set(data),
                Err(e) => global_error.set(Some(e)),
            }
            loading.set(false);
        });
    };
    fetch();

    let columns = vec![
        ColumnDef { header: "ID", filterable: false, responsive_hide: Some("hide-sm") },
        ColumnDef { header: "Usuario", filterable: true, responsive_hide: None },
        ColumnDef { header: "Nombre", filterable: true, responsive_hide: Some("hide-md") },
        ColumnDef { header: "Email", filterable: true, responsive_hide: Some("hide-md") },
        ColumnDef { header: "Rol", filterable: false, responsive_hide: None },
        ColumnDef { header: "Activo", filterable: false, responsive_hide: None },
        ColumnDef { header: "", filterable: false, responsive_hide: None },
    ];

    view! {
        <div class="page-title">Usuarios</div>

        <div class="action-bar">
            <button class="btn btn-primary" on:click=move |_| show_create.set(true)>"+ Nuevo Usuario"</button>
        </div>

        {move || {
            if loading.get() {
                return view! { <LoadingState message="Cargando..." /> }.into_any();
            }
            let data = users.get();
            if data.is_empty() {
                return view! { <EmptyState message="No hay usuarios" /> }.into_any();
            }
            let n = data.len() as i64;
            view! {
                <DataTable
                    columns=columns.clone()
                    page=RwSignal::new(1)
                    total_pages=1
                    total=n
                    on_page_change=Arc::new(|_| {})
                    on_filter=None
                >
                    <tbody>
                        {data.into_iter().map(|u| {
                            let uid = u.id;
                            view! {
                                <tr>
                                    <td class="hide-sm">{u.id}</td>
                                    <td style="font-weight:500;color:var(--text-primary);">{u.username}</td>
                                    <td class="hide-md">{u.full_name}</td>
                                    <td class="hide-md">{u.email}</td>
                                    <td>
                                        <span class={format!("badge badge-{}", u.role)}>{u.role.clone()}</span>
                                    </td>
                                    <td>
                                        <span class={if u.active { "badge badge-active" } else { "badge badge-inactive" }}>
                                            {if u.active { "Activo" } else { "Inactivo" }}
                                        </span>
                                    </td>
                                    <td>
                                        <button class="btn btn-sm" on:click=move |_| editing_id.set(Some(uid))>"Editar"</button>
                                    </td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()}
                    </tbody>
                </DataTable>
            }.into_any()
        }}

        {move || {
            let show = show_create.get();
            let edit = editing_id.get();
            (show || edit.is_some()).then(|| {
                view! {
                    <UserFormModal
                        edit_id=edit
                        on_close=Arc::new(move || { show_create.set(false); editing_id.set(None); })
                        on_saved=Arc::new(move || { show_create.set(false); editing_id.set(None); fetch(); })
                    />
                }
            })
        }}
    }
}

#[component]
fn UserFormModal(
    edit_id: Option<i64>,
    on_close: Arc<dyn Fn() + Send + Sync>,
    on_saved: Arc<dyn Fn() + Send + Sync>,
) -> impl IntoView {
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let full_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let role = RwSignal::new("viewer".to_string());
    let saving = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let is_edit = edit_id.is_some();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        saving.set(true);
        error.set(None);

        let u = username.get();
        let p = password.get();
        let n = full_name.get();
        let e = email.get();
        let r = role.get();
        let eid = edit_id;
        let on_saved = on_saved.clone();

        spawn_local(async move {
            if let Some(eid) = eid {
                let req = UpdateUserRequest {
                    password: if p.is_empty() { None } else { Some(p) },
                    full_name: Some(n),
                    email: Some(e),
                    role: Some(r),
                    active: None,
                };
                match api::update_user(eid, &req).await {
                    Ok(_) => on_saved(),
                    Err(e) => { error.set(Some(e)); saving.set(false); }
                }
            } else {
                let req = CreateUserRequest { username: u, password: p, full_name: n, email: e, role: r };
                match api::create_user(&req).await {
                    Ok(_) => on_saved(),
                    Err(e) => { error.set(Some(e)); saving.set(false); }
                }
            }
        });
    };

    let on_close_btn = on_close.clone();
    let title = if is_edit { "Editar Usuario".to_string() } else { "Nuevo Usuario".to_string() };
    view! {
        <Modal title=title on_close=on_close>
            {move || error.get().map(|e| view! { <div class="login-error">{e}</div> })}
            <form on:submit=on_submit>
                <div class="form-group">
                    <label class="form-label">Usuario</label>
                    <input class="input" type="text" prop:value=move || username.get()
                        on:input=move |ev| username.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label class="form-label">{if is_edit { "Nueva Contraseña (dejar vacío para mantener)" } else { "Contraseña" }}</label>
                    <input class="input" type="password" prop:value=move || password.get()
                        on:input=move |ev| password.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label class="form-label">Nombre Completo</label>
                    <input class="input" type="text" prop:value=move || full_name.get()
                        on:input=move |ev| full_name.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label class="form-label">Email</label>
                    <input class="input" type="email" prop:value=move || email.get()
                        on:input=move |ev| email.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label class="form-label">Rol</label>
                    <select class="select" prop:value=move || role.get()
                        on:change=move |ev| role.set(event_target_value(&ev))>
                        <option value="admin">Admin</option>
                        <option value="operator">Operator</option>
                        <option value="viewer">Viewer</option>
                    </select>
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
