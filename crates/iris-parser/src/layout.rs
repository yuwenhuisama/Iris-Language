use crate::Parser;

impl Parser {
    pub(super) fn validate_property_accessors(&mut self) -> Option<()> {
        if !self.check_after_newlines("{") {
            return Some(());
        }
        let checkpoint = self.checkpoint();
        self.advance();
        self.skip_newlines();
        while !self.check("}") && !self.at_end() {
            self.method_visibility();
            if !self.consume("get") && !self.consume("set") {
                self.error("PARSE_UNEXPECTED_TOKEN");
                return None;
            }
            self.expect(";")?;
            self.skip_newlines();
        }
        self.expect("}")?;
        self.restore(checkpoint);
        Some(())
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
        let result = parse(self);
        self.delimited_layout = outer;
        result
    }

    pub(super) fn expression_layout(&mut self) {
        if self.delimited_layout {
            self.skip_newlines();
        }
    }
}
