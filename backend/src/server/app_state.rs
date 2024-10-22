use crate::domain::logic::user_mgr::UserMgr;
use crate::{domain::model::user::User, repo::user_repo::UserRepo};
use axum::extract::State;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    // pub dbcp: Arc<DbConnPool>,
    // pub auth_mgr: AuthMgr,
    user_repo: Arc<Mutex<UserRepo>>,
}

impl AppState {
    //
    pub fn new(/*dbcp: DbConnPool*/) -> Self {
        // let dbcp = Arc::new(dbcp);
        // let auth_mgr = AuthMgr::new(user_repo.clone());
        let user_repo = Arc::new(Mutex::new(UserRepo::new(/*dbcp.clone()*/)));

        Self { user_repo }
    }

    pub fn get_user_mgr(self) -> UserMgr {
        UserMgr::new(self.user_repo)
    }
}
