use iris_eval::Session;
use iris_runtime::Value;

fn main() -> Result<(), iris_eval::EvaluationError> {
    let mut session = Session::new()?;
    let result = session.evaluate(
        r#"
        class Receiver {
            public property count: Integer = 10
            public fun value() -> Integer { self.count = self.count + 1; self.count }
        }
        class Target {
            public fun relay(&block: Block<() -> Integer>) -> Integer { block.call() }
        }
        let bound = Receiver.new().value
        let callback = bound as Block<() -> Integer>;
        Target.new().relay(&callback)
    "#,
    )?;
    assert_eq!(result, Value::Integer(11u64.into()));
    assert_eq!(
        session.evaluate("bound.call()")?,
        Value::Integer(12u64.into())
    );
    assert_eq!(session.evaluate("(Block<() -> Integer>).type == (BoundMethod<() -> Integer> | Closure<() -> Integer>).type")?, Value::Bool(true));
    assert_eq!(
        session.evaluate("Target.new().relay() { || -> Object; 7 }"),
        Err(iris_eval::EvaluationError::TypeContractError)
    );
    println!("PASS: canonical Block union, retained bound receiver, rejected wrong signature");
    Ok(())
}
