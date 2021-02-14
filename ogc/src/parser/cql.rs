use bytes::complete;
use multi::many1;
use nom::{*, character::complete::*, character::*, combinator::*, sequence::*};
use nom::{
branch::alt,
bytes::complete::{escaped, tag, take_while},
character::complete::{alphanumeric1 as alphanumeric, char, one_of, space0},
combinator::{map, opt, value},
number::complete::double,
sequence::{delimited, preceded, separated_pair, terminated},
Err, IResult,
};
use number::complete::{f32, float};
use sequence::tuple;

use std::{any, convert::From, iter::FromIterator};

use std::str;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Filter {}

#[wasm_bindgen]
#[derive(Debug, PartialEq)]
pub struct ArithmeticExpression {
    left_operand: Operand,
    operator: ArithmeticOperator,
    right_operand: Operand,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq)]
pub enum ArithmeticOperator {
    PlusSign, MinusSign, Asterisk, Solidus
}

impl From<char> for ArithmeticOperator {
    ///FIXME try_from
    fn from(item: char) -> Self {
        match item {
            '+' => ArithmeticOperator::PlusSign,
            '-' => ArithmeticOperator::MinusSign,
            '*' => ArithmeticOperator::Asterisk,
            '/' => ArithmeticOperator::Solidus,
            _   => panic!("Invalid input char"),
        }
    }
}

impl From<&str> for ArithmeticOperator {
    ///FIXME try_from
    fn from(item: &str) -> Self {
        match item.chars().nth(0).unwrap() {
            '+' => ArithmeticOperator::PlusSign,
            '-' => ArithmeticOperator::MinusSign,
            '*' => ArithmeticOperator::Asterisk,
            '/' => ArithmeticOperator::Solidus,
            _   => panic!("Invalid input char"),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq)]
pub struct Identifier {
    value: String,
    is_quoted: bool,
}

#[wasm_bindgen]
impl Identifier {
    pub fn new(value: String, is_quoted: bool) -> Self {
        Identifier { value, is_quoted }
    }

    pub fn name(&self) -> String {
        self.value.clone()
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq)]
pub enum Sign {
    Positive,
    Negative,
}

impl From<char> for Sign {
    ///FIXME try_from
    fn from(item: char) -> Self {
        match item {
            '-' => Sign::Negative,
            _   => Sign::Positive,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq)]
pub struct NumericLiteral {
    sign: Sign,
    value: f32,
    //FIXME more fields here
}

#[wasm_bindgen]
#[derive(Debug, PartialEq)]
pub struct Operand {
    literal: Option<NumericLiteral>,
    identifier: Option<Identifier>,
    function: Option<String>,
}

impl Operand {
    fn literal(x: NumericLiteral) -> Self {
        Self {
            literal: Some(x),
            identifier: None,
            function: None,
        }
    }

    fn identifier(x: Identifier) -> Self {
        Self {
            literal: None,
            identifier: Some(x),
            function: None,
        }
    }

    fn function(x: String) -> Self {
        Self {
            literal: None,
            identifier: None,
            function: Some(x),
        }

    }
}

impl From<f32> for Operand {
    fn from(item: f32) -> Self {
        Operand {
            literal: Some(NumericLiteral { sign: Sign::Positive, value: item }),
            identifier: None,
            function: None
        }
    }
}

impl From<NumericLiteral> for Operand {
    fn from(item: NumericLiteral) -> Self {
        Operand {
            literal: Some(item),
            identifier: None,
            function: None
        }
    }
}

impl From<Identifier> for Operand {
    fn from(item: Identifier) -> Self {
        Operand {
            literal: None,
            identifier: Some(item),
            function: None,
        }
    }
}

/// identifierPart = alpha | digit | dollar | underscore;
pub fn identifier_part<'a>(input: &'a str) -> IResult<&'a str, String> {
    map(nom::multi::many1(alt((alpha1, digit1, tag("$"), tag("_")))), |s| s.into_iter().collect())(input)
}

/// Parse the identifier start without quotes
/// 
/// identifierStart [ {colon | period | identifierPart} ]
/// identifierStart = alpha;
/// identifierPart = alpha | digit | dollar | underscore;
pub fn identifier_start<'a>(input: &'a str) -> IResult<&'a str, Identifier> {
    map(many1(alt((
        map(tag(":"), String::from),
        map(tag("."), String::from),
        identifier_part))
    ), |strings| Identifier::new(strings.into_iter().collect(), false))(input)
}

