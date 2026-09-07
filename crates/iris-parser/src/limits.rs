use crate::Parser;

const MAX_PARSE_DEPTH: usize = 32;
const MAX_EXPRESSION_NODES: usize = 256;

impl Parser {
    pub(super) fn nested<T>(&mut self, parse: impl FnOnce(&mut Self) -> Option<T>) -> Option<T> {
        if self.depth == MAX_PARSE_DEPTH || self.exhausted {
            self.exhausted = true;
            self.error("PARSE_RESOURCE_LIMIT");
            return None;
        }
        self.depth += 1;
        let result = parse(self);
        self.depth -= 1;
        result
    }

    pub(super) fn expression_node(&mut self) -> Option<()> {
        if self.expression_nodes == MAX_EXPRESSION_NODES {
            self.exhausted = true;
            self.error("PARSE_RESOURCE_LIMIT");
            return None;
        }
        self.expression_nodes += 1;
        Some(())
    }

    pub(super) fn ensure_progress(&mut self, start: usize) {
        if self.cursor == start && !self.at_end() {
            self.error("PARSE_UNEXPECTED_TOKEN");
            self.advance();
        }
    }
}
