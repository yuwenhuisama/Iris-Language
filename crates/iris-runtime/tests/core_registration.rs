use iris_runtime::{ClassRegistry, Kernel, KernelError, StaticSpine};

#[test]
fn user_identity_and_publication_are_unchanged_when_core_registration_is_eager()
-> Result<(), KernelError> {
    let mut eager = ClassRegistry::new();
    let mut eager_kernel = Kernel::new(&mut eager)?;
    let mut lazy = ClassRegistry::new();
    let mut lazy_kernel = Kernel::new(&mut lazy)?;

    eager_kernel.register_decorator_classes(&mut eager)?;
    let eager_user = eager.define_class(StaticSpine::new(1), None)?;
    let lazy_user = lazy.define_class(StaticSpine::new(1), None)?;
    lazy_kernel.register_decorator_classes(&mut lazy)?;

    assert_eq!(eager_user, lazy_user);
    assert_eq!(eager.active(eager_user)?, lazy.active(lazy_user)?);
    assert_eq!(
        eager.publish(eager.open(eager_user)?)?,
        lazy.publish(lazy.open(lazy_user)?)?
    );
    Ok(())
}

#[test]
fn core_identity_is_unchanged_when_registered_after_a_user_class() -> Result<(), KernelError> {
    let mut eager = ClassRegistry::new();
    let mut eager_kernel = Kernel::new(&mut eager)?;
    let mut lazy = ClassRegistry::new();
    let mut lazy_kernel = Kernel::new(&mut lazy)?;

    eager_kernel.register_decorator_classes(&mut eager)?;
    let user = lazy.define_class(StaticSpine::new(1), None)?;
    lazy_kernel.register_decorator_classes(&mut lazy)?;

    for name in ["Invocation", "ArgumentChanges", "Plan", "Array", "Hash"] {
        assert_eq!(eager_kernel.core_class(name), lazy_kernel.core_class(name));
        assert_ne!(lazy_kernel.core_class(name), Some(user));
    }
    Ok(())
}
