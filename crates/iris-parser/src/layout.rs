use crate::Parser;
use iris_syntax::{PropertyAccessor, PropertyAccessorKind, PropertyAccessors, Visibility};

impl Parser {
    pub(super) fn property_accessors(&mut self) -> Option<Option<PropertyAccessors>> {
        if !self.check_after_newlines("{") {
            return Some(None);
        }
        self.advance();
        self.skip_newlines();
        let mut members = Vec::new();
        while !self.check("}") && !self.at_end() {
            let visibility = self.method_visibility().unwrap_or(Visibility::Private);
            let kind = match self.peek() {
                Some("get") => PropertyAccessorKind::Get,
                Some("set") => PropertyAccessorKind::Set,
                _ => {
                    self.error("PARSE_UNEXPECTED_TOKEN");
                    return None;
                }
            };
            self.advance();
            self.expect(";")?;
            members.push(PropertyAccessor { kind, visibility });
            self.skip_newlines();
        }
        self.expect("}")?;
        Some(Some(PropertyAccessors { members }))
    }

    /// Only a property's outer initializer reserves an accessor-shaped suffix;
    /// delimited arguments and closure bodies retain ordinary trailing blocks.
    pub(super) fn property_accessor_block_start(&self) -> bool {
        self.property_initializer
            && self.check("{")
            && self.tokens[self.cursor + 1..]
                .iter()
                .find(|token| token.text != "\n")
                .is_some_and(|token| {
                    matches!(
                        token.text.as_str(),
                        "get" | "set" | "public" | "private" | "protected" | "}"
                    )
                })
    }

    pub(super) fn skip_newlines(&mut self) {
        while self.consume("\n") {}
    }

    pub(super) fn consume_after_newlines(&mut self, expected: &str) -> bool {
        let checkpoint = self.checkpoint();
        self.skip_newlines();
        if self.consume(expected) {
            true
        } else {
            self.restore(checkpoint);
            false
        }
    }

    pub(super) fn check_after_newlines(&mut self, expected: &str) -> bool {
        let checkpoint = self.checkpoint();
        self.skip_newlines();
        if self.check(expected) {
            true
        } else {
            self.restore(checkpoint);
            false
        }
    }

    pub(super) fn with_layout<T>(
        &mut self,
        delimited: bool,
        parse: impl FnOnce(&mut Self) -> Option<T>,
    ) -> Option<T> {
        let outer = std::mem::replace(&mut self.delimited_layout, delimited);
        let outer_property = std::mem::replace(&mut self.property_initializer, false);
        let result = parse(self);
        self.delimited_layout = outer;
        self.property_initializer = outer_property;
        result
    }

    pub(super) fn expression_layout(&mut self) {
        if self.delimited_layout {
            self.skip_newlines();
        }
    }
}
