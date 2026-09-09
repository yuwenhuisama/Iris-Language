macro_rules! families {
    ($($variant:ident => $name:literal),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        pub enum BuiltinType { $($variant),+ }
        impl BuiltinType {
            pub const fn name(self) -> &'static str {
                match self { $(Self::$variant => $name),+ }
            }
            pub fn from_name(name: &str) -> Option<Self> {
                match name { $($name => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}

families! {
    Object => "Object", Nil => "Nil", Bool => "Bool", Integer => "Integer",
    Float32 => "Float32", Float64 => "Float64", String => "String", Symbol => "Symbol",
    Array => "Array", Hash => "Hash", Tuple => "Tuple", Range => "Range",
    ReadonlyArray => "ReadonlyArray", MutableString => "MutableString", Bytes => "Bytes",
    ByteArray => "ByteArray", Regex => "Regex", Match => "Match",
    ArrayIterator => "ArrayIterator", HashIterator => "HashIterator", ByteIterator => "ByteIterator",
    Generator => "Generator", Iteration => "Iteration", Closure => "Closure",
    BoundMethod => "BoundMethod", Method => "Method", Task => "Task", Gate => "Gate",
    ExceptionContext => "ExceptionContext", SourceLocation => "SourceLocation",
    StackFrame => "StackFrame", RaiseSite => "RaiseSite", ExternalResource => "ExternalResource",
    Class => "Class", Module => "Module", Type => "Type", ComposedType => "ComposedType",
    Contract => "Contract", ContractView => "ContractView", Transformation => "Transformation",
    FfiLibrary => "FFI::Library",
}
