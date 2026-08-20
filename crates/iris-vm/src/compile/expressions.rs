use iris_syntax::Expression;

pub(super) fn ordinary_receiver_decline(
    receiver: &Expression,
    is_bound: impl FnOnce(&str) -> bool,
) -> Option<&'static str> {
    match receiver {
        Expression::Name(name) if !is_bound(name) => Some("call unbound receiver"),
        _ => None,
    }
}
