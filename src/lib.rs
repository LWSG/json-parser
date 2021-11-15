mod values;
use values::Value;

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

pub struct Parser<'a> {
    src: Peekable<Chars<'a>>,
    pos: usize,
    len: usize,
}
#[derive(Debug)]
pub enum ParserError {
    UnExpectedEOF,
    UnExpectedToken(String),
    ExpectedChar(char),
}
impl<'a> Parser<'a> {
    pub fn new<'b: 'a>(json: &'b str) -> Self {
        Parser {
            pos: 0,
            len: json.len(),
            src: json.chars().peekable(),
        }
    }
    pub fn parse(&mut self) -> Result<Value, ParserError> {
        Err(ParserError::UnExpectedEOF)
    }
    fn parse_escaped(&mut self) -> Result<char, ParserError> {
        match self.next()? {
            '\"' => Ok('\"'),
            '\\' => Ok('\\'),
            '/' => Ok('/'),
            'b' => Ok('\u{8}'),
            'f' => Ok('\u{C}'),
            'n' => Ok('\u{A}'),
            'r' => Ok('\u{D}'),
            't' => Ok('\u{9}'),
            ch => Err(ParserError::UnExpectedToken(format!("\\{}", ch).to_owned())),
        }
    }
    fn parse_value(&mut self) -> Result<Value, ParserError> {
        match *self.peek()? {
            't' | 'f' | 'n' => {
                let mut s = String::from(self.next().unwrap());
                while let Ok(ch) = self.next() {
                    if ch.is_ascii_alphabetic() {
                        s += &ch.to_string();
                    } else {
                        break;
                    }
                }
                print!("{}", s);
                match s.as_str() {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    "null" => Ok(Value::Null),
                    _ => Err(ParserError::UnExpectedToken(s)),
                }
            }
            '\"' => {
                self.next()?;
                let mut s = String::new();
                let mut integ_string = false;
                while let Ok(ch) = self.next() {
                    if ch == '\"' {
                        integ_string = true;
                        break;
                    } else {
                        s.push(ch);
                    }
                }
                if integ_string {
                    Ok(Value::Str(s))
                } else {
                    Err(ParserError::ExpectedChar('\"'))
                }
            }
            '0'..='9' | '-' => {
                let is_zero = *self.peek()? == '0';
                let mut s = String::from(self.next()?);

                let mut is_float = false;
                while let Ok(ch) = self.peek() {
                    if *ch == '.' || *ch == 'e' || *ch == 'E' {
                        s.push(self.next()?);
                        is_float = true;
                    } else if ch.is_numeric() {
                        s.push(self.next()?);
                    } else {
                        break;
                    }
                }
                if is_float {
                    match s.parse() {
                        Ok(f) => {
                            if is_zero && f != 0.0 {
                                Err(ParserError::UnExpectedToken(s))
                            } else {
                                Ok(Value::Float(f))
                            }
                        }
                        Err(_) => Err(ParserError::UnExpectedToken(s)),
                    }
                } else {
                    match s.parse() {
                        Ok(f) => {
                            if is_zero && f != 0 {
                                Err(ParserError::UnExpectedToken(s))
                            } else {
                                Ok(Value::Number(f))
                            }
                        }
                        _ => Err(ParserError::UnExpectedToken(s)),
                    }
                }
            }
            _ => Err(ParserError::UnExpectedEOF),
        }
    }
    fn skip_whitespace(&mut self) {
        while let Ok(&ch) = self.peek() {
            if ch == ' ' || ch == '\n' || ch == '\t' {
                self.next().unwrap();
            } else {
                break;
            }
        }
    }
    fn peek(&mut self) -> Result<&char, ParserError> {
        self.src.peek().ok_or(ParserError::UnExpectedEOF)
    }
    fn next(&mut self) -> Result<char, ParserError> {
        self.src.next().ok_or(ParserError::UnExpectedEOF)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_peek_next() {
        let mut parser = Parser::new("He");
        assert_eq!(*parser.peek().unwrap(), 'H');
        assert_eq!(parser.next().unwrap(), 'H');
        assert_eq!(*parser.peek().unwrap(), 'e');
        assert_eq!(parser.next().unwrap(), 'e');

        match parser.peek() {
            Ok(_) => {
                panic!()
            }
            Err(err) => match err {
                ParserError::UnExpectedEOF => {}
                _ => panic!(),
            },
        };
        match parser.next() {
            Ok(_) => {
                panic!()
            }
            Err(err) => match err {
                ParserError::UnExpectedEOF => {}
                _ => panic!(),
            },
        };
    }
    #[test]
    fn skip_whitespace() {
        let mut parser = Parser::new("\t\n    w");
        parser.skip_whitespace();
        assert_eq!(parser.next().unwrap(), 'w');
    }
    #[test]
    fn bool_null() {
        assert_eq!(
            Parser::new("true").parse_value().unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            Parser::new("false").parse_value().unwrap(),
            Value::Bool(false)
        );
        assert_eq!(Parser::new("null").parse_value().unwrap(), Value::Null);

        match Parser::new("flase").parse_value() {
            Ok(_) => panic!(),
            Err(err) => match err {
                ParserError::UnExpectedToken(s) => assert_eq!(s.as_str(), "flase"),
                _ => panic!(),
            },
        }
        match Parser::new("ture").parse_value() {
            Ok(_) => panic!(),
            Err(err) => match err {
                ParserError::UnExpectedToken(s) => assert_eq!(s.as_str(), "ture"),
                _ => panic!(),
            },
        }
        match Parser::new("nu ll").parse_value() {
            Ok(_) => panic!(),
            Err(err) => match err {
                ParserError::UnExpectedToken(s) => assert_eq!(s.as_str(), "nu"),
                _ => panic!(),
            },
        }
    }
    #[test]
    fn number_float() {
        assert_eq!(
            Parser::new("-12342").parse_value().unwrap(),
            Value::Number(-12342)
        );
        assert_eq!(
            Parser::new("-1.23E03").parse_value().unwrap(),
            Value::Float(-1.23E03)
        );
        match Parser::new("0123").parse_value() {
            Ok(_) => panic!(),
            Err(err) => match err {
                ParserError::UnExpectedToken(s) => assert_eq!("0123", s.as_str()),
                _ => panic!(),
            },
        }
    }
    #[test]
    fn string() {
        assert_eq!(
            Parser::new("\"hello\"").parse_value().unwrap(),
            Value::Str("hello".to_owned())
        );
        match Parser::new("\"hello").parse_value() {
            Ok(_) => panic!(),
            Err(err) => match err {
                ParserError::ExpectedChar(c) => assert_eq!(c, '\"'),
                _ => panic!(),
            },
        }
        assert_eq!(
            Parser::new("\"hello\n\"").parse_value().unwrap(),
            Value::Str("hello\n".to_owned())
        );
    }
}
