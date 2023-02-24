use {crate::tests_suite::*, solana_program_test::tokio};

pub mod instructions;
pub mod tests_suite;
pub mod utils;

#[tokio::test]
async fn test_integration() {
    basic_interactions_test_suite().await;

    // add new test suite here ...
}