use iris_runtime::{ComposedType, NominalType, TypeAtom, Value};
use iris_syntax::TypeExpression;

use super::{Machine, MachineError};
use crate::compile::Program;

#[cfg(test)]
#[path = "composed_types_tests.rs"]
mod tests;

impl Machine {
    pub(super) fn reify_type(
        &mut self,
        expression: &TypeExpression,
        program: &Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<Value, MachineError> {
        if let TypeExpression::Name(name) = expression
            && let Some((_, value)) = self
                .method_types
                .iter()
                .find(|(parameter, _)| parameter == name)
        {
            return Ok(value.clone());
        }
        if matches!(expression, TypeExpression::Generic { name, .. } if matches!(name.as_str(), "Block" | "Closure" | "BoundMethod"))
        {
            return self.reify_callable(expression, (program, classes));
        }
        // A CLOSED generic names one Type per ARGUMENT list: `Box<String>` and
        // `Box<Integer>` are two Types of one class, so the arguments are
        // carried rather than dropped - without them the two are the same
        // value and `same?` cannot tell them apart.
        if let TypeExpression::Generic { name, arguments } = expression {
            if name == "Iteration" {
                return Ok(Value::ComposedType(ComposedType::Intersection(vec![
                    TypeAtom::Iteration(self.nominal_arguments(arguments, program, classes)?),
                ])));
            }
            match self.type_atom(name, program, classes)? {
                TypeAtom::Nominal(class, _) => {
                    return Ok(Value::Type(
                        class,
                        self.nominal_arguments(arguments, program, classes)?,
                    ));
                }
                TypeAtom::Contract(contract, _) => {
                    let arguments = self.nominal_arguments(arguments, program, classes)?;
                    return Ok(Value::Contract(contract, arguments));
                }
                TypeAtom::Iteration(arguments) => {
                    return Ok(Value::ComposedType(ComposedType::Intersection(vec![
                        TypeAtom::Iteration(arguments),
                    ])));
                }
                TypeAtom::NonNil | TypeAtom::Union(_) | TypeAtom::Intersection(_) => {
                    return Err(MachineError::UnsupportedConstruct);
                }
            }
        }
        let form = self.normalize_type(expression, program, classes)?;
        Ok(match form {
            ComposedType::Union(members) | ComposedType::Intersection(members)
                if members.len() == 1 =>
            {
                match &members[0] {
                    TypeAtom::Nominal(class, arguments) => Value::Type(*class, arguments.clone()),
                    TypeAtom::NonNil
                    | TypeAtom::Contract(_, _)
                    | TypeAtom::Iteration(_)
                    | TypeAtom::Union(_)
                    | TypeAtom::Intersection(_) => {
                        Value::ComposedType(ComposedType::Intersection(members))
                    }
                }
            }
            form => Value::ComposedType(form),
        })
    }

    fn normalize_type(
        &mut self,
        expression: &TypeExpression,
        program: &Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<ComposedType, MachineError> {
        match expression {
            TypeExpression::Name(name) if name == "Never" => Ok(ComposedType::Never),
            TypeExpression::Name(name) if name == "NonNil" => {
                Ok(ComposedType::Intersection(vec![TypeAtom::NonNil]))
            }
            TypeExpression::Name(name) if name == "Iteration" => {
                Ok(ComposedType::Union(vec![TypeAtom::Iteration(Vec::new())]))
            }
            TypeExpression::Name(name) => Ok(ComposedType::Union(vec![
                self.type_atom(name, program, classes)?,
            ])),
            TypeExpression::Union(members) => {
                let mut atoms = Vec::new();
                let mut pending: Vec<_> = members.iter().collect();
                while let Some(member) = pending.pop() {
                    if let TypeExpression::Union(nested) = member {
                        pending.extend(nested);
                        continue;
                    }
                    match self.normalize_type(member, program, classes)? {
                        ComposedType::Never => {}
                        ComposedType::Intersection(nested) if nested.len() > 1 => {
                            atoms.push(TypeAtom::Intersection(nested));
                        }
                        ComposedType::Union(members) | ComposedType::Intersection(members) => {
                            atoms.extend(members);
                        }
                    }
                }
                Ok(Self::canonical(
                    ComposedType::Union,
                    self.absorb(atoms, true)?,
                ))
            }
            TypeExpression::Intersection(members) => {
                let mut atoms = Vec::new();
                let mut pending: Vec<_> = members.iter().collect();
                while let Some(member) = pending.pop() {
                    if let TypeExpression::Intersection(nested) = member {
                        pending.extend(nested);
                        continue;
                    }
                    match self.normalize_type(member, program, classes)? {
                        ComposedType::Never => return Ok(ComposedType::Never),
                        ComposedType::Union(nested) if nested.len() > 1 => {
                            atoms.push(TypeAtom::Union(nested));
                        }
                        ComposedType::Union(members) | ComposedType::Intersection(members) => {
                            atoms.extend(members);
                        }
                    }
                }
                let atoms = self.absorb(atoms, false)?;
                if atoms.is_empty() {
                    return Ok(ComposedType::Never);
                }
                Ok(Self::canonical(ComposedType::Intersection, atoms))
            }
            TypeExpression::Generic { name, arguments } if name == "Iteration" => {
                Ok(ComposedType::Intersection(vec![TypeAtom::Iteration(
                    self.nominal_arguments(arguments, program, classes)?,
                )]))
            }
            TypeExpression::Generic { .. } => {
                match self.reify_type(expression, program, classes)? {
                    Value::Contract(contract, arguments) => {
                        Ok(ComposedType::Union(vec![TypeAtom::Contract(
                            contract, arguments,
                        )]))
                    }
                    Value::Type(class, arguments) => {
                        Ok(ComposedType::Union(vec![TypeAtom::Nominal(
                            class, arguments,
                        )]))
                    }
                    Value::ComposedType(form) => Ok(form),
                    _ => Err(MachineError::UnsupportedConstruct),
                }
            }
            TypeExpression::Typeof(_) | TypeExpression::Function { .. } => {
                Err(MachineError::UnsupportedConstruct)
            }
        }
    }

    fn type_atom(
        &self,
        name: &str,
        program: &Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<TypeAtom, MachineError> {
        if let Some((_, value)) = self
            .method_types
            .iter()
            .find(|(parameter, _)| parameter == name)
        {
            return match value {
                Value::Type(class, arguments) => Ok(TypeAtom::Nominal(*class, arguments.clone())),
                _ => Err(MachineError::UnsupportedConstruct),
            };
        }
        if let Some(index) = program.classes.iter().position(|class| class.name == name) {
            return classes
                .get(index)
                .copied()
                .ok_or(MachineError::NameError)
                .and_then(|class| {
                    self.runtime
                        .registry()
                        .class(class)
                        .map_err(MachineError::Class)?;
                    Ok(TypeAtom::Nominal(class, Vec::new()))
                });
        }
        if let Some(index) = program
            .contracts
            .iter()
            .rposition(|contract| contract.name == name)
        {
            return Ok(TypeAtom::Contract(
                program.contract_identity(index),
                Vec::new(),
            ));
        }
        self.builtin_class(name)
            .map(|class| TypeAtom::Nominal(class, Vec::new()))
    }

    pub(super) fn nominal_arguments(
        &mut self,
        arguments: &[TypeExpression],
        program: &Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<Vec<NominalType>, MachineError> {
        let mut reified = Vec::with_capacity(arguments.len());
        for argument in arguments {
            reified.push(self.nominal_type(argument, program, classes)?);
        }
        Ok(reified)
    }

    fn nominal_type(
        &mut self,
        expression: &TypeExpression,
        program: &Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<NominalType, MachineError> {
        match expression {
            TypeExpression::Name(name) => {
                let TypeAtom::Nominal(class, arguments) = self.type_atom(name, program, classes)?
                else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                Ok(NominalType::new(class, arguments))
            }
            TypeExpression::Generic { name, arguments } => {
                if matches!(name.as_str(), "Closure" | "BoundMethod") {
                    let Value::Type(class, arguments) =
                        self.reify_callable(expression, (program, classes))?
                    else {
                        return Err(MachineError::UnsupportedConstruct);
                    };
                    return Ok(NominalType::new(class, arguments));
                }
                let TypeAtom::Nominal(class, _) = self.type_atom(name, program, classes)? else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                Ok(NominalType::new(
                    class,
                    self.nominal_arguments(arguments, program, classes)?,
                ))
            }
            TypeExpression::Typeof(_)
            | TypeExpression::Intersection(_)
            | TypeExpression::Union(_)
            | TypeExpression::Function { .. } => Err(MachineError::UnsupportedConstruct),
        }
    }

    pub(super) fn reify_contract_requirement_type(
        &mut self,
        expression: &TypeExpression,
        arguments: &[NominalType],
        program: &Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<Value, MachineError> {
        let TypeExpression::Generic {
            name,
            arguments: parameters,
        } = expression
        else {
            return self.reify_type(expression, program, classes);
        };
        if !matches!(parameters.as_slice(), [TypeExpression::Name(parameter)] if parameter == "T") {
            return Ok(match self.reify_type(expression, program, classes)? {
                Value::Contract(contract, arguments) => Value::ComposedType(
                    ComposedType::Intersection(vec![TypeAtom::Contract(contract, arguments)]),
                ),
                reified => reified,
            });
        }
        let argument = arguments
            .first()
            .cloned()
            .ok_or(MachineError::UnsupportedConstruct)?;
        if name == "Iteration" {
            return Ok(Value::ComposedType(ComposedType::Intersection(vec![
                TypeAtom::Iteration(vec![argument]),
            ])));
        }
        let TypeAtom::Contract(contract, _) = self.type_atom(name, program, classes)? else {
            return Err(MachineError::UnsupportedConstruct);
        };
        Ok(Value::ComposedType(ComposedType::Intersection(vec![
            TypeAtom::Contract(contract, vec![argument]),
        ])))
    }

    fn absorb(&self, mut atoms: Vec<TypeAtom>, union: bool) -> Result<Vec<TypeAtom>, MachineError> {
        atoms.sort();
        atoms.dedup();
        if !union && atoms.contains(&TypeAtom::NonNil) {
            let nil = self.builtin_class("Nil")?;
            let object = self.builtin_class("Object")?;
            let mut narrowed = Vec::new();
            while let Some(atom) = atoms.pop() {
                match atom {
                    TypeAtom::Nominal(class, _) if class == nil => return Ok(Vec::new()),
                    TypeAtom::Union(mut nested) => {
                        nested.retain(
                            |member| !matches!(member, TypeAtom::Nominal(class, _) if *class == nil),
                        );
                        match nested.len() {
                            0 => return Ok(Vec::new()),
                            1 => atoms.extend(nested),
                            _ => narrowed.push(TypeAtom::Union(nested)),
                        }
                    }
                    TypeAtom::Intersection(nested) => atoms.extend(nested),
                    atom => narrowed.push(atom),
                }
            }
            if narrowed
                .iter()
                .any(|atom| *atom != TypeAtom::NonNil && provably_non_nil(atom, (nil, object)))
            {
                narrowed.retain(|atom| *atom != TypeAtom::NonNil);
            }
            atoms = narrowed;
            atoms.sort();
            atoms.dedup();
        }
        let mut kept = Vec::new();
        for atom in atoms {
            let mut absorbed = false;
            let mut survivors = Vec::new();
            for existing in kept {
                match (&atom, &existing) {
                    (TypeAtom::Nominal(left, la), TypeAtom::Nominal(right, ra))
                        if la.is_empty() && ra.is_empty() =>
                    {
                        let atom_wider = self.is_subtype(*right, *left)?;
                        let existing_wider = self.is_subtype(*left, *right)?;
                        if (union && atom_wider) || (!union && existing_wider) {
                            continue;
                        }
                        if (union && existing_wider) || (!union && atom_wider) {
                            absorbed = true;
                        }
                        survivors.push(existing);
                    }
                    _ => survivors.push(existing),
                }
            }
            kept = survivors;
            if !absorbed {
                kept.push(atom);
            }
        }
        Ok(kept)
    }

    fn canonical(
        build: fn(Vec<TypeAtom>) -> ComposedType,
        mut atoms: Vec<TypeAtom>,
    ) -> ComposedType {
        atoms.sort();
        atoms.dedup();
        match atoms.as_slice() {
            [] => ComposedType::Never,
            [TypeAtom::Union(members)] => ComposedType::Union(members.clone()),
            [TypeAtom::Intersection(members)] => ComposedType::Intersection(members.clone()),
            _ => build(atoms),
        }
    }
}

fn provably_non_nil(
    atom: &TypeAtom,
    (nil, object): (iris_runtime::ClassId, iris_runtime::ClassId),
) -> bool {
    match atom {
        TypeAtom::Nominal(class, _) => *class != nil && *class != object,
        TypeAtom::NonNil => true,
        TypeAtom::Union(members) => members
            .iter()
            .all(|member| provably_non_nil(member, (nil, object))),
        TypeAtom::Intersection(members) => members
            .iter()
            .any(|member| provably_non_nil(member, (nil, object))),
        TypeAtom::Contract(_, _) | TypeAtom::Iteration(_) => false,
    }
}

impl Machine {
    /// Reports whether a value satisfies an annotation.
    ///
    /// Unknown or unsupported annotations are rejected at the boundary.
    pub(super) fn annotation_admits(
        &mut self,
        value: &Value,
        annotation: &TypeExpression,
        program: &crate::compile::Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<bool, MachineError> {
        if let TypeExpression::Name(name) | TypeExpression::Generic { name, .. } = annotation
            && let Some(contract) = program
                .contracts
                .iter()
                .rposition(|known| known.name == *name)
        {
            let arguments = match annotation {
                TypeExpression::Generic { arguments, .. } => arguments.as_slice(),
                _ => &[],
            };
            let requested = self.nominal_arguments(arguments, program, classes)?;
            let receiver = match value {
                Value::ContractView(receiver, _) => receiver.as_ref(),
                value => value,
            };
            let Value::Object(object) = receiver else {
                return Ok(false);
            };
            let class = self
                .runtime
                .class_of(*object)
                .map_err(MachineError::Construction)?;
            let mut current = classes.iter().position(|known| *known == class);
            while let Some(index) = current {
                if let Some((_, expressions)) = program.classes[index]
                    .contract_arguments
                    .iter()
                    .find(|(index, _)| *index == contract)
                {
                    let bindings = self.receiver_type_bindings(receiver, (program, classes))?;
                    let actual = self.with_method_types(bindings, |machine| {
                        machine.nominal_arguments(expressions, program, classes)
                    })?;
                    return Ok(actual == requested);
                }
                current = program.classes[index].superclass;
            }
            return Ok(false);
        }
        if let TypeExpression::Name(name) = annotation
            && let Some((_, target)) = self
                .method_types
                .iter()
                .find(|(parameter, _)| parameter == name)
        {
            return Ok(self.decorator_accepts(target, value));
        }
        match annotation {
            TypeExpression::Generic { name, arguments }
                if matches!(name.as_str(), "Task" | "Kernel::Task") =>
            {
                let (Value::Task(identity), [result]) = (value, arguments.as_slice()) else {
                    return Ok(false);
                };
                let expected = self.reify_type(result, program, classes)?;
                Ok(self.task_types.get(identity) == Some(&expected))
            }
            TypeExpression::Name(name) if name == "Array" => Ok(matches!(
                value,
                Value::Array(_) | Value::ReadonlyArray(_) | Value::ImmutableArray(_)
            )),
            TypeExpression::Name(name) if name == "Hash" => {
                Ok(matches!(value, Value::Hash(_) | Value::ImmutableHash(_)))
            }
            TypeExpression::Name(name) if name == "Symbol" => Ok(matches!(value, Value::Symbol(_))),
            TypeExpression::Name(name) if matches!(name.as_str(), "Bytes" | "Kernel::Bytes") => {
                Ok(matches!(value, Value::Bytes(_)))
            }
            // `C011` admits every value EXCEPT nil, and `C023` makes `Never`
            // uninhabited, so neither resolves through a declared class.
            TypeExpression::Name(name) if name == "NonNil" => Ok(!matches!(value, Value::Nil)),
            TypeExpression::Name(name) if name == "Never" => Ok(false),
            TypeExpression::Name(name) => {
                let class = match self.builtin_class(name) {
                    Ok(class) => class,
                    Err(_) => {
                        let Some(index) = program
                            .classes
                            .iter()
                            .position(|declaration| declaration.name == *name)
                        else {
                            return Err(MachineError::NameError);
                        };
                        let Some(class) = classes.get(index).copied() else {
                            return Err(MachineError::NameError);
                        };
                        class
                    }
                };
                Ok(self.type_test(value, &Value::Class(class))? == Value::Bool(true))
            }
            // `C020` admits a value satisfying ANY constituent, `C022` one
            // satisfying EVERY constituent.
            TypeExpression::Union(members) => {
                for member in members {
                    if self.annotation_admits(value, member, program, classes)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            TypeExpression::Intersection(members) => {
                for member in members {
                    if !self.annotation_admits(value, member, program, classes)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            // `Dynamic<T>` admits what T admits: the wrapper defers the check
            // to run time rather than removing it, so `let x: Dynamic<String>
            // = 1` is still the failure `String` would give.
            TypeExpression::Generic { name, arguments }
                if name == "Dynamic"
                    && let [argument] = arguments.as_slice() =>
            {
                self.annotation_admits(value, argument, program, classes)
            }
            TypeExpression::Generic { arguments, .. }
                if arguments.iter().any(contains_placeholder) =>
            {
                Err(MachineError::NameError)
            }
            // A `BoundMethod<..>` names a Method BOUND to a receiver, and a
            // `Closure<..>` names a closure: neither admits the other, however
            // alike their call signatures look. Admitting every generic left
            // `let m: BoundMethod<..> = { |x| x }` accepted.
            TypeExpression::Generic { name, .. }
                if matches!(name.as_str(), "BoundMethod" | "Block" | "Closure") =>
            {
                let target = self.reify_callable(annotation, (program, classes))?;
                Ok(self.decorator_accepts(&target, value))
            }
            TypeExpression::Generic { name, arguments } => {
                match (name.as_str(), arguments.as_slice(), value) {
                    ("Array", [element], Value::Array(array)) => {
                        for value in array.elements() {
                            if !self.annotation_admits(&value, element, program, classes)? {
                                return Ok(false);
                            }
                        }
                        return Ok(true);
                    }
                    ("Hash", [key_type, value_type], Value::Hash(hash)) => {
                        for (key, value) in hash.entries() {
                            if !self.annotation_admits(&key, key_type, program, classes)?
                                || !self.annotation_admits(&value, value_type, program, classes)?
                            {
                                return Ok(false);
                            }
                        }
                        return Ok(true);
                    }
                    _ => {}
                }
                if matches!(value, Value::ImmutableArray(_) | Value::ImmutableHash(_)) {
                    let target = self.reify_type(annotation, program, classes)?;
                    return Ok(self.type_test(value, &target)? == Value::Bool(true));
                }
                let TypeAtom::Nominal(class, _) = self.type_atom(name, program, classes)? else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                let target_arguments = self.nominal_arguments(arguments, program, classes)?;
                self.instance_admits(value, class, &target_arguments)
            }
            TypeExpression::Typeof(_) | TypeExpression::Function { .. } => {
                Err(MachineError::UnsupportedConstruct)
            }
        }
    }

    pub(super) fn instance_admits(
        &self,
        value: &Value,
        target: iris_runtime::ClassId,
        target_arguments: &[NominalType],
    ) -> Result<bool, MachineError> {
        let Value::Object(object) = value else {
            return Ok(false);
        };
        let class = self
            .runtime
            .class_of(*object)
            .map_err(MachineError::Construction)?;
        if class == target && !target_arguments.is_empty() {
            return self
                .runtime
                .type_arguments_of(*object)
                .map(|arguments| arguments == target_arguments)
                .map_err(MachineError::Construction);
        }
        self.is_subtype(class, target)
    }
}

fn contains_placeholder(expression: &TypeExpression) -> bool {
    match expression {
        TypeExpression::Name(name) => name == "_",
        TypeExpression::Generic { arguments, .. }
        | TypeExpression::Union(arguments)
        | TypeExpression::Intersection(arguments) => arguments.iter().any(contains_placeholder),
        TypeExpression::Function { parameters, result } => {
            parameters.iter().any(contains_placeholder) || contains_placeholder(result)
        }
        TypeExpression::Typeof(_) => false,
    }
}
