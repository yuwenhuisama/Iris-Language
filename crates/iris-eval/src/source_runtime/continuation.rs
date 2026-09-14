use super::wrapper_execution::LexicalContext;
use super::{Binding, EvaluationError, Expression, SourceEvaluator, Statement, Value};
use iris_runtime::ObjectId;
use iris_syntax::{CatchClause, MatchArm, MatchBody, Pattern, TypeExpression};
use std::collections::HashMap;
use std::rc::Rc;

mod adapters;
mod assignment;
mod control;
mod decorators;
mod expressions;
mod lifecycle;
mod roots;
mod scheduler;
mod statements;
mod task_completion;
use super::gc_roots::TraceRoots;
use roots::Stack;
#[cfg(test)]
mod tests;

type Outcome = Result<Value, EvaluationError>;

pub(super) struct Continuation {
    work: Stack<Step>,
    scopes: Stack<HashMap<String, Value>>,
    names: HashMap<String, Binding>,
    receiver: Option<Value>,
    context: LexicalContext,
    source: String,
    exception: Option<Value>,
    exception_context: Option<Value>,
    outcome: Outcome,
    temporary: u64,
}

pub(super) struct SuspendedTask {
    pub continuation: Continuation,
    pub waiting: ObjectId,
    pub order: u64,
}

enum Step {
    Adapter(adapters::Adapter),
    Block(Rc<Vec<Statement>>, usize),
    ExitScope(HashMap<String, Binding>),
    Statement(Statement),
    Expression(Expression),
    Bind(String, bool, Option<TypeExpression>, Value),
    Operands(Operands),
    Operand(Operands, String),
    Assign(Operands, Option<Value>),
    Await,
    NonNull,
    Branch(Vec<Statement>, Option<Vec<Statement>>),
    Logical(iris_syntax::BinaryOperator, Expression),
    Transfer(Statement),
    While(Loop),
    WhileTest(Loop),
    LoopBody(Loop),
    ForStart(ForLoop),
    ForNext(ForLoop, Value),
    ForBody(ForLoop, Value),
    Close(Value),
    Try(Vec<CatchClause>, Option<Vec<Statement>>),
    Finally(Option<Vec<Statement>>),
    FinishFinally(Outcome, Option<Value>, Option<Value>),
    RestoreException(Option<Value>),
    RestoreCall(Option<Value>, LexicalContext, HashMap<String, Binding>),
    Match(Vec<MatchArm>, Option<MatchBody>),
    MatchGuard(Value, Vec<MatchArm>, Option<MatchBody>, MatchBody),
}

struct Operands {
    expression: Expression,
    pending: std::collections::VecDeque<(String, Expression)>,
    values: HashMap<String, Value>,
}

#[derive(Clone)]
struct Loop {
    label: Option<String>,
    condition: Expression,
    body: Vec<Statement>,
}

#[derive(Clone)]
struct ForLoop {
    label: Option<String>,
    binding: Pattern,
    body: Vec<Statement>,
}

impl Continuation {
    fn new(
        evaluator: &SourceEvaluator,
        body: Vec<Statement>,
        locals: HashMap<String, Value>,
    ) -> Self {
        Self {
            work: Stack::new(vec![Step::Block(Rc::new(body), 0)]),
            scopes: Stack::new(vec![locals]),
            names: evaluator.names.clone(),
            receiver: None,
            context: evaluator.wrapper_context(),
            source: evaluator.source.clone(),
            exception: evaluator.active_exception.clone(),
            exception_context: evaluator.active_context.clone(),
            outcome: Ok(Value::Nil),
            temporary: 0,
        }
    }

    fn locals(&self) -> HashMap<String, Value> {
        self.scopes.last().unwrap_or_default()
    }

    fn enter(
        &mut self,
        evaluator: &SourceEvaluator,
        body: Vec<Statement>,
        locals: HashMap<String, Value>,
    ) {
        self.scopes.push(locals);
        self.work.push(Step::ExitScope(evaluator.names.clone()));
        self.work.push(Step::Block(Rc::new(body), 0));
        self.outcome = Ok(Value::Nil);
    }

    fn value(&mut self) -> Outcome {
        std::mem::replace(&mut self.outcome, Ok(Value::Nil))
    }

    pub(super) fn roots(&self) -> Vec<Value> {
        let mut roots = Vec::new();
        self.trace_roots(&mut roots);
        roots
    }
}
