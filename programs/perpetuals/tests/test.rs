use solana_program_test::tokio;

use crate::tests_suite::basic_test_suite;

pub mod instructions;
pub mod tests_suite;
pub mod utils;

#[tokio::test]
async fn test_integration() {
    basic_test_suite().await;

    // add new test suite here ...
}
