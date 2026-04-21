/**
@module SPECIAL.RENDER.COMMON
Provides backend-agnostic render helpers shared by text and HTML output.
*/
// @fileimplements SPECIAL.RENDER.COMMON
pub(super) fn planned_badge_text(planned_release: Option<&str>) -> String {
    match planned_release {
        Some(release) => format!("planned: {release}"),
        None => "planned".to_string(),
    }
}

pub(super) fn deprecated_badge_text(deprecated_release: Option<&str>) -> String {
    match deprecated_release {
        Some(release) => format!("deprecated: {release}"),
        None => "deprecated".to_string(),
    }
}
