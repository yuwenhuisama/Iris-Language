use iris_runtime::{ComposedType, TypeAtom, Value};
use iris_syntax::TypeExpression;

use super::{Machine, MachineError};
use crate::compile::Program;

impl Machine {
    pub(super) fn reify_type(
        &self,
        expression: &TypeExpression,
        program: &Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<Value, MachineError> {
        // A CLOSED generic names one Type per ARGUMENT list: `Box<String>` and
        // `Box<Integer>` are two Types of one class, so the arguments are
        // carried rather than dropped - without them the two are the same
        // value and `same?` cannot tell them apart.
        if let TypeExpression::Generic { name, arguments } = expression
            && let TypeAtom::Nominal(class, _) = self.type_atom(name, program, classes)?
        {
            let mut reified = Vec::with_capacity(arguments.len());
            for argument in arguments {
                let TypeExpression::Name(argument) = argument else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                let TypeAtom::Nominal(argument, _) = self.type_atom(argument, program, classes)?
                else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                reified.push(argument);
            }
            return Ok(Value::Type(class, reified));
        }
        let form = self.normalize_type(expression, program, classes)?;
        Ok(match form {
            ComposedType::Union(members) | ComposedType::Intersection(members)
                if members.len() == 1 =>
            {
                match &members[0] {
                    TypeAtom::Nominal(class, arguments) => Value::Type(*class, arguments.clone()),
                    TypeAtom::NonNil | TypeAtom::Contract(_) | TypeAtom::Union(_) => {
                        Value::ComposedType(ComposedType::Intersection(members))
                    }
                }
            }
            form => Value::ComposedType(form),
        })
    }

    fn normalize_type(
        &self,
        expression: &TypeExpression,
        program: &Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<ComposedType, MachineError> {
        match expression {
            TypeExpression::Name(name) if name == "Never" => Ok(ComposedType::Never),
            TypeExpression::Name(name) if name == "NonNil" => {
                Ok(ComposedType::Intersection(vec![TypeAtom::NonNil]))
            }
            TypeExpression::Name(name) => Ok(ComposedType::Union(vec![
                self.type_atom(name, program, classes)?,
            ])),
            TypeExpression::Union(members) => {
                let mut atoms = Vec::new();
                for member in members {
                    match self.normalize_type(member, program, classes)? {
                        ComposedType::Never => {}
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
                for member in members {
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
            TypeExpression::Typeof(_)
            | TypeExpression::Generic { .. }
            | TypeExpression::Function { .. } => Err(MachineError::UnsupportedConstruct),
        }
    }

    fn type_atom(
        &self,
        name: &str,
        program: &Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<TypeAtom, MachineError> {
        if let Some(index) = program.classes.iter().position(|class| class.name == name) {
            return classes
                .get(index)
                .copied()
                .map(|class| TypeAtom::Nominal(class, Vec::new()))
                .ok_or(MachineError::NameError);
        }
        if let Some(index) = program
            .contracts
            .iter()
            .position(|contract| contract.name == name)
        {
            return Ok(TypeAtom::Contract(iris_runtime::ContractId::new(
                index as u64 + 1,
            )));
        }
        self.builtin_class(name)
            .map(|class| TypeAtom::Nominal(class, Vec::new()))
    }

    fn absorb(&self, mut atoms: Vec<TypeAtom>, union: bool) -> Result<Vec<TypeAtom>, MachineError> {
        if !union && atoms.contains(&TypeAtom::NonNil) {
            let nil = self.builtin_class("Nil")?;
            if atoms.len() == 1 {
                return Ok(atoms);
            }
            for atom in &mut atoms {
                if let TypeAtom::Union(nested) = atom {
                    nested.retain(
                        |member| !matches!(member, TypeAtom::Nominal(class, _) if *class == nil),
                    );
                    if nested.len() == 1 {
                        *atom = nested[0].clone();
                    }
                }
            }
            let had_other = atoms
                .iter()
                .any(|atom| !matches!(atom, TypeAtom::Nominal(class, _) if *class == nil))
                && atoms.iter().any(|atom| *atom != TypeAtom::NonNil);
            atoms.retain(|atom| !matches!(atom, TypeAtom::Nominal(class, _) if *class == nil));
            atoms.retain(|atom| *atom != TypeAtom::NonNil);
            if !had_other {
                return Ok(Vec::new());
            }
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
        if atoms.is_empty() {
            return ComposedType::Never;
        }
        build(atoms)
    }
}

impl Machine {
    /// Reports whether a value satisfies an annotation.
    ///
    /// `IRIS-V1-TYPES-C004` guards the return boundary with this. An
    /// annotation the backend cannot DECIDE admits every value, so an
    /// unmodelled Type stays permissive rather than refusing a program the
    /// reference runs - the check exists to catch a definite mismatch, not to
    /// narrow the accepted surface.
    pub(super) fn annotation_admits(
        &self,
        value: &Value,
        annotation: &TypeExpression,
        program: &crate::compile::Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<bool, MachineError> {
        match annotation {
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
                            return Ok(true);
                        };
                        let Some(class) = classes.get(index).copied() else {
                            return Ok(true);
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
            // A `BoundMethod<..>` names a Method BOUND to a receiver, and a
            // `Closure<..>` names a closure: neither admits the other, however
            // alike their call signatures look. Admitting every generic left
            // `let m: BoundMethod<..> = { |x| x }` accepted.
            TypeExpression::Generic { name, .. } if name == "BoundMethod" => {
                Ok(matches!(value, Value::BoundMethod(_) | Value::Method(_)))
            }
            TypeExpression::Generic { name, .. } if name == "Closure" => {
                Ok(matches!(value, Value::Closure(_)))
            }
            TypeExpression::Typeof(_)
            | TypeExpression::Generic { .. }
            | TypeExpression::Function { .. } => Ok(true),
        }
    }
}
