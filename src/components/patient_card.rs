#[cfg(feature = "frontend")]
use leptos::prelude::*;
#[cfg(feature = "frontend")]
use leptos_router::components::A;
use crate::models::Patient;

#[component]
pub fn PatientCard(patient: Patient) -> impl IntoView {
    view! {
        <A href={format!("/patients/{}", patient.id)} attr:class="card glass" attr:style="display: block; text-decoration: none;">
            <div class="flex justify-between items-center">
                <div>
                    <div style="font-weight: 600; font-size: 1.05rem; color: var(--text-primary);">
                        {patient.patient_id_str}
                    </div>
                    <div style="font-size: 0.8rem; color: var(--text-muted); margin-top: 4px;">
                        {patient.created_at.map(|t| t.format("%Y-%m-%d %H:%M").to_string())}
                    </div>
                </div>
                <div style="text-align: right;">
                    {patient.therapy_start.map(|t| view! { <div style="font-size:0.8rem;color:var(--success);">"Inicio: " {t.format("%Y-%m-%d").to_string()}</div> })}
                    {patient.therapy_end.map(|t| view! { <div style="font-size:0.8rem;color:var(--text-muted);">"Fin: " {t.format("%Y-%m-%d").to_string()}</div> })}
                </div>
            </div>
        </A>
    }
}
