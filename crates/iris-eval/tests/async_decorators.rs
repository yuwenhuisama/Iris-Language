use iris_eval::{Session, evaluate};

#[test]
fn ready_wrapper_runs_eagerly_when_inner_task_is_complete() {
    let mut given = Session::new().unwrap();
    given
        .evaluate(include_str!(
            "../../iris-cli/tests/decorator_async/eager_ready.iris"
        ))
        .unwrap();
    let when = given.evaluate("%[Effects.prefix, Effects.body, Effects.suffix]");
    assert_eq!(when, evaluate("%[1, 1, 1]"));
}

#[test]
fn owner_retains_next_when_callback_resumes_from_gate() {
    let mut given = Session::new().unwrap();
    given
        .evaluate(include_str!(
            "../../iris-cli/tests/decorator_async/pending_owner.iris"
        ))
        .unwrap();
    let when = given.evaluate("%[Effects.prefix, Effects.resumed, Effects.body]");
    assert_eq!(when, evaluate("%[1, 1, 1]"));
}
