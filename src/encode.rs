use crate::resptype::RespValue;

pub fn encode(value: RespValue<'_>) -> String {
    match value {
        RespValue::SimpleString(simple_string) => format!("+{}\r\n", simple_string),
        RespValue::SimpleError(simple_error) => format!("-{}\r\n", simple_error),
        RespValue::Integer(integer) => format!(":{}\r\n", integer),
        RespValue::BulkString(bulk_string) => {
            if let Some(string) = bulk_string {
                format!("${}\r\n{}\r\n", string.chars().count(), string)
            } else {
                "$-1\r\n".to_string()
            }
        }
        RespValue::Arrays(resp_array) => {
            let mut output = String::new();
            if let Some(values) = resp_array {
                output.push_str(&format!("*{}\r\n", values.len()));
                for value in values {
                    output += &encode(value);
                }
            } else {
                output.push_str("*-1\r\n");
            }
            output
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{encode::encode, resptype::RespValue};

    #[test]
    fn encode_simple_string() {
        let expected = "+OK\r\n";
        let value = RespValue::SimpleString("OK");
        assert_eq!(encode(value), expected);
    }

    #[test]
    fn encode_simple_error() {
        let expected = "-Error message\r\n";
        let value = RespValue::SimpleError("Error message");
        assert_eq!(encode(value), expected);
    }

    #[test]
    fn encode_integer() {
        let expected = ":42\r\n";
        let value = RespValue::Integer(42);
        assert_eq!(encode(value), expected);
    }

    #[test]
    fn encode_bulk_string() {
        let expected = "$5\r\nhello\r\n";
        let value = RespValue::BulkString(Some("hello"));
        assert_eq!(encode(value), expected);
    }

    #[test]
    fn encode_empty_bulk_string() {
        let expected = "$0\r\n\r\n";
        let value = RespValue::BulkString(Some(""));
        assert_eq!(encode(value), expected);
    }

    #[test]
    fn encode_null_bulk_string() {
        let expected = "$-1\r\n";
        let value = RespValue::BulkString(None);
        assert_eq!(encode(value), expected);
    }

    #[test]
    fn encode_simple_array() {
        let expected = "*3\r\n:1\r\n:2\r\n:3\r\n";
        let value = RespValue::Arrays(Some(vec![
            RespValue::Integer(1),
            RespValue::Integer(2),
            RespValue::Integer(3),
        ]));
        assert_eq!(encode(value), expected);
    }

    #[test]
    fn encode_recursive_array() {
        let expected = "*3\r\n:1\r\n*2\r\n+hello\r\n$5\r\nworld\r\n*-1\r\n";
        let value = RespValue::Arrays(Some(vec![
            RespValue::Integer(1),
            RespValue::Arrays(Some(vec![
                RespValue::SimpleString("hello"),
                RespValue::BulkString(Some("world")),
            ])),
            RespValue::Arrays(None),
        ]));
        assert_eq!(encode(value), expected);
    }

    #[test]
    fn encode_empty_array() {
        let expected = "*0\r\n";
        let value = RespValue::Arrays(Some(vec![]));
        assert_eq!(encode(value), expected);
    }

    #[test]
    fn encode_null_array() {
        let expected = "*-1\r\n";
        let value = RespValue::Arrays(None);
        assert_eq!(encode(value), expected);
    }
}
