use multi::many1;
use nom::{*, character::complete::*, character::*, combinator::*, sequence::*};
use nom::{
branch::alt,
bytes::complete::{escaped, tag, take_while},
character::complete::{one_of, space0},
combinator::{map,},
sequence::{delimited, preceded, separated_pair, terminated},
Err, IResult,
};
use number::complete::{f32, float};
use sequence::tuple;
use serde::{Deserialize, Serialize,};

use std::{any, convert::From, iter::FromIterator};

use std::str;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Filter {}

#[wasm_bindgen]
#[derive(Debug, Deserialize, PartialEq, Serialize,)]
pub struct Argument {
    #[serde(skip_serializing_if = "Option::is_none")]
    character_literal: Option<char>,
    #[serde(skip_serializing_if = "Option::is_none")]
    numeric_literal: Option<NumericLiteral>,
    #[serde(skip_serializing_if = "Option::is_none")]
    property_name: Option<Identifier>,

//FIXME do all of these
// argument = characterLiteral
//          | numericLiteral
//          | booleanLiteral
//          | spatialLiteral
//          | temporalLiteral
//          | propertyName
//          | function
//          | arithmeticExpression
//          | arrayExpression;

}

#[wasm_bindgen]
impl Argument {
    fn character(item: char) -> Self {
        Self {
            character_literal: Some(item),
            numeric_literal: None,
            property_name: None
        }
    }

    fn numeric(item: NumericLiteral) -> Self {
        Self {
            character_literal: None,
            numeric_literal: Some(item),
            property_name: None
        }
    }

