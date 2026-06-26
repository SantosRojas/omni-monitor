use crate::models::*;
use crate::utils::auth::AuthContext;
use leptos::prelude::*;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsCast;

pub fn get_token() -> Option<String> {
    // First try the reactive context (works inside component bodies)
    if let Some(token) = use_context::<AuthContext>().and_then(|a| a.token.get_untracked()) {
        return Some(token);
    }
    // Fallback: read directly from localStorage (works inside spawn_local async blocks
    // where the Leptos reactive owner context is not available)
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("monitor_token").ok().flatten())
}

fn get_base() -> String {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return "/api".to_string(),
    };
    let location = window.location();
    let port = location.port().ok().unwrap_or_default();
    format!("http://{}:{}/api", location.hostname().unwrap_or_else(|_| "localhost".to_string()), if port.is_empty() { "9002".to_string() } else { port })
}

async fn request<T: DeserializeOwned>(method: &str, path: &str, body: Option<&str>, token: Option<String>) -> Result<T, String> {
    let url = format!("{}{}", get_base(), path);

    let opts = web_sys::RequestInit::new();
    opts.set_method(method);
    if let Some(b) = body {
        opts.set_body(&wasm_bindgen::JsValue::from_str(b));
    }

    let headers = web_sys::Headers::new().map_err(|_| "Failed to create headers".to_string())?;
    headers.set("Content-Type", "application/json").map_err(|_| "".to_string())?;
    if let Some(t) = &token {
        headers.set("Authorization", &format!("Bearer {}", t)).map_err(|_| "".to_string())?;
    }
    opts.set_headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    let window = web_sys::window().ok_or("No window".to_string())?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: web_sys::Response = resp_value.dyn_into().map_err(|_| "Not a response".to_string())?;
    if !resp.ok() {
        let status = resp.status();
        let text_val = wasm_bindgen_futures::JsFuture::from(resp.text().map_err(|_| "No text".to_string())?)
            .await
            .map_err(|e| format!("Error reading response: {:?}", e))?;
        let text_val: String = text_val.as_string().unwrap_or_default();
        return Err(format!("HTTP {}: {}", status, text_val));
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON".to_string())?)
        .await
        .map_err(|e| format!("JSON parse error: {:?}", e))?;

    serde_wasm_bindgen::from_value::<T>(json).map_err(|e| format!("Deserialize error: {}", e))
}

pub async fn api_get<T: DeserializeOwned>(path: &str, token: Option<String>) -> Result<T, String> {
    request("GET", path, None, token).await
}

pub async fn api_post<T: DeserializeOwned>(path: &str, body: &str, token: Option<String>) -> Result<T, String> {
    request("POST", path, Some(body), token).await
}

pub async fn api_put<T: DeserializeOwned>(path: &str, body: &str, token: Option<String>) -> Result<T, String> {
    request("PUT", path, Some(body), token).await
}

pub async fn api_delete<T: DeserializeOwned>(path: &str, token: Option<String>) -> Result<T, String> {
    request("DELETE", path, None, token).await
}

// --- Convenience functions ---

pub async fn login(username: &str, password: &str) -> Result<LoginResponse, String> {
    let body = serde_json::json!({"username": username, "password": password}).to_string();
    api_post("/auth/login", &body, None).await
}

pub async fn get_me(token: Option<String>) -> Result<UserResponse, String> {
    api_get("/auth/me", token).await
}

pub async fn list_patients(page: i64, per_page: i64, search: Option<&str>) -> Result<PaginatedResponse<Patient>, String> {
    let mut path = format!("/patients?page={}&per_page={}", page, per_page);
    if let Some(s) = search {
        path.push_str(&format!("&search={}", urlencode(s)));
    }
    api_get(&path, get_token()).await
}

pub async fn get_patient(id: i64) -> Result<Patient, String> {
    api_get(&format!("/patients/{}", id), get_token()).await
}

pub async fn get_patient_therapies(id: i64) -> Result<Vec<TherapyWithMachine>, String> {
    api_get(&format!("/patients/{}/therapies", id), get_token()).await
}

pub async fn get_patient_history(id: i64, page: i64, per_page: i64) -> Result<PaginatedResponse<TelemetryReading>, String> {
    api_get(&format!("/patients/{}/history?page={}&per_page={}", id, page, per_page), get_token()).await
}

pub async fn get_active_device(id: i64) -> Result<ActiveDevice, String> {
    api_get(&format!("/patients/{}/active-device", id), get_token()).await
}

pub async fn get_therapy_dashboard(id: i64) -> Result<PatientDashboard, String> {
    api_get(&format!("/therapies/{}/dashboard", id), get_token()).await
}

pub async fn get_patient_dashboard(id: i64, signal_ids: Option<&str>, from: Option<&str>, to: Option<&str>) -> Result<PatientDashboard, String> {
    let mut path = format!("/patients/{}/dashboard", id);
    let mut params = Vec::new();
    if let Some(s) = signal_ids { params.push(format!("signal_ids={}", urlencode(s))); }
    if let Some(f) = from { params.push(format!("from={}", urlencode(f))); }
    if let Some(t) = to { params.push(format!("to={}", urlencode(t))); }
    if !params.is_empty() { path.push_str(&format!("?{}", params.join("&"))); }
    api_get(&path, get_token()).await
}

pub async fn list_machines() -> Result<Vec<Machine>, String> {
    api_get("/machines", get_token()).await
}

pub async fn list_machine_ips() -> Result<Vec<MachineIpWithSerial>, String> {
    api_get("/admin/machine-ips", get_token()).await
}

pub async fn create_machine_ip(req: &CreateMachineIpRequest) -> Result<MachineIp, String> {
    let body = serde_json::to_string(req).unwrap_or_default();
    api_post("/admin/machine-ips", &body, get_token()).await
}

pub async fn update_machine_ip(id: i64, req: &UpdateMachineIpRequest) -> Result<MachineIp, String> {
    let body = serde_json::to_string(req).unwrap_or_default();
    api_put(&format!("/admin/machine-ips/{}", id), &body, get_token()).await
}

pub async fn delete_machine_ip(id: i64) -> Result<serde_json::Value, String> {
    api_delete(&format!("/admin/machine-ips/{}", id), get_token()).await
}

pub async fn list_users() -> Result<Vec<UserResponse>, String> {
    api_get("/users", get_token()).await
}

pub async fn create_user(req: &CreateUserRequest) -> Result<UserResponse, String> {
    let body = serde_json::to_string(req).unwrap_or_default();
    api_post("/users", &body, get_token()).await
}

pub async fn update_user(id: i64, req: &UpdateUserRequest) -> Result<UserResponse, String> {
    let body = serde_json::to_string(req).unwrap_or_default();
    api_put(&format!("/users/{}", id), &body, get_token()).await
}

pub async fn delete_user(id: i64) -> Result<serde_json::Value, String> {
    api_delete(&format!("/users/{}", id), get_token()).await
}

fn urlencode(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('#', "%23")
        .replace('&', "%26")
}