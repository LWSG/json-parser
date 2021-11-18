mod values;
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use values::Value;

fn traverse_json(v: &Value) -> String {
    let mut s = String::new();
    match v {
        Value::Null => s += "null",
        Value::Bool(b) => s += if *b { "true" } else { "false" },
        Value::Number(n)=>s+=format!("{}", n),
        Value::Str(string) => s += format!("\"{}\"", string),
        _ => unimplemented!(),
    }
    s
}
pub struct Parser<'a> {
    src: Peekable<Chars<'a>>,
}
#[derive(Debug)]
pub enum ParserError {
    UnExpectedEOF,
    UnExpectedToken(String),
}
impl<'a> Parser<'a> {
    pub fn new<'b: 'a>(json: &'b str) -> Self {
        Parser {
            src: json.chars().peekable(),
        }
    }
    pub fn parse(&mut self) -> Result<Value, ParserError> {
        self.skip_whitespace();
        match *self.peek()? {
            't' | 'f' | 'n' => self.parse_true_false_null(),
            '\"' => self.parse_str(),
            '0'..='9' | '-' => self.parse_num(),
            '[' => self.parse_array(),
            '{' => self.parse_obj(),
            _ => Err(ParserError::UnExpectedEOF),
        }
    }
    fn parse_true_false_null(&mut self) -> Result<Value, ParserError> {
        let mut s = String::from(self.next().unwrap());
        while let Ok(ch) = self.peek() {
            if ch.is_ascii_alphabetic() && *ch != ',' && *ch != ']' && *ch != '}' && *ch != ':' {
                s += &ch.to_string();
                self.next()?;
            } else {
                break;
            }
        }
        match s.as_str() {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            "null" => Ok(Value::Null),
            _ => Err(ParserError::UnExpectedToken(s)),
        }
    }
    fn parse_num(&mut self) -> Result<Value, ParserError> {
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
    fn parse_str(&mut self) -> Result<Value, ParserError> {
        self.next()?;
        let mut s = String::new();
        let mut integ_string = false;
        while let Ok(ch) = self.next() {
            match ch {
                '\"' => {
                    integ_string = true;
                    break;
                }
                '\\' => {
                    s.push(self.parse_escaped()?);
                }
                _ => {
                    s.push(ch);
                }
            }
        }
        if integ_string {
            Ok(Value::Str(s))
        } else {
            Err(ParserError::UnExpectedToken(s))
        }
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
            'u' => {
                let mut s = String::new();
                while let Ok(ch) = self.peek() {
                    if ch.is_numeric() || 'A' <= *ch && *ch <= 'F' {
                        s += &ch.to_string();
                    } else {
                        break;
                    }
                    self.next()?;
                }
                let num = i64::from_str_radix(&s[..], 16).unwrap();
                Ok(char::from_u32(num as u32).unwrap())
            }
            ch => Err(ParserError::UnExpectedToken(format!("\\{}", ch).to_owned())),
        }
    }
    fn parse_array(&mut self) -> Result<Value, ParserError> {
        self.next()?;
        self.skip_whitespace();
        let mut v = Vec::new();
        loop {
            v.push(self.parse()?);
            self.skip_whitespace();
            let ch = self.peek()?;
            match *ch {
                ']' => {
                    self.next()?;
                    break;
                }
                ',' => {
                    self.next()?;
                }
                _ => {
                    return Err(ParserError::UnExpectedToken(ch.to_string()));
                }
            }
        }

        Ok(Value::Array(v))
    }
    fn parse_obj(&mut self) -> Result<Value, ParserError> {
        self.next()?;
        self.skip_whitespace();
        let mut m = HashMap::new();
        loop {
            let key = match self.parse_str()? {
                Value::Str(s) => s,
                _ => {
                    panic!();
                }
            };
            self.skip_whitespace();
            let ch = self.next()?;
            let value = match ch {
                ':' => self.parse()?,
                _ => {
                    return Err(ParserError::UnExpectedToken(ch.to_string()));
                }
            };
            let ch = self.peek()?;
            m.insert(key, value);
            match *ch {
                '}' => {
                    self.next()?;
                    break;
                }
                ',' => {
                    self.next()?;
                }
                _ => {
                    return Err(ParserError::UnExpectedToken(ch.to_string()));
                }
            }
        }

        Ok(Value::Object(m))
    }
    fn skip_whitespace(&mut self) {
        while let Ok(&ch) = self.peek() {
            if ch.is_whitespace() {
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
        assert_eq!(Parser::new("true").parse().unwrap(), Value::Bool(true));
        assert_eq!(Parser::new("false").parse().unwrap(), Value::Bool(false));
        assert_eq!(Parser::new("null").parse().unwrap(), Value::Null);

        match Parser::new("flase").parse() {
            Ok(_) => panic!(),
            Err(err) => match err {
                ParserError::UnExpectedToken(s) => assert_eq!(s.as_str(), "flase"),
                _ => panic!(),
            },
        }
        match Parser::new("ture").parse() {
            Ok(_) => panic!(),
            Err(err) => match err {
                ParserError::UnExpectedToken(s) => assert_eq!(s.as_str(), "ture"),
                _ => panic!(),
            },
        }
        match Parser::new("nu ll").parse() {
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
            Parser::new("-12342").parse().unwrap(),
            Value::Number(-12342)
        );
        assert_eq!(
            Parser::new("-1.23E03").parse().unwrap(),
            Value::Float(-1.23E03)
        );
        match Parser::new("0123").parse() {
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
            Parser::new("\"hello\"").parse().unwrap(),
            Value::Str("hello".to_owned())
        );
        match Parser::new("\"hello").parse() {
            Ok(_) => panic!(),
            Err(err) => match err {
                ParserError::UnExpectedToken(s) => assert_eq!(s.as_str(), "hello"),
                _ => panic!(),
            },
        }
        assert_eq!(
            Parser::new("\"hello\n\"").parse().unwrap(),
            Value::Str("hello\n".to_owned())
        );
        assert_eq!(
            Parser::new(r#""\b\f\n\r\t\"""#).parse().unwrap(),
            Value::Str("\u{8}\u{C}\u{A}\u{D}\u{9}\"".to_owned())
        );
        assert_eq!(
            Parser::new(r#""\u2764""#).parse().unwrap(),
            Value::Str("❤".to_owned())
        );
    }
    #[test]
    fn array() {
        assert_eq!(
            Parser::new("  [12,\"89\",true,[false,null]]")
                .parse()
                .unwrap(),
            Value::Array(vec![
                Value::Number(12),
                Value::Str("89".to_owned()),
                Value::Bool(true),
                Value::Array(vec![Value::Bool(false), Value::Null])
            ])
        );
    }
}
