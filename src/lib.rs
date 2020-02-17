extern crate nom;

use std::str::from_utf8;

use nom::IResult;
use nom::character::complete::{char, digit1};
use nom::bytes::complete::take;

use nom::combinator::{map_res, opt, recognize, map};
use nom::sequence::{preceded, terminated, tuple};
use nom::branch::alt;

#[derive(Debug,PartialEq)]
pub enum BValue<'a> {
    BString(&'a str),
    BNumber(i64),
}

fn number(i: &[u8]) -> IResult<&[u8], i64> {
    let signed_digit = recognize(tuple((opt(char('-')), digit1)));
    let parsed_num = map_res(signed_digit, |s: &[u8]| from_utf8(s).unwrap().parse::<i64>());
    terminated(preceded(char('i'), parsed_num), char('e'))(i)
}

fn string_len(i: &[u8]) -> IResult<&[u8], usize> {
    let len = terminated(digit1, char(':'));
    map_res(len, |s: &[u8]| from_utf8(s).unwrap().parse::<usize>())(i)
}

fn string(i: &[u8]) -> IResult<&[u8], &str> {
    let (left, len) = string_len(i)?;
    let u8_res = take(len);
    map_res(u8_res, from_utf8)(left)
}

fn value(i: &[u8]) -> IResult<&[u8], BValue> {
    let bnumber = map(number, BValue::BNumber);
    let bstring = map(string, BValue::BString);
    alt((bnumber, bstring))(i)
}


#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::ErrorKind;
    use nom::Err::Error;


    #[test]
    fn parse_number() {
        assert_eq!(number(&b"i3e"[..]), Ok((&b""[..], 3)));
        assert_eq!(number(&b"i-3e"[..]), Ok((&b""[..], -3)));
    }

    #[test]
    fn parse_string() {
        assert_eq!(string_len(&b"12:Hello World!"[..]), Ok((&b"Hello World!"[..], 12)));
        assert_eq!(string(&b"12:Hello World!"[..]), Ok((&b""[..], "Hello World!")));
        assert_eq!(string(&b"15:Hello World!"[..]), Err(Error((&b"Hello World!"[..], ErrorKind::Eof))));
    }

    #[test]
    fn parse_value() {
        assert_eq!(value(&b"i3e"[..]), Ok((&b""[..], BValue::BNumber(3))));
        assert_eq!(value(&b"12:Hello World!"[..]), Ok((&b""[..], BValue::BString("Hello World!"))));
    }
}