/// Parse a quoted identifier
/// 
/// doubleQuote identifier doubleQuote
pub fn identifier_quoted<'a>(input: &'a str) -> IResult<&'a str, Identifier> {
    map(delimited(tag("\""), nom::multi::many1(identifier), tag("\"")),
        |x| {
            let value: String = concat_values(x);
            Identifier::new(value, true)
    })(input)
}

/// Parse an identifier without quotes
/// 
/// identifier = identifierStart [ {colon | period | identifierPart} ]
/// 
/// identifierStart = alpha;
///
/// identifierPart = alpha | digit | dollar | underscore;
pub fn identifier_unquoted<'a>(input: &'a str) -> IResult<&'a str, Identifier> {
    let parse_start = map(alpha1, String::from); 
    let parse_opt_section = alt((
         map(tag(":"), String::from),
         map(tag("."), String::from),
        identifier_part)); 
    map(many1(alt((parse_start, parse_opt_section))), |x| Identifier::new(x.into_iter().collect(), false))(input)
}

/// Parse an identifier (property name)
/// propertyName = identifier;
/// 
/// identifier = identifierStart [ {colon | period | identifierPart} ] \
///            | doubleQuote identifier doubleQuote;
/// 
/// identifierStart = alpha;
/// 
/// identifierPart = alpha | digit | dollar | underscore;
pub fn identifier<'a>(input: &'a str) -> IResult<&'a str, Identifier> {
    alt((identifier_unquoted, identifier_quoted))(input)
}

/// arithmeticOperator = plusSign | minusSign | asterisk | solidus;
pub fn arithmetic_operator<'a>(input: &'a str) -> IResult<&'a str, ArithmeticOperator> {
    map(alt((one_of("+-*/"), delimited(space0, one_of("+-*/"), space0))),
     ArithmeticOperator::from)(input)
}

/// arithmeticOperand = numericLiteral
///                   | propertyName
///                   | function;
pub fn arithmetic_operand<'a,>(input: &'a str) -> IResult<&'a str, Operand> {
    alt((map(numeric_literal, Operand::from), map(identifier, Operand::from)))(input)
}

/// #=============================================================================#
/// # An arithemtic expression is an expression composed of an arithmetic
/// # operand (a property name, a number or a function that returns a number),
/// # an arithmetic operators (+,-,*,/) and another arithmetic operand.
/// #=============================================================================#
/// arithmeticExpression = arithmeticOperand arithmeticOperator arithmeticOperand
///                      | leftParen arithmeticExpression rightParen;
/// 
/// arithmeticOperator = plusSign | minusSign | asterisk | solidus;
/// 
/// arithmeticOperand = numericLiteral
///                   | propertyName
///                   | function;
pub fn arithmetic_expression<'a>(input: &'a str) -> IResult<&'a str, ArithmeticExpression> {
    map(alt((arithmetic_expression_spaced, delimited(tag("("),arithmetic_expression_spaced, tag(")")))),
    |x| ArithmeticExpression { left_operand: x.0, operator: x.1, right_operand: x.2 })(input)
}

fn arithmetic_expression_spaced<'a>(input: &'a str) -> IResult<&'a str, (Operand, ArithmeticOperator, Operand)> {
    tuple((delimited(space0, arithmetic_operand, space0),
                     delimited(space0, arithmetic_operator, space0),
                     delimited(space0, arithmetic_operand, space0)))(input)
}


/// #=============================================================================#
/// # Definition of a FUNCTION
/// # The functions offered by an implementation are provided at `/functions`
/// #=============================================================================#
/// function = identifier leftParen {argumentList} rightParen;
/// 
/// argumentList = argument [ { comma argument } ];
/// 
/// argument = characterLiteral
///          | numericLiteral
///          | booleanLiteral
///          | spatialLiteral
///          | temporalLiteral
///          | propertyName
///          | function
///          | arithmeticExpression
///          | arrayExpression;
pub fn function<'a>(input: &'a str) -> IResult<&'a str, ArithmeticExpression> {
    todo!("Not implemented")
}

