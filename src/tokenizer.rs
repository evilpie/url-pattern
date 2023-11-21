// SPDX-License-Identifier: MIT

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

pub(crate) fn tokenize(input: &str, _policy: Policy) -> Vec<Token> {
    let mut tokens = vec![];

    let mut iter = input.chars().peekable();
    loop {
        let chr = if let Some(chr) = iter.next() {
            chr
        } else {
            break;
        };

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
                        'A'..='Z' | 'a'..='z' => name.push(iter.next().unwrap()),
                        _ => break,
                    }
                }
                tokens.push(Token::Name(name))
            }
            // 8. If tokenizer’s code point is U+0028 (():
            '(' => {
                unimplemented!("RegExp token")
            },
            _ => {
                // TODO
                tokens.push(Token::Char(chr));
            }
        }
    }

    tokens.push(Token::End);

    tokens
}
