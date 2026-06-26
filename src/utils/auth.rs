use crate::models::UserResponse;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[derive(Clone)]
pub struct AuthContext {
    pub token: RwSignal<Option<String>>,
    pub user: RwSignal<Option<UserResponse>>,
}

impl AuthContext {
    pub fn new() -> Self {
        let ctx = Self {
            token: RwSignal::new(None),
            user: RwSignal::new(None),
        };
        ctx.restore_session();
        ctx
    }

    fn restore_session(&self) {
        let this = self.clone();
        spawn_local(async move {
            if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                if let Ok(Some(token)) = storage.get_item("monitor_token") {
                    this.token.set(Some(token.clone()));
                    if let Ok(Some(user_json)) = storage.get_item("monitor_user") {
                        if let Ok(user) = serde_json::from_str::<UserResponse>(&user_json) {
                            this.user.set(Some(user));
                            return;
                        }
                    }
                    let user = crate::utils::api::get_me(Some(token.clone())).await;
                    if let Ok(u) = user {
                        let u_clone = u.clone();
                        this.user.set(Some(u));
                        let _ = storage.set_item("monitor_user", &serde_json::to_string(&u_clone).unwrap_or_default());
                    } else {
                        this.token.set(None);
                        this.user.set(None);
                        let _ = storage.remove_item("monitor_token");
                        let _ = storage.remove_item("monitor_user");
                    }
                }
            }
        });
    }

    pub fn login(&self, token: &str, user: UserResponse) {
        self.token.set(Some(token.to_string()));
        self.user.set(Some(user.clone()));
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item("monitor_token", token);
            let _ = storage.set_item("monitor_user", &serde_json::to_string(&user).unwrap_or_default());
        }
    }

    pub fn logout(&self) {
        self.token.set(None);
        self.user.set(None);
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.remove_item("monitor_token");
            let _ = storage.remove_item("monitor_user");
        }
    }

    pub fn is_admin(&self) -> bool {
        self.user.get().map(|u| u.role == "admin").unwrap_or(false)
    }

    pub fn is_logged_in(&self) -> bool {
        self.token.get().is_some()
    }
}

pub fn use_auth() -> AuthContext {
    expect_context::<AuthContext>()
}
