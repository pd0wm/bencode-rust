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

fn parse_number(i: &[u8]) -> IResult<&[u8], i64> {
    let signed_digit = recognize(pair(opt(char('-')), digit1));
    let parsed_num = map_res(signed_digit, |s: &[u8]| from_utf8(s).unwrap().parse::<i64>());
    terminated(preceded(char('i'), parsed_num), char('e'))(i)
}

fn parse_length(i: &[u8]) -> IResult<&[u8], usize> {
    let len = terminated(digit1, char(':'));
    map_res(len, |s: &[u8]| from_utf8(s).unwrap().parse::<usize>())(i)
}

fn parse_string(i: &[u8]) -> IResult<&[u8], String> {
    let (left, len) = parse_length(i)?;
    let result = take(len);
    let result = map(result, |s: &[u8]| s.to_vec());
    map_res(result, String::from_utf8)(left)
}

fn parse_list(i: &[u8]) -> IResult<&[u8], Vec<BValue>> {
    let values = many1(parse_value);
    preceded(char('l'), terminated(values, char('e')))(i)
}

fn parse_dict(i: &[u8]) -> IResult<&[u8], HashMap<String, BValue>> {
    let kv = pair(parse_string, parse_value);
    let kv = many1(kv);
    let kv = terminated(preceded(char('d'), kv), char('e'));
    map(kv, |s| s.into_iter().collect())(i)
}

pub fn parse_value(i: &[u8]) -> IResult<&[u8], BValue> {
    let bnumber = map(parse_number, BValue::BNumber);
    let bstring = map(parse_string, BValue::BString);
    let blist = map(parse_list, BValue::BList);
    let bdict = map(parse_dict, BValue::BDict);
    alt((bnumber, bstring, blist, bdict))(i)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_value(&b"i3e"[..]), Ok((&b""[..], BValue::BNumber(3))));
        assert_eq!(parse_value(&b"i-3e"[..]), Ok((&b""[..], BValue::BNumber(-3))));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_value(&b"12:Hello World!"[..]), Ok((&b""[..], BValue::BString("Hello World!".to_string()))));
    }

    #[test]
    fn test_parse_list() {
        let expected = BValue::BList(vec![BValue::BString("spam".to_string()), BValue::BString("eggs".to_string())]);
        assert_eq!(parse_value(&b"l4:spam4:eggse"[..]), Ok((&b""[..], expected)));
    }

    #[test]
    fn test_parse_dict() {
        let mut expected : HashMap<String, BValue> = HashMap::new();
        expected.entry("cow".to_string()).or_insert(BValue::BString("moo".to_string()));
        expected.entry("spam".to_string()).or_insert(BValue::BString("eggs".to_string()));

        assert_eq!(parse_value(&b"d3:cow3:moo4:spam4:eggse"[..]), Ok((&b""[..], BValue::BDict(expected))));
    }
}
