use super::*;
use crate::source_runtime::gc_roots::TraceRoots;
use std::cell::{RefCell, RefMut};

pub(super) struct Stack<T>(Rc<RefCell<Vec<T>>>);

impl<T> Stack<T> {
    pub fn new(values: Vec<T>) -> Self {
        Self(Rc::new(RefCell::new(values)))
    }
    pub fn push(&self, value: T) {
        self.0.borrow_mut().push(value);
    }
    pub fn pop(&self) -> Option<T> {
        self.0.borrow_mut().pop()
    }
    pub fn insert(&self, index: usize, value: T) {
        self.0.borrow_mut().insert(index, value);
    }
    pub fn last_mut(&self) -> Option<RefMut<'_, T>> {
        RefMut::filter_map(self.0.borrow_mut(), |values| values.last_mut()).ok()
    }
}

impl<T: Clone> Stack<T> {
    pub fn last(&self) -> Option<T> {
        self.0.borrow().last().cloned()
    }
}

impl<T: TraceRoots> TraceRoots for Stack<T> {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        self.0.trace_roots(roots);
    }
}

impl<T> Clone for Stack<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl TraceRoots for Step {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        match self {
            Self::Adapter(adapter) => adapter.roots(roots),
            Self::Operands(operands) | Self::Operand(operands, _) => {
                operands.values.trace_roots(roots)
            }
            Self::Assign(operands, current) => {
                operands.values.trace_roots(roots);
                current.trace_roots(roots);
            }
            Self::ExitScope(names) => names.trace_roots(roots),
            Self::RestoreCall(receiver, context, names) => {
                receiver.trace_roots(roots);
                context.trace_roots(roots);
                names.trace_roots(roots);
            }
            Self::Bind(_, _, _, value)
            | Self::Close(value)
            | Self::ForNext(_, value)
            | Self::ForBody(_, value)
            | Self::MatchGuard(value, ..)
            | Self::SafeNavigationIndex(value, ..) => value.trace_roots(roots),
            Self::FinishFinally(outcome, previous, context) => {
                outcome.trace_roots(roots);
                previous.trace_roots(roots);
                context.trace_roots(roots);
            }
            Self::RestoreException(value) => value.trace_roots(roots),
            Self::Block(..)
            | Self::Statement(_)
            | Self::Expression(_)
            | Self::Await
            | Self::NonNull
            | Self::RemoveTemporary(_)
            | Self::SafeNavigation(..)
            | Self::Branch(..)
            | Self::Logical(..)
            | Self::Transfer(_)
            | Self::While(_)
            | Self::WhileTest(_)
            | Self::LoopBody(_)
            | Self::ForStart(_)
            | Self::Try(..)
            | Self::Finally(_)
            | Self::Match(..) => {}
        }
    }
}

impl TraceRoots for Operands {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        self.values.trace_roots(roots);
    }
}

impl TraceRoots for Continuation {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        self.work.trace_roots(roots);
        self.scopes.trace_roots(roots);
        self.names.trace_roots(roots);
        self.receiver.trace_roots(roots);
        self.exception.trace_roots(roots);
        self.exception_context.trace_roots(roots);
        self.context.trace_roots(roots);
        self.outcome.trace_roots(roots);
    }
}
