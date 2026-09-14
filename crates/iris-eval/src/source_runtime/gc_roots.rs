use super::{Binding, EvaluationError, SourceEvaluator, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

pub(super) trait TraceRoots {
    fn trace_roots(&self, roots: &mut Vec<Value>);
}

impl TraceRoots for Value {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        roots.push(self.clone());
    }
}

impl<T: TraceRoots> TraceRoots for Vec<T> {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        for value in self {
            value.trace_roots(roots);
        }
    }
}

impl<T: TraceRoots> TraceRoots for Option<T> {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        if let Some(value) = self {
            value.trace_roots(roots);
        }
    }
}

impl<T: TraceRoots> TraceRoots for RefCell<T> {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        self.borrow().trace_roots(roots);
    }
}

impl<T: TraceRoots> TraceRoots for Rc<T> {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        self.as_ref().trace_roots(roots);
    }
}

impl TraceRoots for (Value, Value) {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        self.0.trace_roots(roots);
        self.1.trace_roots(roots);
    }
}

impl TraceRoots for Binding {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        roots.push(self.value());
    }
}

impl<T: TraceRoots> TraceRoots for HashMap<String, T> {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        for value in self.values() {
            value.trace_roots(roots);
        }
    }
}

impl TraceRoots for (String, Option<Binding>) {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        self.1.trace_roots(roots);
    }
}

impl TraceRoots for Result<Value, EvaluationError> {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        match self {
            Ok(value)
            | Err(
                EvaluationError::Raised(value)
                | EvaluationError::Return(value)
                | EvaluationError::LoopBreak(_, value)
                | EvaluationError::GeneratorYield(value, _),
            ) => roots.push(value.clone()),
            Err(_) => {}
        }
    }
}

#[derive(Default)]
pub(super) struct RootRegistry(Vec<Weak<dyn TraceRoots>>);

impl RootRegistry {
    pub fn register<T: TraceRoots + 'static>(&mut self, value: T) -> Rc<T> {
        self.0.retain(|root| root.strong_count() > 0);
        let value = Rc::new(value);
        let erased: Rc<dyn TraceRoots> = value.clone();
        self.0.push(Rc::downgrade(&erased));
        value
    }

    pub fn trace(&self, roots: &mut Vec<Value>) {
        for root in &self.0 {
            if let Some(root) = root.upgrade() {
                root.trace_roots(roots);
            }
        }
    }
}

impl SourceEvaluator {
    pub(super) fn rooted<T: TraceRoots + 'static>(&mut self, value: T) -> Rc<T> {
        self.active_roots.register(value)
    }

    pub(super) fn evaluate_arguments(
        &mut self,
        expressions: &[iris_syntax::Expression],
        environment: (&HashMap<String, Value>, Option<Value>),
    ) -> Result<Vec<Value>, EvaluationError> {
        let values = self.rooted(RefCell::new(Vec::new()));
        for expression in expressions {
            let value = self.expression(expression, environment.0, environment.1.clone())?;
            values.borrow_mut().push(value);
        }
        Ok(Rc::try_unwrap(values)
            .unwrap_or_else(|_| unreachable!())
            .into_inner())
    }
}
