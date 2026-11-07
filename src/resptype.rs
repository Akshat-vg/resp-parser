#[derive(Debug, PartialEq)]
pub enum RespValue<'input> {
    SimpleString(&'input str),
    SimpleError(&'input str),
    Integer(i64),
    BulkString(Option<&'input str>),
    Arrays(Option<Vec<RespValue<'input>>>),
}
