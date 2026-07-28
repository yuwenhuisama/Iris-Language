//! Recursive-descent declarations plus Pratt expressions.

#[cfg(test)]
mod tests {
    #[test]
    fn crate_identity_is_available_when_compiled() {
        let crate_name = env!("CARGO_PKG_NAME");

        assert_ne!(crate_name, "");
    }
}
