use nom::bytes::streaming::{is_not, take};
use nom::character::streaming::{char, crlf, digit1};
use nom::combinator::map_res;
use nom::number::streaming::double;
use nom::sequence::{terminated, preceded};
use nom::IResult;
use std::str::{from_utf8, FromStr};

#[inline(always)]
pub fn parse_str_with_prefix(input: &[u8], prefix: char) -> IResult<&[u8], &str> {
    map_res(preceded(char(prefix), terminated(is_not("\r\n"), crlf)), |out| {
        from_utf8(out)
    })(input)
}

#[inline(always)]
pub fn parse_integer_with_prefix(input: &[u8], prefix: char) -> IResult<&[u8], usize> {
    map_res(preceded(char(prefix), terminated(digit1, crlf)), |out| {
        usize::from_str(from_utf8(out).unwrap())
    })(input)
}

#[inline(always)]
pub fn parse_double_with_prefix(input: &[u8], prefix: char) -> IResult<&[u8], f64> {
    preceded(char(prefix), terminated(double, crlf))(input)
}

#[inline(always)]
pub fn parse_bytes_with_length(input: &[u8], length: usize) -> IResult<&[u8], &[u8]> {
    terminated(take(length), crlf)(input)
}

#[inline(always)]
pub fn parse_str_with_length(input: &[u8], length: usize) -> IResult<&[u8], &str> {
    terminated(map_res(take(length), from_utf8), crlf)(input)
}
