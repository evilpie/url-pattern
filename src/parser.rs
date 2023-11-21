// SPDX-License-Identifier: MIT

use crate::tokenizer::Token;
use crate::Options;
use std::fmt;

#[derive(Debug)]
pub(crate) enum Modifier {
    Optional,
    ZeroOrMore,
    OneOrMore,
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Modifier::Optional => "?",
                Modifier::ZeroOrMore => "*",
                Modifier::OneOrMore => "+",
            }
        )
    }
}

// https://urlpattern.spec.whatwg.org/#part
#[derive(Debug)]
pub(crate) enum Part {
    FixedText {
        value: String,
        modifier: Option<Modifier>,
    },
    RegExp {
        value: String,
    },
    SegmentWildcard {
        name: String,
        modifier: Option<Modifier>,
        prefix: String,
        suffix: String,
    },
    FullWildcard {
        name: String,
        modifier: Option<Modifier>,
        prefix: String,
        suffix: String,
    },
}

pub(crate) struct Parser<'a> {
    tokens: &'a [Token],
    options: &'a Options,
    index: usize,
    pub(crate) pending_fixed_value: String,
    pub(crate) parts: Vec<Part>,
}

impl<'a> Parser<'_> {
    pub(crate) fn new(tokens: &'a [Token], options: &'a Options) -> Parser<'a> {
        Parser {
            tokens,
            options,
            index: 0,
            pending_fixed_value: String::new(),
            parts: vec![],
        }
    }

    // https://urlpattern.spec.whatwg.org/#parse-a-pattern-string
    pub(crate) fn parse(&mut self) {
        loop {
            // 1. Let char token be the result of running try to consume a token given parser and "char".
            let char_token = self.try_consume_token(|token| matches!(token, Token::Char(_)));

            // 2. Let name token be the result of running try to consume a token given parser and "name".
            let name_token = self.try_consume_token(|token| matches!(token, Token::Name(_)));

            // 3. Let regexp or wildcard token be the result of running try to consume a regexp or wildcard token given parser and name token.
            let regexp_or_wildcard = self.try_consume_regexp_or_wildcard(name_token.as_ref());

            // 4. If name token is not null or regexp or wildcard token is not null:
            if name_token.is_some() || regexp_or_wildcard.is_some() {
                // 1. Let prefix be the empty string.
                // 2. If char token is not null then set prefix to char token’s value.
                let mut prefix: String = match char_token {
                    Some(Token::Char(chr)) => chr.into(),
                    Some(_) => panic!("unexpected token {:?}", char_token),
                    None => "".into(),
                };
                // 3. If prefix is not the empty string and not options’s prefix code point:
                if !prefix.is_empty() {
                    if let Some(opt_prefix) = self.options.prefix {
                        if prefix != opt_prefix.to_string() {
                            // 1. Append prefix to the end of parser’s pending fixed value.
                            self.pending_fixed_value.push_str(&prefix);

                            // 2. Set prefix to the empty string.
                            prefix.clear()
                        }
                    }
                }

                // Run maybe add a part from the pending fixed value given parser.
                self.maybe_add_part_from_pending_fixed_value();

                // Let modifier token be the result of running try to consume a modifier token given parser.
                let modifier = self.try_consume_modifier();

                // Run add a part given parser, prefix, name token, regexp or wildcard token, the empty string, and modifier token.
                self.add_part(prefix, name_token, regexp_or_wildcard, "".into(), modifier);

                // Continue
                continue;
            }

            // Let fixed token be char token.
            // If fixed token is null, then set fixed token to the result of running try to consume a token given parser and "escaped-char".
            let fixed_token = char_token
                .or_else(|| self.try_consume_token(|token| matches!(token, Token::EscapedChar(_))));

            // If fixed token is not null:
            if let Some(fixed_token) = fixed_token {
                let value = if let Token::Char(value) | Token::EscapedChar(value) = fixed_token {
                    value
                } else {
                    panic!("impossible");
                };

                // Append fixed token’s value to parser’s pending fixed value.
                self.pending_fixed_value.push(value);

                // Continue.
                continue;
            }

            // 8. Let open token be the result of running try to consume a token given parser and "open".
            // 9. If open token is not null:
            if let Some(_) = self.try_consume_token(|token| matches!(token, Token::Open)) {
                // Set prefix be the result of running consume text given parser.
                let prefix = self.consume_text();

                // Set name token to the result of running try to consume a token given parser and "name".
                let name = self.try_consume_token(|token| matches!(token, Token::Name(_)));

                // Set regexp or wildcard token to the result of running try to consume a regexp or wildcard token given parser and name token.
                let regexp_or_wildcard = self.try_consume_regexp_or_wildcard(name.as_ref());

                // Let suffix be the result of running consume text given parser.
                let suffix = self.consume_text();

                // Run consume a required token given parser and "close".
                if let None = self.try_consume_token(|token| matches!(token, Token::Close)) {
                    panic!("missing close token")
                }

                // Set modifier token to the result of running try to consume a modifier token given parser.
                let modifier = self.try_consume_modifier();

                // Run add a part given parser, prefix, name token, regexp or wildcard token, suffix, and modifier token.
                self.add_part(prefix, name, regexp_or_wildcard, suffix, modifier);

                // Continue.
                continue;
            }

            // Run maybe add a part from the pending fixed value given parser.
            self.maybe_add_part_from_pending_fixed_value();

            // Run consume a required token given parser and "end".
            if let None = self.try_consume_token(|token| matches!(token, Token::End)) {
                panic!("expected end");
            }

            break;
        }
    }

    fn consume_text(&mut self) -> String {
        let mut result = String::new();
        loop {
            match self.tokens[self.index] {
                Token::Char(chr) | Token::EscapedChar(chr) => {
                    result.push(chr);
                    self.index += 1;
                }
                _ => return result,
            }
        }
    }

    fn try_consume_token(&mut self, matches: fn(&Token) -> bool) -> Option<Token> {
        let next_token = &self.tokens[self.index];
        if !matches(next_token) {
            return None;
        }
        self.index += 1;
        Some(next_token.clone())
    }

    fn try_consume_modifier(&mut self) -> Option<Modifier> {
        let modifier = match self.tokens[self.index] {
            Token::QuestionMark => Modifier::Optional,
            Token::Plus => Modifier::OneOrMore,
            Token::Asterisk => Modifier::ZeroOrMore,
            _ => return None,
        };

        self.index += 1;
        Some(modifier)
    }

    // https://urlpattern.spec.whatwg.org/#try-to-consume-a-regexp-or-wildcard-token
    fn try_consume_regexp_or_wildcard(&mut self, name_token: Option<&Token>) -> Option<Token> {
        // 1. Let token be the result of running try to consume a token given parser and "regexp".
        let token = self.try_consume_token(|token| matches!(token, Token::RegExp(_)));

        // 2. If name token is null and token is null, then set token to the result of running try to consume a token given parser and "asterisk".
        if name_token.is_none() && token.is_none() {
            return self.try_consume_token(|token| matches!(token, Token::Asterisk));
        }

        // 3. Return token.
        token
    }

    /// https://urlpattern.spec.whatwg.org/#add-a-part
    fn add_part(
        &mut self,
        prefix: String,
        name: Option<Token>,
        regexp_or_wildcard: Option<Token>,
        suffix: String,
        modifier: Option<Modifier>,
    ) {
        // 1. Let modifier be "none".
        // 2. If modifier token is not null:
        // ...
        // NOTE: Implemented by try_consume_modifier

        // 3. If name token is null and regexp or wildcard token is null and modifier is "none":
        if name.is_none() && regexp_or_wildcard.is_none() && modifier.is_none() {
            // Note: This was a "{foo}" grouping
            assert!(suffix.is_empty());

            // Append prefix to the end of parser’s pending fixed value.
            self.pending_fixed_value.push_str(&prefix);

            // Return
            return;
        }

        // 4. Run maybe add a part from the pending fixed value given parser.
        self.maybe_add_part_from_pending_fixed_value();

        // 5. If name token is null and regexp or wildcard token is null:
        if name.is_none() && regexp_or_wildcard.is_none() {
            // Assert: suffix is the empty string.
            assert!(suffix.is_empty());

            // If prefix is the empty string, then return.
            if prefix.is_empty() {
                return;
            }

            // Let encoded value be the result of running parser’s encoding callback given prefix.
            // TODO:
            let encoded_value = prefix.clone();

            // Let part be a new part whose type is "fixed-text", value is encoded value, and modifier is modifier.
            // Append part to parser’s part list.
            self.parts.push(Part::FixedText {
                value: encoded_value,
                modifier,
            });

            // Return.
            return;
        }

        // 6. Let regexp value be the empty string.
        let regexp_value = match regexp_or_wildcard {
            // 7. If regexp or wildcard token is null, then set regexp value to parser’s segment wildcard regexp.
            None => "<segment wildcard>".into(),
            // 8. Otherwise if regexp or wildcard token’s type is "asterisk", then set regexp value to the full wildcard regexp value.
            Some(Token::Asterisk) => "<full wildcard>".into(),
            // 9. Otherwise set regexp value to regexp or wildcard token’s value.
            Some(Token::RegExp(ref value)) => value.clone(),
            Some(_) => panic!("invalid regexp_or_wildcard token"),
        };

        // 10. Let type be "regexp".
        // 11. If regexp value is parser’s segment wildcard regexp:
        //     1. Set type to "segment-wildcard".
        // 12. Otherwise if regexp value is the full wildcard regexp value:

        // 13. Let name be the empty string.
        // 14. If name token is not null, then set name to name token’s value.
        let name = if name.is_some() {
            match name {
                Some(Token::Name(name)) => name.clone(),
                _ => panic!("invalid name token"),
            }
        } else {
            // 15. Otherwise if regexp or wildcard token is not null:
            // XXX: Spec bug must be non-null.
            assert!(regexp_or_wildcard.is_some());
            "1".into()
        };

        // 16. If the result of running is a duplicate name given parser and name is true, then throw a TypeError.
        // TODO:

        // 17. Let encoded prefix be the result of running parser’s encoding callback given prefix.
        // 18. Let encoded suffix be the result of running parser’s encoding callback given suffix.

        // Let part be a new part whose type is type, value is regexp value, modifier is modifier, name is name, prefix is encoded prefix, and suffix is encoded suffix.
        if regexp_value == "<segment wildcard>" {
            self.parts.push(Part::SegmentWildcard {
                name,
                modifier,
                prefix,
                suffix,
            })
        } else if regexp_value == "<full wildcard>" {
            self.parts.push(Part::FullWildcard {
                name,
                modifier,
                prefix,
                suffix,
            })
        }
    }

    fn maybe_add_part_from_pending_fixed_value(&mut self) {
        // 1. If parser’s pending fixed value is the empty string, then return.
        if self.pending_fixed_value.is_empty() {
            return;
        }

        // 2. Let encoded value be the result of running parser’s encoding callback given parser’s pending fixed value.
        // TODO
        let encoded_value = self.pending_fixed_value.clone();

        // 3. Set parser’s pending fixed value to the empty string.
        self.pending_fixed_value.clear();

        // 4. Let part be a new part whose type is "fixed-text", value is encoded value, and modifier is "none".
        // 5. Append part to parser’s part list.
        self.parts.push(Part::FixedText {
            value: encoded_value,
            modifier: None,
        });
    }
}
