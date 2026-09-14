use super::{DecoratorProtocolError, ProtocolCategory};
use crate::{ObjectId, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskId(ObjectId);
impl TaskId {
    pub const fn new(id: ObjectId) -> Self {
        Self(id)
    }
    pub const fn object_id(self) -> ObjectId {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivationId(ObjectId);
impl ActivationId {
    pub const fn new(id: ObjectId) -> Self {
        Self(id)
    }
    pub const fn object_id(self) -> ObjectId {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeOwner {
    Synchronous {
        activation: ActivationId,
        entering_task: Option<TaskId>,
    },
    Asynchronous(TaskId),
}

impl ScopeOwner {
    pub const fn task(self) -> Option<TaskId> {
        match self {
            Self::Synchronous { entering_task, .. } => entering_task,
            Self::Asynchronous(task) => Some(task),
        }
    }
}

/// Backend lifecycle bookkeeping only; this neither runs nor settles Tasks.
#[derive(Debug)]
pub struct NextScope {
    next: ObjectId,
    owner: ScopeOwner,
    active: bool,
    in_flight: bool,
    bridge: Option<TaskId>,
}

impl NextScope {
    pub const fn synchronous(
        next: ObjectId,
        activation: ActivationId,
        entering_task: Option<TaskId>,
    ) -> Self {
        Self {
            next,
            owner: ScopeOwner::Synchronous {
                activation,
                entering_task,
            },
            active: true,
            in_flight: false,
            bridge: None,
        }
    }
    pub const fn asynchronous(next: ObjectId, owner: TaskId) -> Self {
        Self {
            next,
            owner: ScopeOwner::Asynchronous(owner),
            active: true,
            in_flight: false,
            bridge: None,
        }
    }
    pub const fn next(&self) -> ObjectId {
        self.next
    }
    pub const fn owner(&self) -> ScopeOwner {
        self.owner
    }
    pub const fn in_flight(&self) -> bool {
        self.in_flight
    }
    pub const fn bridge(&self) -> Option<TaskId> {
        self.bridge
    }

    pub fn begin_attempt(
        &mut self,
        invoking_task: Option<TaskId>,
        bridge: Option<TaskId>,
    ) -> Result<(), DecoratorProtocolError> {
        if !self.active {
            return Err(self.error(ProtocolCategory::Expired));
        }
        if invoking_task != self.owner.task() {
            return Err(self.error(ProtocolCategory::ForeignTask));
        }
        if self.in_flight {
            return Err(self.error(ProtocolCategory::Overlap));
        }
        self.in_flight = true;
        self.bridge = bridge;
        Ok(())
    }
    pub fn finish_attempt(&mut self) {
        self.in_flight = false;
        self.bridge = None;
    }
    pub fn finish_owner(&mut self) -> Option<DecoratorProtocolError> {
        let active = self.active;
        self.active = false;
        match self.owner {
            ScopeOwner::Asynchronous(_) if active && self.in_flight => {
                Some(self.error(ProtocolCategory::UnfinishedInner))
            }
            ScopeOwner::Asynchronous(_) | ScopeOwner::Synchronous { .. } => None,
        }
    }
    fn error(&self, category: ProtocolCategory) -> DecoratorProtocolError {
        DecoratorProtocolError::new(category, Some(self.next))
    }
    pub fn visit_values(&self, visitor: &mut impl FnMut(&Value)) {
        visitor(&Value::Closure(self.next));
        if let Some(task) = self.owner.task() {
            visitor(&Value::Task(task.object_id()));
        }
        if let Some(bridge) = self.bridge {
            visitor(&Value::Task(bridge.object_id()));
        }
    }
}
