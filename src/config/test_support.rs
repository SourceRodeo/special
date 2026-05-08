/**
@module SPECIAL.CONFIG.TEST_SUPPORT
Shared test-only temp-directory helpers for config module fixtures.
*/
// @fileimplements SPECIAL.CONFIG.TEST_SUPPORT
use crate::test_support::TempProjectDir;

pub(super) fn temp_config_test_dir(prefix: &str) -> TempProjectDir {
    TempProjectDir::canonicalized(prefix)
}
