use std::sync::Arc;
use leptos::prelude::*;

#[derive(Clone)]
pub struct ColumnDef {
    pub header: &'static str,
    pub filterable: bool,
    pub responsive_hide: Option<&'static str>,
}

#[component]
pub fn DataTable(
    columns: Vec<ColumnDef>,
    page: RwSignal<i64>,
    total_pages: i64,
    total: i64,
    on_page_change: Arc<dyn Fn(i64) + Send + Sync>,
    on_filter: Option<Arc<dyn Fn(String, String) + Send + Sync>>,
    children: Children,
) -> impl IntoView {
    let prev_disabled = move || page.get() <= 1;
    let next_disabled = move || page.get() >= total_pages;
    let page_info = move || format!("{} registros", total);

    let prev_click = {
        let opc = on_page_change.clone();
        move |_: leptos::ev::MouseEvent| {
            let p = (page.get() - 1).max(1);
            page.set(p);
            opc(p);
        }
    };

    let next_click = {
        let opc = on_page_change.clone();
        move |_: leptos::ev::MouseEvent| {
            let p = (page.get() + 1).min(total_pages);
            page.set(p);
            opc(p);
        }
    };

    let filter_signals: Vec<Option<RwSignal<String>>> = columns
        .iter()
        .map(|c| {
            if c.filterable {
                Some(RwSignal::new(String::new()))
            } else {
                None
            }
        })
        .collect();

    view! {
        <div class="table-container glass" style="padding: 0; overflow: hidden;">
            <table>
                <thead>
                    <tr>
                        {columns.iter().enumerate().map(|(i, col)| {
                            let class = col.responsive_hide.unwrap_or("");
                            let header_label = col.header;
                            let filter_cell = col.filterable.then(|| {
                                let signal = filter_signals[i].as_ref().unwrap();
                                let on_filter = on_filter.clone();
                                let hdr = header_label;
                                view! {
                                    <input
                                        class="table-filter-input"
                                        type="text"
                                        placeholder="Filtrar..."
                                        prop:value=move || signal.get()
                                        on:input=move |ev| {
                                            let val = event_target_value(&ev);
                                            signal.set(val.clone());
                                            if let Some(ref cb) = on_filter {
                                                cb(hdr.to_string(), val);
                                            }
                                        }
                                    />
                                }
                            });
                            view! {
                                <th class=class>
                                    <span>{header_label}</span>
                                    {filter_cell}
                                </th>
                            }
                        }).collect::<Vec<_>>()}
                    </tr>
                </thead>
                {children()}
            </table>
        </div>
        {move || (total_pages > 1).then(|| {
            view! {
                <div class="pagination">
                    <button
                        class="btn btn-sm"
                        disabled=prev_disabled
                        on:click=prev_click
                    >
                        "◀"
                    </button>
                    {move || {
                        let current = page.get();
                        if total_pages == 0 {
                            return vec![];
                        }
                        let start = (current - 2).max(1);
                        let end = (current + 2).min(total_pages);
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
        })}
    }
}

