use super::provably_non_nil;
use iris_runtime::{ClassId, ContractId, TypeAtom};

#[test]
fn nonnil_proof_stays_conservative_when_atoms_are_composed() {
    let nil = ClassId::new(1);
    let object = ClassId::new(2);
    let string = TypeAtom::Nominal(ClassId::new(3), Vec::new());
    let contract = TypeAtom::Contract(ContractId::new(1), Vec::new());
    for (given, expected) in [
        (TypeAtom::Nominal(nil, Vec::new()), false),
        (TypeAtom::Nominal(object, Vec::new()), false),
        (TypeAtom::NonNil, true),
        (string.clone(), true),
        (contract.clone(), false),
        (TypeAtom::Iteration(Vec::new()), false),
        (
            TypeAtom::Union(vec![string.clone(), contract.clone()]),
            false,
        ),
        (
            TypeAtom::Union(vec![string.clone(), TypeAtom::NonNil]),
            true,
        ),
        (TypeAtom::Intersection(vec![string, contract.clone()]), true),
        (
            TypeAtom::Intersection(vec![TypeAtom::Nominal(object, Vec::new()), contract]),
            false,
        ),
    ] {
        let when = provably_non_nil(&given, (nil, object));

        assert_eq!(when, expected, "{given:?}");
    }
}
