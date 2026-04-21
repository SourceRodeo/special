/**
@module SPECIAL.TESTS.SUPPORT.CLI.ARCHITECTURE
Architecture and module-analysis fixture facade for CLI integration tests.
*/
// @fileimplements SPECIAL.TESTS.SUPPORT.CLI.ARCHITECTURE
#[path = "architecture/analysis.rs"]
mod analysis;
#[path = "architecture/declarations.rs"]
mod declarations;

pub use analysis::*;
pub use declarations::*;
