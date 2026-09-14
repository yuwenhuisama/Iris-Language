use iris_syntax::{Declaration, ExportDeclaration, Program, ProgramEntry};

pub(crate) fn prepare_exports(program: &mut Program) {
    for declaration in &mut program.declarations {
        unwrap_class(declaration);
    }
    for entry in &mut program.entries {
        if let ProgramEntry::Declaration(declaration) = entry {
            unwrap_class(declaration);
        }
    }
}

fn unwrap_class(declaration: &mut Declaration) {
    if let Declaration::Export(export) = declaration
        && let ExportDeclaration::Declaration(inner) = export.as_ref()
        && matches!(inner.as_ref(), Declaration::Class(_))
    {
        *declaration = inner.as_ref().clone();
    }
}
