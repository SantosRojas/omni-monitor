pub fn trigger_download(patient_id: i64) {
    let url = format!("/api/patients/{}/export", patient_id);
    let window = web_sys::window().unwrap();
    let _ = window.location().assign(&url);
}

pub fn trigger_therapy_download(therapy_id: i64) {
    let url = format!("/api/therapies/{}/export", therapy_id);
    let window = web_sys::window().unwrap();
    let _ = window.location().assign(&url);
}
