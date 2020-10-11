use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_until, take_while_m_n};
use nom::character::complete::{digit1, multispace0, space0};
use nom::combinator::map_res;
use nom::multi::{fold_many1, many0};
use nom::IResult;
use rgb::RGB;

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

// TXT format parsing
fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn from_dec(input: &str) -> Result<isize, std::num::ParseIntError> {
    isize::from_str_radix(input, 10)
}

fn hex_tuple(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

fn hex_color(input: &str) -> IResult<&str, RGB<u8>> {
    let (input, red) = hex_tuple(input)?;
    let (input, green) = hex_tuple(input)?;
    let (input, blue) = hex_tuple(input)?;
    Ok((input, RGB::new(red, green, blue)))
}

fn number(input: &str) -> IResult<&str, isize> {
    let (input, n) = fold_many1(alt((tag("-"), digit1)), String::new(), |acc, item| {
        acc + item
    })(input)?;
    let n = from_dec(&n).unwrap();
    Ok((input, n))
}

fn line(input: &str) -> IResult<&str, ([isize; 3], RGB<u8>)> {
    let (input, _) = many0(comment)(input)?;
    let (input, _) = space0(input)?;
    let (input, x) = number(input)?;
    let (input, _) = space0(input)?;
    let (input, z) = number(input)?;
    let (input, _) = space0(input)?;
    let (input, y) = number(input)?;
    let (input, _) = space0(input)?;
    let (input, color) = hex_color(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, ([x, y * -1, z], color)))
}

fn comment(input: &str) -> IResult<&str, ()> {
    let (input, _) = space0(input)?;
    let (input, _) = tag("#")(input)?;
    let (input, _) = take_until("\n")(input)?;
    let (input, _) = tag("\n")(input)?;
    Ok((input, ()))
}

pub fn import_txt(input: &str) -> Vec<(RGB<u8>, [isize; 3])> {
    let (input, lines) = many0(line)(input).unwrap();
    let mut data = vec![];
    for line in lines {
        data.push((line.1, line.0));
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FILE: &'static str = r#"#This is a comment
    10 20 30 abcdef #end of line comment
    11 -21 31 012345

    12 22 32 6789AB"#;

    #[test]
    fn it_parses() {
        let s = import_txt(TEST_FILE);

        assert_eq!(s[0], (RGB::new(0xab, 0xcd, 0xef), [10, 20, 30]));
        assert_eq!(s[1], (RGB::new(0x01, 0x23, 0x45), [11, -21, 31],));
        assert_eq!(s[2], (RGB::new(0x67, 0x89, 0xab), [12, 22, 32],));
        assert_eq!(s.len(), 3);
    }
}
