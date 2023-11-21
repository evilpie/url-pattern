// SPDX-License-Identifier: MIT

use crate::ParseError;

/// https://urlpattern.spec.whatwg.org/#token
#[derive(Clone, Debug)]
pub(crate) enum Token {
    Open,
    Close,
    RegExp(String),
    Name(String),
    Char(char),
    EscapedChar(char),
    Plus,         // "other-modifier"
    QuestionMark, // "other-modifier"
    Asterisk,
    End,
    InvalidChar,
}

pub(crate) enum Policy {
    Strict,
    _Lenient,
}

pub(crate) fn tokenize(input: &str, _policy: Policy) -> Result<Vec<Token>, ParseError> {
    let mut tokens = vec![];

    let mut iter = input.chars().peekable();
    while let Some(chr) = iter.next() {
        match chr {
            // If tokenizer’s code point is U+002A (*):
            '*' => {
                // Run add a token with default position and length given tokenizer and "asterisk".
                tokens.push(Token::Asterisk);
            }
            // If tokenizer’s code point is U+002B (+) or U+003F (?):
            '+' => {
                // Run add a token with default position and length given tokenizer and "other-modifier".
                tokens.push(Token::Plus);
            }
            '?' => {
                // Run add a token with default position and length given tokenizer and "other-modifier".
                tokens.push(Token::QuestionMark);
            }
            // If tokenizer’s code point is U+005C (\):
            '\\' => {
                unimplemented!("EscapedChar");
            }
            // If tokenizer’s code point is U+007B ({):
            '{' => {
                // Run add a token with default position and length given tokenizer and "open".
                tokens.push(Token::Open);
            }
            // If tokenizer’s code point is U+007D (}):
            '}' => {
                // Run add a token with default position and length given tokenizer and "close".
                tokens.push(Token::Close)
            }
            // If tokenizer’s code point is U+003A (:):
            ':' => {
                let mut name = String::new();
                while let Some(chr) = iter.peek() {
                    match chr {
                        // TODO: correct this.
                        'A'..='Z' | 'a'..='z' => name.push(iter.next().unwrap()),
                        _ => break,
                    }
                }
                tokens.push(Token::Name(name))
            }
            // 8. If tokenizer’s code point is U+0028 (():
            '(' => {
                let mut depth = 1;

                // TODO: this is totally not implemented.

                let mut regexp = String::new();
                while let Some(chr) = iter.next() {
                    if !chr.is_ascii() {
                        todo!();
                    }

                    // todo 3,4

                    match chr {
                        '(' => {
                            depth += 1
                            // TODO!
                        }
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                // Set regexp position to tokenizer’s next index.???
                                break;
                            }
                        }
                        _ => {}
                    }

                    regexp.push(chr);
                }

                if depth != 0 {
                    return Err(ParseError::ParenthesesMissmatch);
                }

                tokens.push(Token::RegExp(regexp))
            }
            _ => {
                // TODO
                tokens.push(Token::Char(chr));
            }
        }
    }

    tokens.push(Token::End);
    Ok(tokens)
}
