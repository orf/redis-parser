use nom::bytes::streaming::{is_not, take};
use nom::character::streaming::{char, crlf, digit1};
use nom::combinator::map_res;
use nom::number::streaming::double;
use nom::sequence::{delimited, terminated};
use nom::IResult;
use std::str::{from_utf8, FromStr};

pub fn parse_str_with_prefix(input: &[u8], prefix: char) -> IResult<&[u8], &str> {
    map_res(delimited(char(prefix), is_not("\r\n"), crlf), |out| {
        from_utf8(out)
    })(input)
}

pub fn parse_integer_with_prefix(input: &[u8], prefix: char) -> IResult<&[u8], usize> {
    map_res(delimited(char(prefix), digit1, crlf), |out| {
        usize::from_str(from_utf8(out).unwrap())
    })(input)
}

pub fn parse_double_with_prefix(input: &[u8], prefix: char) -> IResult<&[u8], f64> {
    delimited(char(prefix), double, crlf)(input)
}

pub fn parse_bytes_with_length(input: &[u8], length: usize) -> IResult<&[u8], &[u8]> {
    terminated(take(length), crlf)(input)
}

pub fn parse_str_with_length(input: &[u8], length: usize) -> IResult<&[u8], &str> {
    // terminated(map_res!(input, take!($size), $crate::lib::std::str::from_utf8), crlf)(input)
    terminated(map_res(take(length), from_utf8), crlf)(input)
}