/// argumentList = argument [ { comma argument } ];
pub fn argument_list<'a>(input: &'a str) -> IResult<&'a str, ArithmeticExpression> {
    todo!("Not implemented")
}

/// argument = characterLiteral
///          | numericLiteral
///          | booleanLiteral
///          | spatialLiteral
///          | temporalLiteral
///          | propertyName
///          | function
///          | arithmeticExpression
///          | arrayExpression;
pub fn argument<'a>(input: &'a str) -> IResult<&'a str, ArithmeticExpression> {
    todo!("Not implemented")
}

pub fn character_literal<'a>(input: &'a str) -> IResult<&'a str, ArithmeticExpression> {
    todo!("Not implemented")
}

/// #=============================================================================#
/// # Definition of NUMERIC literals
/// #=============================================================================#
///
/// numericLiteral = unsignedNumericLiteral | signedNumericLiteral;
/// 
/// unsignedNumericLiteral = exactNumericLiteral | approximateNumericLiteral;
/// 
/// signedNumericLiteral = [sign] exactNumericLiteral | approximateNumericLiteral;
/// 
/// exactNumericLiteral = unsignedInteger [ period [ unsignedInteger ] ]
///                       | period unsignedInteger;
/// 
/// approximateNumericLiteral = mantissa "E" exponent;
/// 
/// mantissa = exactNumericLiteral;
/// 
/// exponent = signedInteger;
/// 
/// signedInteger = [ sign ] unsignedInteger;
/// 
/// unsignedInteger = {digit};
/// 
/// sign = plusSign | minusSign;
pub fn numeric_literal<'a>(input: &'a str) -> IResult<&'a str, NumericLiteral> {
    alt((unsigned_numeric_literal, signed_numeric_literal))(input) //TODO approximateNumericLiteral etc
}

/// signedNumericLiteral = [sign] exactNumericLiteral | approximateNumericLiteral;
pub fn signed_numeric_literal<'a>(input: &'a str) -> IResult<&'a str, NumericLiteral> {
    map(pair(one_of("+-"), number::complete::float), 
    |x| NumericLiteral { sign: Sign::from(x.0), value: x.1 })(input)
}

/// unsignedNumericLiteral = exactNumericLiteral | approximateNumericLiteral;
pub fn unsigned_numeric_literal<'a>(input: &'a str) -> IResult<&'a str, NumericLiteral> {
    map(number::complete::float, |x| NumericLiteral { sign: Sign::Positive, value: x.into()})(input)
}

pub fn spatial_literal<'a>(input: &'a str) -> IResult<&'a str, ArithmeticExpression> {
    todo!("Not implemented")
}

pub fn temporal_literal<'a>(input: &'a str) -> IResult<&'a str, ArithmeticExpression> {
    todo!("Not implemented")
}

pub fn array_expression<'a>(input: &'a str) -> IResult<&'a str, ArithmeticExpression> {
    todo!("Not implemented")
}

