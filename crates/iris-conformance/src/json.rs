use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub enum Value {
    Array(Vec<Value>),
    Bool,
    Null,
    Number,
    Object(BTreeMap<String, Value>),
    String(String),
}

pub fn parse(input: &str) -> Result<Value, String> {
    let mut parser = Parser {
        input: input.as_bytes(),
        cursor: 0,
    };
    let value = parser.value()?;
    parser.space();
    if parser.cursor == input.len() {
        Ok(value)
    } else {
        Err("JSON trailing input".into())
    }
}

struct Parser<'a> {
    input: &'a [u8],
    cursor: usize,
}

impl Parser<'_> {
    fn value(&mut self) -> Result<Value, String> {
        self.space();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(Value::String),
            Some(b't') => self.word(b"true", Value::Bool),
            Some(b'f') => self.word(b"false", Value::Bool),
            Some(b'n') => self.word(b"null", Value::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err("JSON value expected".into()),
        }
    }
    fn object(&mut self) -> Result<Value, String> {
        self.byte(b'{')?;
        let mut values = BTreeMap::new();
        self.space();
        while self.peek() != Some(b'}') {
            let key = self.string()?;
            self.byte(b':')?;
            let value = self.value()?;
            if values.insert(key, value).is_some() {
                return Err("duplicate JSON key".into());
            }
            self.space();
            if self.peek() != Some(b'}') {
                self.byte(b',')?;
            }
            self.space();
        }
        self.byte(b'}')?;
        Ok(Value::Object(values))
    }
    fn array(&mut self) -> Result<Value, String> {
        self.byte(b'[')?;
        let mut values = Vec::new();
        self.space();
        while self.peek() != Some(b']') {
            values.push(self.value()?);
            self.space();
            if self.peek() != Some(b']') {
                self.byte(b',')?;
            }
            self.space();
        }
        self.byte(b']')?;
        Ok(Value::Array(values))
    }
    fn string(&mut self) -> Result<String, String> {
        self.byte(b'"')?;
        let mut result = String::new();
        loop {
            match self.next().ok_or("unterminated JSON string")? {
                b'"' => return Ok(result),
                b'\\' => result.push(self.escape()?),
                byte if byte >= 0x20 => result.push(char::from(byte)),
                _ => return Err("invalid JSON string".into()),
            }
        }
    }
    fn escape(&mut self) -> Result<char, String> {
        match self.next().ok_or("short JSON escape")? {
            b'"' => Ok('"'),
            b'\\' => Ok('\\'),
            b'/' => Ok('/'),
            b'b' => Ok('\u{8}'),
            b'f' => Ok('\u{c}'),
            b'n' => Ok('\n'),
            b'r' => Ok('\r'),
            b't' => Ok('\t'),
            b'u' => {
                let end = self.cursor + 4;
                let digits = self
                    .input
                    .get(self.cursor..end)
                    .ok_or("short JSON unicode")?;
                self.cursor = end;
                let hex = std::str::from_utf8(digits).map_err(|_| "bad JSON unicode")?;
                char::from_u32(u32::from(
                    u16::from_str_radix(hex, 16).map_err(|_| "bad JSON unicode")?,
                ))
                .ok_or("bad JSON scalar".into())
            }
            _ => Err("bad JSON escape".into()),
        }
    }
    fn number(&mut self) -> Result<Value, String> {
        while matches!(
            self.peek(),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            self.cursor += 1;
        }
        Ok(Value::Number)
    }
    fn word(&mut self, word: &[u8], value: Value) -> Result<Value, String> {
        if self
            .input
            .get(self.cursor..)
            .is_some_and(|tail| tail.starts_with(word))
        {
            self.cursor += word.len();
            Ok(value)
        } else {
            Err("invalid JSON word".into())
        }
    }
    fn byte(&mut self, expected: u8) -> Result<(), String> {
        self.space();
        if self.next() == Some(expected) {
            Ok(())
        } else {
            Err("unexpected JSON byte".into())
        }
    }
    fn space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.cursor += 1;
        }
    }
    fn peek(&self) -> Option<u8> {
        self.input.get(self.cursor).copied()
    }
    fn next(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.cursor += 1;
        Some(byte)
    }
}
