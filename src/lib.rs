// SPDX-License-Identifier: MIT

/*!
url-pattern is a VERY INCOMPLETE implementation of the WHATWG [URL Pattern](https://urlpattern.spec.whatwg.org/) standard.
Seriously **DON'T USE THIS** (yet)!
*/

mod parser;
mod tokenizer;

use crate::parser::{Modifier, Parser, Part};
use crate::tokenizer::{tokenize, Policy};

/// <https://urlpattern.spec.whatwg.org/#options>
#[derive(Default, Clone)]
pub struct Options {
    pub delimiter: Option<char>,
    pub prefix: Option<char>,
    pub ignore_case: bool,
}

/// https://urlpattern.spec.whatwg.org/#generate-a-segment-wildcard-regexp
fn generate_segment_wildcard_regexp(opts: &Options) -> String {
    format!(
        "[^{}]+?",
        escape_regexp(
            &opts
                .delimiter
                .map_or_else(|| String::new(), |chr| chr.to_string())
        )
    )
}

/// https://urlpattern.spec.whatwg.org/#full-wildcard-regexp-value
fn full_wildcard_regexp() -> &'static str {
    ".*"
}

fn escape_regexp(str: &str) -> String {
    // TODO:
    str.replace("/", "\\/")
}

/// https://urlpattern.spec.whatwg.org/#generate-a-regular-expression-and-name-list
fn generate_regexp(parts: &[Part], opts: &Options) -> String {
    let mut result: String = "^".into();

    for part in parts {
        let (_, modifier, regexp, prefix, suffix) = match part {
            // If part’s type is "fixed-text":
            Part::FixedText { value, modifier } => {
                result.push_str(
                    if let Some(modifier) = modifier {
                        format!("(?:{}){modifier}", &escape_regexp(value))
                    } else {
                        escape_regexp(value)
                    }
                    .as_ref(),
                );
                continue;
            }
            Part::SegmentWildcard {
                name,
                modifier,
                prefix,
                suffix,
            } => (
                name,
                modifier,
                generate_segment_wildcard_regexp(opts),
                escape_regexp(prefix),
                escape_regexp(suffix),
            ),
            Part::FullWildcard {
                name,
                modifier,
                prefix,
                suffix,
            } => (
                name,
                modifier,
                full_wildcard_regexp().into(),
                escape_regexp(prefix),
                escape_regexp(suffix),
            ),
            Part::RegExp { .. } => todo!(),
        };

        // If part’s prefix is the empty string and part’s suffix is the empty string:
        // ..
        if prefix.is_empty() && suffix.is_empty() {
            result.push_str(
                match modifier {
                    None => format!("({regexp})"),
                    Some(modifier @ Modifier::Optional) => format!("({regexp}){modifier}"),
                    Some(modifier) => format!("((?:{regexp}){modifier})"),
                }
                .as_ref(),
            );
            continue;
        }

        result.push_str(
            match modifier {
                None => format!("(?:{prefix}({regexp}){suffix})"),
                Some(modifier @ Modifier::Optional) => {
                    format!("(?:{prefix}({regexp}){suffix}){modifier}")
                }
                Some(Modifier::ZeroOrMore) => {
                    format!("(?:{prefix}((?:{regexp})(?:{suffix}{prefix}(?:{regexp}))*){suffix})?")
                }
                Some(Modifier::OneOrMore) => {
                    format!("(?:{prefix}((?:{regexp})(?:{suffix}{prefix}(?:{regexp}))*){suffix})")
                }
            }
            .as_ref(),
        );
    }

    result.push('$');

    result
}

/// Parses a pattern string and returns a regular expression for matching that
/// pattern.
pub fn regexp_for_pattern(input: &str, options: &Options) -> String {
    let tokens = tokenize(input, Policy::Strict);

    let mut parser = Parser::new(&tokens, options);
    parser.parse();

    generate_regexp(&parser.parts, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Verify that of all these results are correct!!

    #[test]
    fn smoke_test() {
        let opts = Options {
            delimiter: Some('/'),
            prefix: Some('/'),
            ignore_case: false,
        };

        let result = regexp_for_pattern("abc", &opts);
        assert_eq!(result, "^abc$");

        let result = regexp_for_pattern("{foo}", &opts);
        assert_eq!(result, "^foo$");

        let result = regexp_for_pattern("{bar}?", &opts);
        assert_eq!(result, "^(?:bar)?$");

        let result = regexp_for_pattern("/:bar", &opts);
        assert_eq!(result, r"^(?:\/([^\/]+?))$");

        let result = regexp_for_pattern("/:foo/:bar", &opts);
        assert_eq!(result, r"^(?:\/([^\/]+?))(?:\/([^\/]+?))$");

        let result = regexp_for_pattern("/:foo/:bar?", &opts);
        assert_eq!(result, r"^(?:\/([^\/]+?))(?:\/([^\/]+?))?$");

        let result = regexp_for_pattern("/:foo?/:bar?", &opts);
        assert_eq!(result, r"^(?:\/([^\/]+?))?(?:\/([^\/]+?))?$");

        let result = regexp_for_pattern("/:foo?/:bar", &opts);
        assert_eq!(result, r"^(?:\/([^\/]+?))?(?:\/([^\/]+?))$");
    }

    #[test]
    fn parse_example_1() {
        // From https://urlpattern.spec.whatwg.org/#parse-example-1
        let opts = Options {
            delimiter: Some('/'),
            prefix: Some('/'),
            ignore_case: false,
        };

        // assert_eq!(regexp_for_pattern("/:foo(bar)?", &opts), "bla");
        assert_eq!(regexp_for_pattern("/", &opts), r"^\/$");
        assert_eq!(regexp_for_pattern(":foo", &opts), r"^([^\/]+?)$");
        // assert_eq!(regexp_for_pattern("(bar)", &opts), "bla");
        assert_eq!(regexp_for_pattern("/:foo", &opts), r"^(?:\/([^\/]+?))$");
        // assert_eq!(regexp_for_pattern("/(bar)", &opts), "bla");
        assert_eq!(regexp_for_pattern("/:foo?", &opts), r"^(?:\/([^\/]+?))?$");
        // assert_eq!(regexp_for_pattern("/(bar)?", &opts), "bla");
    }

    #[test]
    fn parse_example_2() {
        // From https://urlpattern.spec.whatwg.org/#parsing-example-2
        let opts = Options {
            delimiter: Some('/'),
            prefix: Some('/'),
            ignore_case: false,
        };

        // assert_eq!(regexp_for_pattern("{a:foo(bar)b}?", &opts), r"^(?:a([^\/]+?)(bar)b)?$");
        assert_eq!(regexp_for_pattern("{:foo}?", &opts), r"^([^\/]+?)?$");
        // assert_eq!(regexp_for_pattern("{(bar)}?", &opts), "bla");
        assert_eq!(regexp_for_pattern("{ab}?", &opts), r"^(?:ab)?$");
    }
}
