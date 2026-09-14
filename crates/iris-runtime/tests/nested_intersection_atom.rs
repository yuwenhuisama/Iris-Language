use std::collections::HashSet;

use iris_runtime::{ClassId, TypeAtom};

#[test]
fn nested_intersection_atom_preserves_equality_and_hash_when_cloned() {
    // Given
    let atom = TypeAtom::Intersection(vec![
        TypeAtom::Nominal(ClassId::new(1), Vec::new()),
        TypeAtom::NonNil,
    ]);
    let mut atoms = HashSet::new();
    atoms.insert(atom.clone());

    // When
    let reconstructed = TypeAtom::Intersection(vec![
        TypeAtom::Nominal(ClassId::new(1), Vec::new()),
        TypeAtom::NonNil,
    ]);

    // Then
    assert_eq!(atom, reconstructed);
    assert!(atoms.contains(&reconstructed));
}
