#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecoratorKind {
    Class,
    Module,
    Contract,
    Method,
    Property,
}

impl DecoratorKind {
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Class => "class",
            Self::Module => "module",
            Self::Contract => "contract",
            Self::Method => "method",
            Self::Property => "property",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecoratorReason {
    Origin,
    Open,
    Upgrade,
    Rollback,
    ClosedMaterialization,
}

impl DecoratorReason {
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Origin => "origin",
            Self::Open => "open",
            Self::Upgrade => "upgrade",
            Self::Rollback => "rollback",
            Self::ClosedMaterialization => "closed_materialization",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParameterCategory {
    Positional,
    Rest,
    Keyword,
    KeywordRest,
    Block,
}

impl ParameterCategory {
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Positional => "positional",
            Self::Rest => "rest",
            Self::Keyword => "keyword",
            Self::KeywordRest => "keyword_rest",
            Self::Block => "block",
        }
    }
}
