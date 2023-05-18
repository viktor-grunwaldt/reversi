use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::{map, map_res, opt, recognize, value},
    error::{FromExternalError, ParseError},
    number::complete,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};
use std::io;
pub fn set_up() {
    println!("RDY");
}
#[derive(Clone, Debug, PartialEq)]
pub enum Response {
    UGO(f64, f64),
    HEDID(f64, f64, Option<(usize, usize)>),
    ONEMORE,
    BYE,
    FAIL,
}

fn p_float<'a, E>(i: &'a str) -> IResult<&'a str, f64, E>
where
    E: ParseError<&'a str>,
{
    delimited(multispace0, complete::double, multispace0)(i)
}

fn parse_ugo<'a, E>(i: &'a str) -> IResult<&'a str, Response, E>
where
    E: ParseError<&'a str>,
{
    let parser = preceded(tag("UGO"), pair(p_float, p_float));
    map(parser, |(x, y)| Response::UGO(x, y))(i)
}

fn p_int<'a, E>(i: &'a str) -> IResult<&'a str, i32, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    delimited(
        multispace0,
        map_res(recognize(pair(opt(tag("-")), digit1)), str::parse),
        multispace0,
    )(i)
}
fn convert_move(x: i32, y: i32) -> Option<(usize, usize)> {
    usize::try_from(x).ok().zip(usize::try_from(y).ok())
}

fn parse_hedid<'a, E>(i: &'a str) -> IResult<&'a str, Response, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let parser = preceded(tag("HEDID"), tuple((p_float, p_float, p_int, p_int)));
    map(parser, |(x, y, a, b)| {
        Response::HEDID(x, y, convert_move(a, b))
    })(i)
}
fn parse_response<'a, E>(i: &'a str) -> IResult<&'a str, Response, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    alt((
        value(Response::ONEMORE, tag("ONEMORE")),
        value(Response::BYE, tag("BYE")),
        parse_ugo,
        parse_hedid,
    ))(i)
}
pub fn read_response() -> Response {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Error reading input");
    parse_response::<()>(&input)
        .map(|res| res.1)
        .unwrap_or(Response::FAIL)
}

pub fn send_message(my_move: Option<(usize, usize)>, transpose_board:bool) {
    let (x, y) = match my_move {
        Some((row, col)) => (row as i32, col as i32),
        None => (-1, -1),
    };
    if transpose_board {
        println!("IDO {} {}", y, x);
    }else {
        println!("IDO {} {}", x, y);
    }
}

#[cfg(test)]
mod tests {
    use crate::communication::*;
    #[test]
    fn test_onemore() {
        assert_eq!(parse_response::<()>("ONEMORE"), Ok(("", Response::ONEMORE)));
        assert_eq!(
            parse_response::<()>("ONEMORE-1234"),
            Ok(("-1234", Response::ONEMORE))
        );
    }
    #[test]
    fn test_bye() {
        assert_eq!(parse_response::<()>("BYE"), Ok(("", Response::BYE)));
        assert_eq!(
            parse_response::<()>("BYE-1234"),
            Ok(("-1234", Response::BYE))
        );
    }
    #[test]
    fn test_double() {
        assert_eq!(p_float::<()>("59.6969 "), Ok(("", 59.6969)));
        assert_eq!(p_float::<()>("0 "), Ok(("", 0_f64)));
        assert_eq!(p_float::<()>(""), Err(nom::Err::Error(())));
    }
    #[test]
    fn test_int() {
        assert_eq!(p_int::<()>("59.6969"), Ok((".6969", 59)));
        assert_eq!(p_int::<()>("59 "), Ok(("", 59)));
        assert_eq!(p_int::<()>(" 58 "), Ok(("", 58)));
        assert_eq!(p_int::<()>(" 42 42 "), Ok(("42 ", 42)));
        assert_eq!(p_int::<()>("0"), Ok(("", 0)));
        assert_eq!(p_int::<()>("-1"), Ok(("", -1)));
        assert_eq!(p_int::<()>(""), Err(nom::Err::Error(())));
        assert_eq!(p_int::<()>("BYE"), Err(nom::Err::Error(())));
    }
    #[test]
    fn test_ugo() {
        assert_eq!(
            parse_ugo::<()>("UGO 59.6969 0.0"),
            Ok(("", Response::UGO(59.6969, 0_f64)))
        );
        assert_eq!(parse_ugo::<()>("UGO 59.6969"), Err(nom::Err::Error(())));
        assert_eq!(
            parse_ugo::<()>("UGO 59.25 16.5"),
            Ok(("", Response::UGO(59.25, 16.5)))
        );
        assert_eq!(parse_ugo::<()>("UGO     "), Err(nom::Err::Error(())));
        assert_eq!(parse_ugo::<()>(" UGO     "), Err(nom::Err::Error(())));
        assert_eq!(parse_ugo::<()>(""), Err(nom::Err::Error(())));
    }
    #[test]
    fn test_hedid() {
        assert_eq!(
            parse_hedid::<()>("HEDID 59.25 16.5 0 0"),
            Ok(("", Response::HEDID(59.25, 16.5, Some((0, 0)))))
        );
        assert_eq!(
            parse_hedid::<()>("HEDID 59.25 16.5 -1 -1"),
            Ok(("", Response::HEDID(59.25, 16.5, None)))
        );
        assert_eq!(
            parse_hedid::<()>("UGO 59.6969 0.0"),
            Err(nom::Err::Error(()))
        );
    }
}
