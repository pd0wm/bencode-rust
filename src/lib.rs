extern crate nom;

use std::str::from_utf8;
use std::collections::HashMap;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::take;
use nom::character::complete::{char, digit1};
use nom::combinator::{map_res, opt, recognize, map};
use nom::multi::many1;
use nom::sequence::{preceded, terminated, pair};

#[derive(Debug,PartialEq)]
pub enum BValue {
    BString(String),
    BNumber(i64),
    BList(Vec<BValue>),
    BDict(HashMap<String, BValue>),
}

fn number(i: &[u8]) -> IResult<&[u8], i64> {
    let signed_digit = recognize(pair(opt(char('-')), digit1));
    let parsed_num = map_res(signed_digit, |s: &[u8]| from_utf8(s).unwrap().parse::<i64>());
    terminated(preceded(char('i'), parsed_num), char('e'))(i)
}

fn length(i: &[u8]) -> IResult<&[u8], usize> {
    let len = terminated(digit1, char(':'));
    map_res(len, |s: &[u8]| from_utf8(s).unwrap().parse::<usize>())(i)
}

fn string(i: &[u8]) -> IResult<&[u8], String> {
    let (left, len) = length(i)?;
    let result = take(len);
    let result = map(result, |s: &[u8]| s.to_vec());
    map_res(result, String::from_utf8)(left)
}

fn list(i: &[u8]) -> IResult<&[u8], Vec<BValue>> {
    let values = many1(value);
    preceded(char('l'), terminated(values, char('e')))(i)
}

fn dict(i: &[u8]) -> IResult<&[u8], HashMap<String, BValue>> {
    let kv = pair(string, value);
    let kv = many1(kv);
    let kv = terminated(preceded(char('d'), kv), char('e'));
    map(kv, |s| s.into_iter().collect())(i)
}

fn value(i: &[u8]) -> IResult<&[u8], BValue> {
    let bnumber = map(number, BValue::BNumber);
    let bstring = map(string, BValue::BString);
    let blist = map(list, BValue::BList);
    let bdict = map(dict, BValue::BDict);
    alt((bnumber, bstring, blist, bdict))(i)
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
        assert_eq!(length(&b"12:Hello World!"[..]), Ok((&b"Hello World!"[..], 12)));
        assert_eq!(string(&b"12:Hello World!"[..]), Ok((&b""[..], "Hello World!".to_string())));
        assert_eq!(string(&b"15:Hello World!"[..]), Err(Error((&b"Hello World!"[..], ErrorKind::Eof))));
    }

    #[test]
    fn parse_value() {
        assert_eq!(value(&b"i3e"[..]), Ok((&b""[..], BValue::BNumber(3))));
        assert_eq!(value(&b"12:Hello World!"[..]), Ok((&b""[..], BValue::BString("Hello World!".to_string()))));
    }

    #[test]
    fn parse_list() {
        let expected = BValue::BList(vec![BValue::BString("spam".to_string()), BValue::BString("eggs".to_string())]);
        assert_eq!(value(&b"l4:spam4:eggse"[..]), Ok((&b""[..], expected)));
    }

    #[test]
    fn parse_dict() {
        let mut expected : HashMap<String, BValue> = HashMap::new();
        expected.entry("cow".to_string()).or_insert(BValue::BString("moo".to_string()));
        expected.entry("spam".to_string()).or_insert(BValue::BString("eggs".to_string()));

        assert_eq!(value(&b"d3:cow3:moo4:spam4:eggse"[..]), Ok((&b""[..], BValue::BDict(expected))));
    }
}
