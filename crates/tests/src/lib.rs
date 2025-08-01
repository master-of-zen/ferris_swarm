pub mod unit;
pub mod integration;
pub mod performance;

// Test utilities and common setup
pub mod common;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_suite_smoke_test() {
        // Ensure the test crate itself compiles and basic dependencies work
        assert!(true);
    }
}