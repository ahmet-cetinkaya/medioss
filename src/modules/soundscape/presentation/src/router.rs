pub use crate::dependencies::{SharedDependencies, SoundscapeDependencies};
use axum::routing::{get, put};
use axum::Router;
use soundscape_application::commands;

use crate::categories::{create_category, delete_category, list_categories, rename_category};
use crate::icons::{clear_icon, get_icon, upload_icon};
use crate::mixes::{create_mix, delete_mix, get_mix, list_mixes, update_mix};
use crate::sounds::{
    delete_sound, download_sound, list_sounds, stream_sound, update_sound, upload_sound,
};

pub fn soundscape_router() -> Router<SharedDependencies> {
    Router::new()
        .route(
            "/soundscape/categories",
            get(list_categories).post(create_category),
        )
        .route(
            "/soundscape/categories/{id}",
            put(rename_category).delete(delete_category),
        )
        .route(
            "/soundscape/sounds",
            get(list_sounds)
                .post(upload_sound)
                .layer(axum::extract::DefaultBodyLimit::max(
                    commands::sounds::MAX_UPLOAD_BYTES,
                )),
        )
        .route(
            "/soundscape/sounds/{id}",
            put(update_sound).delete(delete_sound),
        )
        .route(
            "/soundscape/sounds/{id}/icon",
            get(get_icon).put(upload_icon).delete(clear_icon).layer(
                axum::extract::DefaultBodyLimit::max(2 * soundscape_domain::MAX_ICON_BYTES),
            ),
        )
        .route("/soundscape/sounds/{id}/audio", get(stream_sound))
        .route("/soundscape/sounds/{id}/download", get(download_sound))
        .route("/soundscape/mixes", get(list_mixes).post(create_mix))
        .route(
            "/soundscape/mixes/{id}",
            get(get_mix).put(update_mix).delete(delete_mix),
        )
}
