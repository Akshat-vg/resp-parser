#[derive(Debug, PartialEq)]
pub enum RespValue<'input> {
    SimpleString(&'input str),
    SimpleError(&'input str),
    Integer(i64),
    BulkString(Option<&'input str>),
    Arrays(Option<Vec<RespValue<'input>>>),
}

pub struct Parser<'input> {
    input: &'input str,
    pos: usize,
}

impl<'input> Parser<'input> {
    pub fn new(input: &'input str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn parse(&mut self) -> RespValue<'input> {
        let byte = self.input.as_bytes().get(self.pos).unwrap();
        self.pos += 1;
        match byte {
            b'+' => self.simple_string(),
            b'-' => self.simple_error(),
            b':' => self.integer(),
            b'$' => self.bulk_string(),
            b'*' => self.array(),
            _ => {
                panic!("not yet impl or unrecognized bytecode: {byte}");
            }
        }
    }

    fn simple_string(&mut self) -> RespValue<'input> {
        RespValue::SimpleString(self.line())
    }

    fn simple_error(&mut self) -> RespValue<'input> {
        RespValue::SimpleError(self.line())
    }

    fn integer(&mut self) -> RespValue<'input> {
        RespValue::Integer(Parser::to_int(self.line()))
    }

    fn bulk_string(&mut self) -> RespValue<'input> {
        let bulk_str_len = Parser::to_int(self.line());

        match bulk_str_len {
            ..0 => RespValue::BulkString(None),
            0 => {
                self.pos += 2;
                RespValue::BulkString(Some(""))
            }
            _ => {
                let start = self.pos;
                let end = start + bulk_str_len as usize;
                self.pos = end + 2;
                RespValue::BulkString(Some(&self.input[start..end]))
            }
        }
    }

    fn array(&mut self) -> RespValue<'input> {
        let item_count = match Parser::to_int(self.line()) {
            n if n > 0 => n as usize,
            0 => return RespValue::Arrays(Some(vec![])),
            _ => return RespValue::Arrays(None),
        };
        let items: Vec<RespValue<'input>> = (0..item_count).map(|_| self.parse()).collect();
        RespValue::Arrays(Some(items))
    }

    fn to_int(num: &str) -> i64 {
        num.parse::<i64>().unwrap()
    }

    fn line(&mut self) -> &'input str {
        let start = self.pos;
        let len = self.input.as_bytes()[start..]
            .windows(2)
            .position(|clrf| clrf == b"\r\n")
            .unwrap();
        let end = start + len;
        self.pos = end + 2;
        &self.input[start..end]
    }
}

#[cfg(test)]
mod test {
    use crate::{Parser, RespValue};
    fn assert_fully_consumed(p: &Parser<'_>, input: &str) {
        assert_eq!(
            p.pos(),
            input.len(),
            "Parser stopped at byte {} of {}",
            p.pos(),
            input.len()
        );
    }

    #[test]
    fn simple_string() {
        let s = "+OK\r\n";
        let mut p = Parser::new(s);
        let x = p.parse();
        assert_eq!(x, RespValue::SimpleString("OK"));
        assert_fully_consumed(&p, s);
    }

    #[test]
    fn simple_error() {
        let s = "-Error message\r\n";
        let mut p = Parser::new(s);
        let x = p.parse();
        assert_eq!(x, RespValue::SimpleError("Error message"));
        assert_fully_consumed(&p, s);
    }

    #[test]
    fn integer() {
        let s = ":42\r\n";
        let mut p = Parser::new(s);
        let x = p.parse();
        assert_eq!(x, RespValue::Integer(42));
        assert_fully_consumed(&p, s);
    }

    #[test]
    fn bulk_string() {
        let s = "$5\r\nhello\r\n";
        let mut p = Parser::new(s);
        let x = p.parse();
        assert_eq!(x, RespValue::BulkString(Some("hello")));
        assert_fully_consumed(&p, s);
    }

    #[test]
    fn empty_bulk_string() {
        let s = "$0\r\n\r\n";
        let mut p = Parser::new(s);
        let x = p.parse();
        assert_eq!(x, RespValue::BulkString(Some("")));
        assert_fully_consumed(&p, s);
    }

    #[test]
    fn null_bulk_string() {
        let s = "$-1\r\n";
        let mut p = Parser::new(s);
        let x = p.parse();
        assert_eq!(x, RespValue::BulkString(None));
        assert_fully_consumed(&p, s);
    }

    #[test]
    fn simple_array() {
        let s = "*3\r\n:1\r\n:2\r\n:3\r\n";
        let mut p = Parser::new(s);
        let x = p.parse();
        assert_eq!(
            x,
            RespValue::Arrays(Some(vec![
                RespValue::Integer(1),
                RespValue::Integer(2),
                RespValue::Integer(3),
            ]))
        );
        assert_fully_consumed(&p, s);
    }

    #[test]
    fn recursive_array() {
        let s = "*3\r\n:1\r\n*2\r\n+hello\r\n$5\r\nworld\r\n*-1\r\n";
        let mut p = Parser::new(s);
        let x = p.parse();
        assert_eq!(
            x,
            RespValue::Arrays(Some(vec![
                RespValue::Integer(1),
                RespValue::Arrays(Some(vec![
                    RespValue::SimpleString("hello"),
                    RespValue::BulkString(Some("world")),
                ])),
                RespValue::Arrays(None),
            ]))
        );
        assert_fully_consumed(&p, s);
    }

    #[test]
    fn empty_array() {
        let s = "*0\r\n";
        let mut p = Parser::new(s);
        let x = p.parse();
        assert_eq!(x, RespValue::Arrays(Some(vec![])));
        assert_fully_consumed(&p, s);
    }
}
