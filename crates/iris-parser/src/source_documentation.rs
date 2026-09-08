use crate::{
    recording::Recorder,
    source::{DeclarationKind, Documentation, SourceKind, Span},
};
use iris_lexer::{Comment, CommentKind};

impl Recorder {
    pub fn documented_entry(&mut self, mark: usize, start: usize) {
        for id in &self.document.roots[mark..] {
            match &self.document.node(*id).kind {
                SourceKind::Declaration(_) => self.documentation_starts.push((*id, start)),
                SourceKind::Export => {
                    for child in self.document.node(*id).children.clone() {
                        self.documentation_starts.push((child, start));
                    }
                }
                _ => {}
            }
        }
    }

    pub fn attach_documentation(&mut self, source: &str) {
        if self.documentation_starts.is_empty() {
            return;
        }
        let mut decorators: Vec<_> = self
            .document
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, SourceKind::Decorator))
            .map(|node| node.span)
            .collect();
        decorators.sort_unstable_by_key(|span| span.start);
        let mut damage: Vec<_> = self
            .document
            .recovery
            .iter()
            .map(|entry| entry.span.start)
            .collect();
        damage.sort_unstable();
        self.documentation_starts
            .sort_unstable_by_key(|(id, start)| (id.0, *start));
        self.documentation_starts.dedup_by_key(|(id, _)| *id);
        for &(id, start) in &self.documentation_starts {
            let node = self.document.node(id);
            let SourceKind::Declaration(declaration) = &node.kind else {
                continue;
            };
            if matches!(
                declaration.kind,
                DeclarationKind::Parameter
                    | DeclarationKind::TypeParameter
                    | DeclarationKind::PatternBinding
            ) {
                continue;
            }
            let first_decorator = decorators.partition_point(|span| span.start < start);
            let decorators_valid = decorators[first_decorator..]
                .iter()
                .take_while(|span| span.end <= declaration.name.span.start)
                .all(|decorator| {
                    let next = self
                        .document
                        .tokens
                        .partition_point(|token| token.offset.0 < decorator.end);
                    let next = self.document.tokens[next..]
                        .iter()
                        .find(|token| token.kind != iris_lexer::TokenKind::Newline);
                    next.is_some_and(|token| {
                        let gap = &source[decorator.end..token.offset.0];
                        gap.bytes().all(|byte| matches!(byte, b' ' | b'\t'))
                            || adjacent(source, decorator.end, token.offset.0)
                    })
                });
            if !decorators_valid {
                continue;
            }
            let end = self
                .document
                .comments
                .partition_point(|comment| comment.end.0 <= start);
            let Some(last) = end.checked_sub(1) else {
                continue;
            };
            let comments = &self.document.comments;
            if !is_doc(comments[last].kind)
                || !adjacent(source, comments[last].end.0, start)
                || !standalone(source, comments[last].offset.0)
            {
                continue;
            }
            let mut first = last;
            if comments[last].kind == CommentKind::DocumentationLine {
                while first > 0
                    && comments[first - 1].kind == CommentKind::DocumentationLine
                    && adjacent(source, comments[first - 1].end.0, comments[first].offset.0)
                    && standalone(source, comments[first - 1].offset.0)
                {
                    first -= 1;
                }
            }
            let span = Span {
                start: comments[first].offset.0,
                end: comments[last].end.0,
            };
            let first_damage = damage.partition_point(|offset| *offset < span.start);
            if damage
                .get(first_damage)
                .is_some_and(|offset| *offset <= node.span.end)
            {
                continue;
            }
            let (text, truncated) = doc_text(source, &comments[first..=last]);
            self.document.documentation.push(Documentation {
                declaration: id,
                span,
                text,
                truncated,
            });
        }
    }
}

const fn is_doc(kind: CommentKind) -> bool {
    matches!(
        kind,
        CommentKind::DocumentationLine | CommentKind::DocumentationBlock
    )
}

fn standalone(source: &str, start: usize) -> bool {
    let prefix = &source[..start];
    let line = prefix.rsplit(['\r', '\n']).next().unwrap_or("");
    line.trim_start_matches('\u{feff}')
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t'))
}

fn adjacent(source: &str, end: usize, start: usize) -> bool {
    let mut newlines = 0;
    let mut bytes = source[end..start].bytes().peekable();
    while let Some(byte) = bytes.next() {
        match byte {
            b' ' | b'\t' => {}
            b'\r' => {
                newlines += 1;
                if bytes.peek() == Some(&b'\n') {
                    bytes.next();
                }
            }
            b'\n' => newlines += 1,
            _ => return false,
        }
    }
    newlines == 1
}

fn doc_text(source: &str, comments: &[Comment]) -> (String, bool) {
    let mut text = String::new();
    let mut truncated = false;
    for (index, comment) in comments.iter().enumerate() {
        let end = match comment.kind {
            CommentKind::DocumentationBlock => {
                comment.end.0.saturating_sub(2).max(comment.offset.0 + 3)
            }
            CommentKind::DocumentationLine => comment.end.0,
            CommentKind::Line | CommentKind::Block => continue,
        };
        let content = &source[comment.offset.0 + 3..end];
        match comment.kind {
            CommentKind::DocumentationLine => {
                if index > 0 {
                    append(&mut text, "\n", &mut truncated);
                }
                append(
                    &mut text,
                    content.strip_prefix(' ').unwrap_or(content),
                    &mut truncated,
                );
            }
            CommentKind::DocumentationBlock => {
                let mut lines = content.lines().peekable();
                let mut first = true;
                while let Some(line) = lines.next() {
                    let trimmed = line.trim_start_matches([' ', '\t']);
                    let cleaned = match trimmed.strip_prefix('*') {
                        Some(after) if after.is_empty() || after.starts_with(' ') => {
                            after.strip_prefix(' ').unwrap_or(after)
                        }
                        _ => line,
                    };
                    if (first || lines.peek().is_none()) && cleaned.trim().is_empty() {
                        continue;
                    }
                    if !first {
                        append(&mut text, "\n", &mut truncated);
                    }
                    append(
                        &mut text,
                        if first {
                            cleaned.strip_prefix(' ').unwrap_or(cleaned)
                        } else {
                            cleaned
                        },
                        &mut truncated,
                    );
                    first = false;
                    if truncated {
                        break;
                    }
                }
            }
            CommentKind::Line | CommentKind::Block => {}
        }
        if truncated {
            break;
        }
    }
    (text, truncated)
}

fn append(text: &mut String, value: &str, truncated: &mut bool) {
    let mut end = value.len().min(2048 - text.len());
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    text.push_str(&value[..end]);
    *truncated |= end < value.len();
}
