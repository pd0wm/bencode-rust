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
    BBytes(Vec<u8>),
    BNumber(i64),
    BList(Vec<BValue>),
    BDict(HashMap<String, BValue>),
}

impl BValue {
    pub fn get_list(&self) -> &Vec<BValue> {
        match self {
            BValue::BList(list) => list,
            _ => panic!("Failed to get dict"),
        }
    }

    pub fn get_dict(&self) -> &HashMap<String, BValue> {
        match self {
            BValue::BDict(dict) => dict,
            _ => panic!("Failed to get dict"),
        }
    }

    pub fn get_number(&self) -> &i64 {
        match self {
            BValue::BNumber(n) => n,
            _ => panic!("Failed to get number"),
        }
    }

    pub fn get_bytes(&self) -> &Vec<u8> {
        match self {
            BValue::BBytes(bytes) => bytes,
            _ => panic!("Failed to get bytes"),
        }
    }

    pub fn get_string(&self) -> &str {
        from_utf8(self.get_bytes()).unwrap()
    }
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
    map_res(parse_bytes, String::from_utf8)(i)
}

fn parse_bytes(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (left, len) = parse_length(i)?;
    let result = take(len);
    map(result, |s: &[u8]| s.to_vec())(left)
}

fn parse_list(i: &[u8]) -> IResult<&[u8], Vec<BValue>> {
    let values = many1(parse);
    preceded(char('l'), terminated(values, char('e')))(i)
}

fn parse_dict(i: &[u8]) -> IResult<&[u8], HashMap<String, BValue>> {
    let kv = pair(parse_string, parse);
    let kv = many1(kv);
    let kv = terminated(preceded(char('d'), kv), char('e'));
    map(kv, |s| s.into_iter().collect())(i)
}

pub fn parse(i: &[u8]) -> IResult<&[u8], BValue> {
    let bnumber = map(parse_number, BValue::BNumber);
    let bbytes = map(parse_bytes, BValue::BBytes);
    let blist = map(parse_list, BValue::BList);
    let bdict = map(parse_dict, BValue::BDict);
    alt((bnumber, bbytes, blist, bdict))(i)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse(&b"i3e"[..]), Ok((&b""[..], BValue::BNumber(3))));
        assert_eq!(parse(&b"i-3e"[..]), Ok((&b""[..], BValue::BNumber(-3))));
    }

    #[test]
    fn test_parse_bytes() {
        assert_eq!(parse(&b"12:Hello World!"[..]), Ok((&b""[..], BValue::BBytes("Hello World!".as_bytes().to_vec()))));
    }

    #[test]
    fn test_parse_list() {
        let expected = BValue::BList(vec![BValue::BBytes("spam".as_bytes().to_vec()), BValue::BBytes("eggs".as_bytes().to_vec())]);
        assert_eq!(parse(&b"l4:spam4:eggse"[..]), Ok((&b""[..], expected)));
    }

    #[test]
    fn test_parse_dict() {
        let mut expected : HashMap<String, BValue> = HashMap::new();
        expected.entry("cow".to_string()).or_insert(BValue::BBytes("moo".as_bytes().to_vec()));
        expected.entry("spam".to_string()).or_insert(BValue::BBytes("eggs".as_bytes().to_vec()));

        assert_eq!(parse(&b"d3:cow3:moo4:spam4:eggse"[..]), Ok((&b""[..], BValue::BDict(expected))));
    }
}