    fn property(item: Identifier) -> Self {
        Self {
            character_literal: None,
            numeric_literal: None,
            property_name: Some(item),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ArithmeticExpression {
    left_operand: Operand,
    operator: ArithmeticOperator,
    right_operand: Operand,
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Function {
    name: String,
    args: Vec<Argument>,
}

impl Function {
    pub fn new(name: String, args: Vec<Argument>) -> Self { Self { name, args } }
}


#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Sign {
    Positive,
    Negative,
}

impl From<f32> for Sign {
    ///FIXME try_from
    fn from(item: f32) -> Self {
        println!("From f32: {} {}", item, item < 0f32);
        if item < 0f32 {
            Sign::Negative
        } else {
            Sign::Positive
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NumericLiteral {
    sign: Sign,
    value: f32,
    //FIXME more fields here
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Operand {
    #[serde(skip_serializing_if = "Option::is_none")]
    literal: Option<NumericLiteral>,
    #[serde(skip_serializing_if = "Option::is_none")]
    identifier: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function: Option<Function>,
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

    fn function(x: Function) -> Self {
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
            literal: Some(NumericLiteral { sign: if item >= 0.0 { Sign::Positive } else { Sign::Negative }, value: item }),
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

impl From<Function> for Operand {
    fn from(item: Function) -> Self {
        Operand {
            literal: None,
            identifier: None,
            function: Some(item),
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
    alt((map(numeric_literal, Operand::from), 
         map(identifier, Operand::from),
         map(function, Operand::from)))(input)
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
    // map(arithmetic_expression_spaced,
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
pub fn function<'a>(input: &'a str) -> IResult<&'a str, Function> {
    map(tuple((alpha1, delimited(tag("("),argument_list, tag(")")))),
        |x| Function::new(x.0.to_string(), x.1))(input)
}

/// argumentList = argument [ { comma argument } ];
pub fn argument_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Argument>> {
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
    println!("Numeric literal {}", input);
    signed_numeric_literal(input) //TODO approximateNumericLiteral etc
}

/// signedNumericLiteral = [sign] exactNumericLiteral | approximateNumericLiteral;
pub fn signed_numeric_literal<'a>(input: &'a str) -> IResult<&'a str, NumericLiteral> {
    println!("Signed num lit {}", input);
    // let wrapped = (tag("-+"), space0);
    map(alt((number::complete::float, preceded(satisfy(|c| c == '+' || c == '-'), number::complete::float))),
     |x| NumericLiteral { sign: Sign::from(x), value: x })(input)
}

// /// unsignedNumericLiteral = exactNumericLiteral | approximateNumericLiteral;
// pub fn unsigned_numeric_literal<'a>(input: &'a str) -> IResult<&'a str, NumericLiteral> {
//     println!("Unsigned num lit {}", input);
//     map(number::complete::float, |x| NumericLiteral { sign: Sign::Positive, value: x.into()})(input)
// }

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
    use super::{Argument, ArithmeticExpression, ArithmeticOperator, Function, Identifier, NumericLiteral, Operand, Sign,
         arithmetic_expression, arithmetic_expression_spaced, arithmetic_operand, arithmetic_operator, identifier, identifier_start, signed_numeric_literal};

    proptest! {

        #[test]
        fn parse_signed_numeric_literal(num: f32) {
            let number = &num.to_string();
            let r = signed_numeric_literal(number).unwrap();
            assert_eq!(r.0, "");
            assert_eq!(r.1.sign, if num < 0.0 { Sign::Negative } else { Sign::Positive });
            assert_eq!(r.1.value, num);
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
        fn parse_arithmetic_operand_function(function in "[a-zA-Z]{5}", num1: f32, num2: f32) {
            let ex = format!("{}({},{})", function, num1, num2);
            let r = arithmetic_operand(&ex).unwrap();
            assert_eq!(r.0, "");
            assert_eq!(r.1.literal, None);
            assert_eq!(r.1.identifier, None);
            assert_eq!(r.1.function, Some(
                 Function {
                    name: function, 
                    args: vec![Argument::numeric(NumericLiteral { value: num1, sign: Sign::from(num1) }),
                               Argument::numeric(NumericLiteral { value: num2, sign: Sign::from(num2) })] 
                }));
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
        fn parse_arithmetic_numeric(left_operand in "[a-zA-Z]{10}", operator in "[-+*/]", right_operand: f32) {
            let example = &format!("{} {} {}", left_operand, operator, right_operand);
            println!("Example numeric 1 is {}", example);
            let r = arithmetic_expression(example).unwrap();
            assert_eq!(r.0, "");
            assert_eq!(r.1.left_operand, Operand::identifier(Identifier::new(left_operand.clone(), false)));
            assert_eq!(r.1.right_operand, Operand::from(right_operand));
            assert_eq!(r.1.operator, ArithmeticOperator::from(operator.clone().pop().unwrap()));

            let example2 = &format!("{} {} {}", right_operand, operator, left_operand);
            println!("Example numeric 2 is {}", example2);
            let r2 = arithmetic_expression(example2).unwrap();
            assert_eq!(r2.0, "");
            assert_eq!(r2.1.right_operand, Operand::identifier(Identifier::new(left_operand.clone(), false)));
            assert_eq!(r2.1.left_operand, Operand::from(right_operand));
            assert_eq!(r2.1.operator, ArithmeticOperator::from(operator.clone().pop().unwrap()));
        }

        #[test]
        fn parse_arithmetic_spaced(left_operand in "[a-zA-Z]{10}", operator in "[-+*/]", right_operand: f32) {
            //FIXME unsigned too?
            let example = &format!("{} {} {}", left_operand, operator, right_operand);
            println!("Example is {}", example);
            let r = arithmetic_expression_spaced(example).unwrap();
            assert_eq!(r.0, "");
            assert_eq!(r.1.0, Operand::identifier(Identifier::new(left_operand.clone(), false)));
            assert_eq!(r.1.2, Operand::from(right_operand));
            assert_eq!(r.1.1, ArithmeticOperator::from(operator.clone().pop().unwrap()));

            let example2 = &format!("{} {} {}", right_operand, operator, left_operand);
            println!("Example is {}", example2);
            let r2 = arithmetic_expression_spaced(example2).unwrap();
            assert_eq!(r2.0, "");
            assert_eq!(r2.1.0, Operand::from(right_operand));
            assert_eq!(r2.1.2, Operand::identifier(Identifier::new(left_operand.clone(), false)));
            assert_eq!(r2.1.1, ArithmeticOperator::from(operator.clone().pop().unwrap()));
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
        fn parse_identifier_quoted(ch in "\"[a-zA-Z]{5}[:.][a-zA-Z]{5}\"", num1: u32, ch2 in "\"[:.]\"", num2: u32) {
            let ex1 = format!("{}{}{}", num1, ch, num2);
            let r = identifier(&ex1).unwrap();
            assert_eq!(r.0, "");
            assert!(r.1.is_quoted);
            assert_eq!(r.1.value, ch.replace("\\", "").replace("\"", ""));

            let ex2 = format!("{}{}{}", num1, ch2, num2);
            let r2 = identifier(&ex2).unwrap();
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
    fn parse_arithmetic_numeric_example() {
        let example = "AAAAAAAaaa - -282120334891901200000000000000000000000000000000000000000000000000000000000000000000.0";
        let r = arithmetic_expression(example).unwrap();
        assert_eq!(r.0, "");
        assert_eq!(r.1.left_operand, Operand::identifier(Identifier::new("AAAAAAAaaa".to_string(), false)));
        assert_eq!(r.1.operator, ArithmeticOperator::from("-"));
        assert_eq!(r.1.right_operand, Operand::literal(NumericLiteral { value: -f32::INFINITY, sign: Sign::Negative }));
    }
}