fn concat_values(x: Vec<Identifier>) -> String {
    x.into_iter().fold("".to_string(), |a, e| {
        a + &e.value
    })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use super::{ArithmeticExpression, ArithmeticOperator, Identifier, NumericLiteral, Operand, Sign, arithmetic_expression, arithmetic_operand, arithmetic_operator, identifier, identifier_start, signed_numeric_literal, unsigned_numeric_literal};

    proptest! {

        #[test]
        fn parse_unsigned_numeric_literal(number in "[0-9]{5}") {
            let r = unsigned_numeric_literal(&number).unwrap();
            assert_eq!(r.0, "");
            assert_eq!(r.1.sign, Sign::Positive);
            assert_eq!(r.1.value, number.parse::<f32>().unwrap());
        }

        #[test]
        fn parse_signed_numeric_literal(number in "[+-][0-9]{5}") {
            let r = signed_numeric_literal(&number).unwrap();
            assert_eq!(r.0, "");
            assert_eq!(r.1.sign, Sign::from(number.chars().nth(0).unwrap()));
            assert_eq!(r.1.value, number.chars().skip(1).take(number.len()).collect::<String>().parse::<f32>().unwrap());
        }

        #[test]
        fn parse_signed_numeric_literal_float(number in "[+-][0-9]{5}\\.[0-9]{5}") {
            let r = signed_numeric_literal(&number).unwrap();
            assert_eq!(r.0, "");
            assert_eq!(r.1.sign, Sign::from(number.chars().nth(0).unwrap()));
            assert_eq!(r.1.value, number.chars().skip(1).take(number.len()).collect::<String>().parse::<f32>().unwrap());
        }

        #[test]
        fn parse_arithmetic_operand(operand in "[a-zA-Z]{5}") {
            let r = arithmetic_operand(&operand).unwrap();
            assert_eq!(r.0, "");
            assert_eq!(r.1.literal, None);
            assert_eq!(r.1.identifier, Some(Identifier::new(operand.clone(), false)));
            assert_eq!(r.1.function, None);
        }

        #[test]
        fn parse_arithmetic_operator(operator in "[-+*/]") {
            let r = arithmetic_operator(&operator).unwrap();
            assert_eq!(r.0, "");
            assert_eq!(r.1, ArithmeticOperator::from(operator.chars().nth(0).unwrap()));

            let op2 = format!(" {} ", operator);
            let r2 = arithmetic_operator(&op2).unwrap();
            assert_eq!(r2.0, "");
            assert_eq!(r2.1, ArithmeticOperator::from(operator.chars().nth(0).unwrap()));
        }

        #[test]
        fn parse_arithmetic(left_operand in "[a-zA-Z]{10}", operator in "[-+*/]", right_operand in "[a-zA-Z]{10}") {
            let example = &format!("{} {} {}", left_operand, operator, right_operand);
            let r = arithmetic_expression(example).unwrap();
            assert_eq!(r.0, "");
            assert_eq!(r.1.left_operand, Operand::identifier(Identifier::new(left_operand.clone(), false)));
            assert_eq!(r.1.right_operand, Operand::identifier(Identifier::new(right_operand.clone(), false)));
            assert_eq!(r.1.operator, ArithmeticOperator::from(operator.clone().pop().unwrap()));

            // Without whitespace
            let example2 = &format!("{}{}{}", left_operand, operator, right_operand);
            let r2 = arithmetic_expression(example2).unwrap();
            assert_eq!(r2.0, "");
            assert_eq!(r.1.left_operand, Operand::identifier(Identifier::new(left_operand.clone(), false)));
            assert_eq!(r.1.right_operand, Operand::identifier(Identifier::new(right_operand.clone(), false)));
            assert_eq!(r2.1.operator, ArithmeticOperator::from(operator.clone().pop().unwrap()));
        }

        #[test]
        fn parse_arithmetic_numerical(left_operand in "[a-zA-Z]{10}", operator in "[-+*/]", right_operand in "[0-9]{10}") {
            let example = &format!("{} {} {}", left_operand, operator, right_operand);
            let r = arithmetic_expression(example).unwrap();
            assert_eq!(r.0, "");
            assert_eq!(r.1.left_operand, Operand::identifier(Identifier::new(left_operand.clone(), false)));
            assert_eq!(r.1.right_operand, Operand::from(right_operand.parse::<f32>().unwrap()));
            assert_eq!(r.1.operator, ArithmeticOperator::from(operator.clone().pop().unwrap()));
        }

        #[test]
        fn parse_arithmetic_paren(left_operand in "[a-zA-Z]{10}", operator in "[-+*/]", right_operand in "[a-zA-Z]{10}") {
            let example = &format!("({} {} {})", left_operand, operator, right_operand);
            let r = arithmetic_expression(example).unwrap();
            assert_eq!(r.0, "");
            assert_eq!(r.1.left_operand, Operand::identifier(Identifier::new(left_operand.clone(), false)));
            assert_eq!(r.1.right_operand, Operand::identifier(Identifier::new(right_operand.clone(), false)));
            assert_eq!(r.1.operator, ArithmeticOperator::from(operator.clone().pop().unwrap()));
        }

        #[test]
        fn parse_identifier_chars_supported_unquoted(ch in "[a-zA-Z][:.]", ch2 in "[0-9][:.]") {
            let r = identifier(&ch).unwrap();
            assert_eq!(r.0, "");
            assert!(!r.1.is_quoted);
            assert_eq!(r.1.value, ch);

            let r2 = identifier(&ch2).unwrap();
            assert_eq!(r2.0, "");
            assert!(!r2.1.is_quoted);
            assert_eq!(r2.1.value, ch2);
        }
    
        #[test]
        fn parse_identifier_start_chars_digits_supported_unquoted(ch in "[a-zA-Z][:.]", ch2 in "[0-9][:.]") {
            let r = identifier_start(&ch).unwrap();
            assert_eq!(r.0, "");
            assert!(!r.1.is_quoted);
            assert_eq!(r.1.value, ch);

            let r2 = identifier_start(&ch2).unwrap();
            assert_eq!(r2.0, "");
            assert!(!r2.1.is_quoted);
            assert_eq!(r2.1.value, ch2);
        }

        #[test]
        fn parse_identifier_unquoted(ch in "[a-zA-Z]{5}[:.][a-zA-Z]{5}", ch2 in "[0-9][:.]") {
            let r = identifier(&ch).unwrap();
            assert_eq!(r.0, "");
            assert!(!r.1.is_quoted);
            assert_eq!(r.1.value, ch);

            let r2 = identifier(&ch2).unwrap();
            assert_eq!(r2.0, "");
            assert!(!r2.1.is_quoted);
            assert_eq!(r2.1.value, ch2);
        }

        #[test]
        fn parse_identifier_quoted(ch in "\"[a-zA-Z]{5}[:.][a-zA-Z]{5}\"", ch2 in "\"[0-9][:.][0-9]\"") {
            let r = identifier(&ch).unwrap();
            assert_eq!(r.0, "");
            assert!(r.1.is_quoted);
            assert_eq!(r.1.value, ch.replace("\\", "").replace("\"", ""));

            let r2 = identifier(&ch2).unwrap();
            assert_eq!(r2.0, "");
            assert!(r2.1.is_quoted);
            assert_eq!(r2.1.value, ch2.replace("\\", "").replace("\"", ""));
        }
    }
  
    #[test]
    fn test_parse_identifier() {
        assert_eq!(Identifier { value: "propertyName".to_string(), is_quoted: false }, identifier("propertyName").unwrap().1);
        assert_eq!(Identifier { value: "propertyName".to_string(), is_quoted: true }, identifier("\"propertyName\"").unwrap().1);
        assert_eq!(Identifier { value: "vehicle_height".to_string(), is_quoted: false }, identifier("vehicle_height").unwrap().1);
        assert_eq!(Identifier { value: "vehicle_height".to_string(), is_quoted: true }, identifier("\"vehicle_height\"").unwrap().1);
    }

    #[test]
    fn test_parse_arithmetic_expression() {
        assert_eq!(
            ArithmeticExpression { 
                left_operand: Operand::from(Identifier::new("speed".to_string(), false)),
                operator: ArithmeticOperator::PlusSign,
                right_operand: Operand::from(Identifier::new("delay".to_string(), false)),
                
            },
            arithmetic_expression("speed + delay").unwrap().1);
        assert_eq!(
            ArithmeticExpression { 
                left_operand: Operand::from(Identifier::new("speed".to_string(), false)),
                operator: ArithmeticOperator::MinusSign,
                right_operand: Operand::from(10f32)
            },
            arithmetic_expression("speed - 10").unwrap().1);
        assert_eq!(
            ArithmeticExpression { 
                left_operand: Operand::from(Identifier::new("speed".to_string(), false)),
                operator: ArithmeticOperator::Asterisk,
                right_operand: Operand::from(10f32)
            },
            arithmetic_expression("speed * 10").unwrap().1);
        assert_eq!(
            ArithmeticExpression { 
                left_operand: Operand::from(Identifier::new("speed".to_string(), false)),
                operator: ArithmeticOperator::Solidus,
                right_operand: Operand::from(10f32)
            },
            arithmetic_expression("speed / 10").unwrap().1);
    }
    
    #[test]
    fn parse_arithmetic_numerical_example() {
        let example = "AAAAAAAaaa - 0000000000";
        let r = arithmetic_expression(example).unwrap();
        assert_eq!(r.0, "");
            assert_eq!(r.1.left_operand, Operand::identifier(Identifier::new("AAAAAAAaaa".to_string(), false)));
            assert_eq!(r.1.right_operand, Operand::literal(NumericLiteral { value: "0000000000".parse().unwrap(), sign: Sign::Positive }));
            assert_eq!(r.1.operator, ArithmeticOperator::from("-"));
    }
}