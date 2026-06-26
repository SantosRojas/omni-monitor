use std::sync::Arc;
use leptos::prelude::*;
use leptos_router::components::A;
use crate::models::{Patient, PaginatedResponse};

#[component]
pub fn PatientTable(
    data: PaginatedResponse<Patient>,
    page: RwSignal<i64>,
    on_page_change: Arc<dyn Fn(i64) + Send + Sync>,
) -> impl IntoView {
    let prev_disabled = move || page.get() <= 1;
    let next_disabled = move || page.get() >= data.total_pages;
    let page_info = move || format!("{} registros", data.total);
    let prev_click = {
        let opc = on_page_change.clone();
        move |_: leptos::ev::MouseEvent| { let p = page.get() - 1; page.set(p); opc(p); }
    };
    let next_click = {
        let opc = on_page_change.clone();
        move |_: leptos::ev::MouseEvent| { let p = page.get() + 1; page.set(p); opc(p); }
    };

    view! {
        <div class="table-container glass" style="padding: 0; overflow: hidden;">
            <table>
                <thead>
                    <tr>
                        <th>ID</th>
                        <th>Identificador</th>
                        <th>Registrado</th>
                        <th>Inicio Terapia</th>
                        <th>Fin Terapia</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {data.data.into_iter().map(|p| view! {
                        <tr>
                            <td>{p.id}</td>
                            <td style="font-weight: 500; color: var(--text-primary);">{p.patient_id_str}</td>
                            <td>{p.created_at.map(|t| t.format("%Y-%m-%d %H:%M").to_string())}</td>
                            <td>{p.therapy_start.map(|t| t.format("%Y-%m-%d %H:%M").to_string())}</td>
                            <td>{p.therapy_end.map(|t| t.format("%Y-%m-%d %H:%M").to_string())}</td>
                            <td>
                                <A href={format!("/patients/{}", p.id)} attr:class="btn btn-sm btn-primary">"Ver"</A>
                            </td>
                        </tr>
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
        <div class="pagination">
            <button
                class="btn btn-sm"
                disabled=prev_disabled
                on:click=prev_click
            >
                "◀"
            </button>
            {move || {
                let total = data.total_pages;
                let current = page.get();
                let start = (current - 2).max(1);
                let end = (current + 2).min(total);
                let opc = on_page_change.clone();
                (start..=end).map(move |i| {
                    let opc = opc.clone();
                    view! {
                        <button
                            class=move || format!("btn btn-sm {}", if i == current { "active" } else { "" })
                            on:click=move |_: leptos::ev::MouseEvent| { page.set(i); opc(i); }
                        >
                            {i}
                        </button>
                    }
                }).collect::<Vec<_>>()
            }}
            <button
                class="btn btn-sm"
                disabled=next_disabled
                on:click=next_click
            >
                "▶"
            </button>
            <span style="font-size:0.8rem;color:var(--text-muted);margin-left:8px;">
                {page_info}
            </span>
        </div>
    }
}
