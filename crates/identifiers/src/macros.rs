//! The one code generator this crate publishes: [`wire_string_newtype!`].

/// Joins the `///` lines captured by [`wire_string_newtype!`] into a JSON Schema `description`.
///
/// Public only because the macro expands inside other crates. Not part of the wire contract.
#[doc(hidden)]
#[must_use]
pub fn doc_description(lines: &[&str]) -> String {
    let mut out = String::new();
    for line in lines {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(line.strip_prefix(' ').unwrap_or(line));
    }
    out.trim().to_owned()
}

/// Generates a validated, wire-stable string newtype.
///
/// Produces: a private-field tuple struct; `PATTERN`, `MAX_LEN`, `EXAMPLES` consts;
/// `parse`, `as_str`; `Display`, `FromStr`, `TryFrom<String>`, `From<T> for String`;
/// `Serialize`/`Deserialize` via `try_from`/`into`; and a `JsonSchema` emitting
/// `{"type":"string","pattern":PATTERN,"maxLength":MAX_LEN,"examples":EXAMPLES,
///   "title":<name>,"description":<the doc comment>}`.
///
/// The compiled validator is `regex::Regex::new(PATTERN)` behind a `std::sync::LazyLock`, so the
/// published `pattern` and the runtime check are literally the same string (spec D19).
///
/// `MAX_LEN` is measured in UTF-8 bytes and is a redundant guard: every `PATTERN` also bounds the
/// length. It exists so an over-long input reports [`IdentifierError::TooLong`] instead of an
/// unhelpful pattern mismatch, and so the regex never runs on an unbounded input.
///
/// [`IdentifierError::TooLong`]: crate::IdentifierError::TooLong
///
/// # Example
///
/// ```
/// ratatoskr_identifiers::wire_string_newtype! {
///     /// A lowercase dotted routing key.
///     pub struct RoutingKey {
///         pattern  = r"^[a-z]+(\.[a-z]+)*$",
///         max_len  = 64,
///         examples = ["content.document"],
///     }
/// }
/// assert!(RoutingKey::parse("content.document").is_ok());
/// assert!(RoutingKey::parse("Content.Document").is_err());
/// ```
#[macro_export]
macro_rules! wire_string_newtype {
    (
        $(#[doc = $doc:literal])*
        pub struct $name:ident {
            pattern  = $pattern:expr,
            max_len  = $max_len:expr,
            examples = [$($example:expr),* $(,)?],
        }
    ) => {
        $(#[doc = $doc])*
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(::std::string::String);

        impl $name {
            /// The published JSON Schema `pattern`. The runtime validator compiles this exact
            /// string, so the schema and the parser cannot drift.
            pub const PATTERN: &'static str = $pattern;

            /// Contract maximum length in UTF-8 bytes, published as JSON Schema `maxLength`.
            pub const MAX_LEN: usize = $max_len;

            /// Synthetic example values published as JSON Schema `examples`.
            pub const EXAMPLES: &'static [&'static str] = &[$($example),*];

            /// The `///` lines of this type, published as the JSON Schema `description`.
            const DOC_LINES: &'static [&'static str] = &[$($doc),*];

            /// The published `PATTERN`, compiled once.
            #[allow(
                clippy::expect_used,
                reason = "PATTERN is a compile-time contract constant; a build whose pattern \
                          does not compile is broken before it reaches a wire"
            )]
            fn compiled_pattern() -> &'static ::regex::Regex {
                static COMPILED: ::std::sync::LazyLock<::regex::Regex> =
                    ::std::sync::LazyLock::new(|| {
                        ::regex::Regex::new($name::PATTERN)
                            .expect("contract PATTERN must be a valid regular expression")
                    });
                &COMPILED
            }

            /// Validates `raw` against the contract and wraps it.
            ///
            /// # Errors
            ///
            /// `IdentifierError::Empty` for the empty string, `IdentifierError::TooLong` beyond
            /// `MAX_LEN` UTF-8 bytes, `IdentifierError::PatternMismatch` when `PATTERN` does not
            /// match.
            pub fn parse(raw: &str) -> ::core::result::Result<Self, $crate::IdentifierError> {
                if raw.is_empty() {
                    return ::core::result::Result::Err($crate::IdentifierError::Empty {
                        type_name: ::core::stringify!($name),
                    });
                }
                if raw.len() > Self::MAX_LEN {
                    return ::core::result::Result::Err($crate::IdentifierError::TooLong {
                        type_name: ::core::stringify!($name),
                        got: raw.len(),
                        max: Self::MAX_LEN,
                    });
                }
                if !Self::compiled_pattern().is_match(raw) {
                    return ::core::result::Result::Err(
                        $crate::IdentifierError::PatternMismatch {
                            type_name: ::core::stringify!($name),
                            pattern: Self::PATTERN,
                            input: ::std::borrow::ToOwned::to_owned(raw),
                        },
                    );
                }
                ::core::result::Result::Ok(Self(::std::borrow::ToOwned::to_owned(raw)))
            }

            /// The validated wire text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl ::core::str::FromStr for $name {
            type Err = $crate::IdentifierError;

            fn from_str(raw: &str) -> ::core::result::Result<Self, Self::Err> {
                Self::parse(raw)
            }
        }

        impl ::core::convert::TryFrom<::std::string::String> for $name {
            type Error = $crate::IdentifierError;

            fn try_from(raw: ::std::string::String) -> ::core::result::Result<Self, Self::Error> {
                Self::parse(&raw)
            }
        }

        impl ::core::convert::From<$name> for ::std::string::String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl ::schemars::JsonSchema for $name {
            fn schema_name() -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Borrowed(::core::stringify!($name))
            }

            fn schema_id() -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Borrowed(::core::concat!(
                    ::core::module_path!(),
                    "::",
                    ::core::stringify!($name)
                ))
            }

            fn json_schema(_generator: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                ::schemars::json_schema!({
                    "type": "string",
                    "title": ::core::stringify!($name),
                    "description": $crate::doc_description(Self::DOC_LINES),
                    "pattern": Self::PATTERN,
                    "maxLength": Self::MAX_LEN,
                    "examples": Self::EXAMPLES,
                })
            }
        }
    };
}
