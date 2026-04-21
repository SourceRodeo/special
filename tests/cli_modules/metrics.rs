/**
@module SPECIAL.TESTS.CLI_MODULES.METRICS
`special arch --metrics` analysis and structured-output tests.
*/
// @fileimplements SPECIAL.TESTS.CLI_MODULES.METRICS
#[path = "metrics/complexity.rs"]
mod complexity;
#[path = "metrics/coupling.rs"]
mod coupling;
#[path = "metrics/dependencies.rs"]
mod dependencies;
#[path = "metrics/item_signals.rs"]
mod item_signals;
#[path = "metrics/language_packs.rs"]
mod language_packs;
#[path = "metrics/overview.rs"]
mod overview;
#[path = "metrics/quality.rs"]
mod quality;
