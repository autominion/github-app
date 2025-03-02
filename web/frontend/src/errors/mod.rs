use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::NavigateOptions;

use crate::{api::ApiError, routes::paths};

mod error_modal;

pub use error_modal::ErrorModal;

#[derive(Default, Clone, PartialEq)]
pub struct ErrorStore {
    pub error: Option<ApiError>,
}

pub fn handle_api_result<T>(
    result: Result<T, ApiError>,
    navigate: Arc<impl Fn(&str, NavigateOptions)>,
    error_store: &RwSignal<ErrorStore>,
) -> Result<T, ApiError> {
    match result {
        Ok(value) => Ok(value),
        Err(ApiError::Unauthorized) => {
            navigate(paths::LOGIN, Default::default());
            error_store.update(|store| store.error = Some(ApiError::Unauthorized));
            Err(ApiError::Unauthorized)
        }
        Err(err) => {
            error_store.update(|store| store.error = Some(err.clone()));
            Err(err)
        }
    }
}
