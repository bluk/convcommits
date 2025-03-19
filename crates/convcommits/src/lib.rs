//! ConvCommits is a library to parse [Conventional Commits][conv_commits].
//!
//! * [Latest API Documentation][api_docs]
//!
//! [conv_commits]: https://www.conventionalcommits.org/
//! [api_docs]: https://docs.rs/convcommits/

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]

use core::fmt;

fn is_newline_ch(ch: char) -> bool {
    matches!(ch, '\n')
}

#[derive(Debug)]
enum InvalidElement {
    Type,
    Scope,
    Desc,
    Body,
    Footer,
}

impl fmt::Display for InvalidElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidElement::Type => f.write_str("invalid element"),
            InvalidElement::Scope => f.write_str("invalid scope"),
            InvalidElement::Desc => f.write_str("invalid description"),
            InvalidElement::Body => f.write_str("invalid body"),
            InvalidElement::Footer => f.write_str("invalid footer"),
        }
    }
}

/// Error when parsing the commit message
#[derive(Debug)]
pub struct Error {
    code: InvalidElement,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.code, f)
    }
}

impl core::error::Error for Error {}

impl Error {
    /// If the type element is invalid
    #[must_use]
    pub fn is_invalid_type(&self) -> bool {
        matches!(self.code, InvalidElement::Type)
    }

    /// If the scope element is invalid
    #[must_use]
    pub fn is_invalid_scope(&self) -> bool {
        matches!(self.code, InvalidElement::Scope)
    }

    /// If the description element is invalid
    #[must_use]
    pub fn is_invalid_desc(&self) -> bool {
        matches!(self.code, InvalidElement::Desc)
    }

    /// If the body element is invalid
    #[must_use]
    pub fn is_invalid_body(&self) -> bool {
        matches!(self.code, InvalidElement::Body)
    }

    /// If the footer element is invalid
    #[must_use]
    pub fn is_invalid_footer(&self) -> bool {
        matches!(self.code, InvalidElement::Footer)
    }
}

/// Commit message parsed into the structured elements of the [Conventional
/// Commits][conv_commits] standard.
///
/// [conv_commits]: https://www.conventionalcommits.org/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit<'a> {
    ty: &'a str,
    scope: Option<&'a str>,
    /// Indicates a `!` was detected in the first line after the type and
    /// optional scope
    ///
    /// If this is false, the footer must also be checked for "BREAKING CHANGE"
    /// and similar variants.
    is_breaking_change: bool,
    desc: &'a str,
    body: &'a str,
    footers: &'a str,
}

impl<'a> Commit<'a> {
    /// Returns the type of the commit.
    ///
    /// ```rust
    /// # fn main() -> Result<(), convcommits::Error> {
    /// let msg = r#"feat: add parse() function"#;
    /// let c = convcommits::parse(msg)?;
    /// assert_eq!(c.ty(), "feat");
    /// Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn ty(&self) -> &'a str {
        self.ty
    }

    /// Returns the type of the conventional commit.
    ///
    /// ```rust
    /// # fn main() -> Result<(), convcommits::Error> {
    /// let msg = r#"feat: add no scope parsing"#;
    /// let c = convcommits::parse(msg)?;
    /// assert_eq!(c.scope(), None);
    ///
    /// let msg = r#"feat(parser): add scope parsing"#;
    /// let c = convcommits::parse(msg)?;
    /// assert_eq!(c.scope(), Some("parser"));
    /// Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn scope(&self) -> Option<&'a str> {
        self.scope
    }

    /// Returns the type of the conventional commit.
    ///
    /// ```rust
    /// # fn main() -> Result<(), convcommits::Error> {
    /// let msg = r#"feat: add desc parsing"#;
    /// let c = convcommits::parse(msg)?;
    /// assert_eq!(c.desc(), "add desc parsing");
    /// Ok(())
    /// # }
    /// ```
    pub fn desc(&self) -> &'a str {
        self.desc
    }

    /// Returns true if the message indicates a breaking change.
    ///
    /// A breaking change is indicated by either:
    ///
    /// * a `!` following the type and optional scope
    /// * a `BREAKING CHANGE` or `BREAKING-CHANGE` trailing footer
    ///
    /// ```rust
    /// # fn main() -> Result<(), convcommits::Error> {
    /// let msg = r#"feat!: add breaking change parsing"#;
    /// let c = convcommits::parse(msg)?;
    /// assert!(c.is_breaking_change());
    ///
    /// let msg = r#"feat(parser)!: add breaking change parsing"#;
    /// let c = convcommits::parse(msg)?;
    /// assert!(c.is_breaking_change());
    /// Ok(())
    /// # }
    /// ```
    pub fn is_breaking_change(&self) -> bool {
        if self.is_breaking_change {
            return true;
        }

        // TODO: Need to scan trailing footers for BREAKING CHANGE

        false
    }

    /// Returns the body of the conventional commit.
    pub fn body(&self) -> &str {
        self.body
    }
}

/// Parse a commit message into the structured elements of the [Conventional
/// Commits][conv_commits] standard.
///
/// # Errors
///
/// If the commit message does not match the conventional commit structure, then
/// an error can be returned.
///
/// [conv_commits]: https://www.conventionalcommits.org/
pub fn parse(text: &str) -> Result<Commit<'_>, Error> {
    let mut is_breaking_change = false;

    let mut char_indices = text.char_indices();
    let start_ty_idx;

    loop {
        let (pos, ch) = char_indices.next().ok_or(Error {
            code: InvalidElement::Type,
        })?;

        if !ch.is_whitespace() {
            start_ty_idx = pos;
            break;
        }
    }

    let ty;
    let scope: Option<&str>;

    loop {
        let (pos, ch) = char_indices.next().ok_or(Error {
            code: InvalidElement::Type,
        })?;

        let end_ty_idx;

        match ch {
            ':' => {
                end_ty_idx = pos;
                scope = None;
            }
            '(' => {
                end_ty_idx = pos;

                let end_scope_idx;

                let (pos, ch) = char_indices.next().ok_or(Error {
                    code: InvalidElement::Scope,
                })?;
                let start_scope_idx = pos;
                match ch {
                    ')' => {
                        end_scope_idx = pos;
                        scope = Some(&text[start_scope_idx..end_scope_idx]);
                    }
                    _ => loop {
                        let (pos, ch) = char_indices.next().ok_or(Error {
                            code: InvalidElement::Scope,
                        })?;
                        match ch {
                            ')' => {
                                end_scope_idx = pos;
                                scope = Some(&text[start_scope_idx..end_scope_idx]);
                                break;
                            }
                            _ if ch.is_whitespace() => {
                                if is_newline_ch(ch) {
                                    return Err(Error {
                                        code: InvalidElement::Scope,
                                    });
                                }
                            }
                            _ => {}
                        }
                    },
                }

                loop {
                    let (_pos, ch) = char_indices.next().ok_or(Error {
                        code: InvalidElement::Scope,
                    })?;
                    match ch {
                        ':' => {
                            break;
                        }
                        '!' => {
                            is_breaking_change = true;

                            loop {
                                let (_, ch) = char_indices.next().ok_or(Error {
                                    code: InvalidElement::Scope,
                                })?;
                                match ch {
                                    ':' => {
                                        break;
                                    }
                                    _ if ch.is_whitespace() => {
                                        if is_newline_ch(ch) {
                                            return Err(Error {
                                                code: InvalidElement::Type,
                                            });
                                        }
                                    }
                                    _ => {
                                        return Err(Error {
                                            code: InvalidElement::Type,
                                        });
                                    }
                                }
                            }
                            break;
                        }
                        _ if ch.is_whitespace() => {
                            if is_newline_ch(ch) {
                                return Err(Error {
                                    code: InvalidElement::Type,
                                });
                            }
                        }
                        _ => {
                            return Err(Error {
                                code: InvalidElement::Type,
                            });
                        }
                    }
                }
            }
            '!' => {
                end_ty_idx = pos;
                scope = None;
                is_breaking_change = true;

                loop {
                    let (_, ch) = char_indices.next().ok_or(Error {
                        code: InvalidElement::Type,
                    })?;
                    match ch {
                        ':' => {
                            break;
                        }
                        _ if ch.is_whitespace() => {
                            if is_newline_ch(ch) {
                                return Err(Error {
                                    code: InvalidElement::Type,
                                });
                            }
                        }
                        _ => {
                            return Err(Error {
                                code: InvalidElement::Type,
                            });
                        }
                    }
                }
            }
            _ => {
                if is_newline_ch(ch) {
                    return Err(Error {
                        code: InvalidElement::Type,
                    });
                }
                continue;
            }
        }

        ty = &text[start_ty_idx..end_ty_idx];
        break;
    }

    let start_desc_idx;

    loop {
        let (pos, ch) = char_indices.next().ok_or(Error {
            code: InvalidElement::Desc,
        })?;

        if !ch.is_whitespace() {
            start_desc_idx = pos;
            break;
        }
    }

    let desc;

    let end_desc_idx;

    loop {
        if let Some((pos, ch)) = char_indices.next() {
            if is_newline_ch(ch) {
                end_desc_idx = pos;
                desc = &text[start_desc_idx..end_desc_idx];
                break;
            }
        } else {
            desc = &text[start_desc_idx..];
            return Ok(Commit {
                ty,
                scope,
                is_breaking_change,
                desc,
                body: "",
                footers: "",
            });
        }
    }

    let Some((_, ch)) = char_indices.next() else {
        return Ok(Commit {
            ty,
            scope,
            is_breaking_change,
            desc,
            body: "",
            footers: "",
        });
    };

    if !is_newline_ch(ch) {
        return Err(Error {
            code: InvalidElement::Body,
        });
    }

    let start_body_idx;

    loop {
        let Some((pos, ch)) = char_indices.next() else {
            return Ok(Commit {
                ty,
                scope,
                is_breaking_change,
                desc,
                body: "",
                footers: "",
            });
        };

        if !ch.is_whitespace() {
            start_body_idx = pos;
            break;
        }
    }

    let mut end_body_idx = start_body_idx;
    let mut is_last_ch_whitspace = false;

    loop {
        let Some((pos, ch)) = char_indices.next() else {
            if !is_last_ch_whitspace {
                end_body_idx = text.len();
            }
            return Ok(Commit {
                ty,
                scope,
                is_breaking_change,
                desc,
                body: &text[start_body_idx..end_body_idx],
                footers: "",
            });
        };

        if ch.is_whitespace() {
            if !is_last_ch_whitspace {
                end_body_idx = pos;
            }
            is_last_ch_whitspace = true;
        } else {
            is_last_ch_whitspace = false;
        }
    }

    Ok(Commit {
        ty,
        scope,
        is_breaking_change,
        desc,
        body: &text[start_body_idx..end_body_idx],
        footers: "",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_msg() -> Result<(), Error> {
        let msg = r#"feat: add parser to implementation"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "feat");
        assert_eq!(c.scope(), None);
        assert!(!c.is_breaking_change());
        assert_eq!(c.desc(), "add parser to implementation");
        assert_eq!(c.body(), "");
        // TODO
        Ok(())
    }

    #[test]
    fn parse_breaking_change() -> Result<(), Error> {
        let msg = r#"fix!: remove whitespace in scope"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "fix");
        assert_eq!(c.scope(), None);
        assert!(c.is_breaking_change());
        assert_eq!(c.desc(), "remove whitespace in scope");
        assert_eq!(c.body(), "");
        // TODO
        Ok(())
    }

    #[test]
    fn parse_scope_and_breaking_change() -> Result<(), Error> {
        let msg = r#"fix(api)!: remove whitespace in scope"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "fix");
        assert_eq!(c.scope(), Some("api"));
        assert!(c.is_breaking_change());
        assert_eq!(c.desc(), "remove whitespace in scope");
        assert_eq!(c.body(), "");
        // TODO
        Ok(())
    }

    #[test]
    fn parse_no_body() -> Result<(), Error> {
        let msg = r#"docs: fix spelling in README"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "docs");
        assert_eq!(c.scope(), None);
        assert!(!c.is_breaking_change());
        assert_eq!(c.desc(), "fix spelling in README");
        assert_eq!(c.body(), "");
        // TODO
        Ok(())
    }

    #[test]
    fn parse_scope() -> Result<(), Error> {
        let msg = r#"feat(api): add scope parsing"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "feat");
        assert_eq!(c.scope(), Some("api"));
        assert!(!c.is_breaking_change());
        assert_eq!(c.desc(), "add scope parsing");
        assert_eq!(c.body(), "");
        // TODO
        Ok(())
    }

    #[test]
    fn parse_single_paragraph_body() -> Result<(), Error> {
        let msg = r#"feat: add scope parsing

Parse the scope in parentheses.
"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "feat");
        assert_eq!(c.scope(), None);
        assert!(!c.is_breaking_change());
        assert_eq!(c.desc(), "add scope parsing");
        assert_eq!(c.body(), "Parse the scope in parentheses.");
        // TODO
        Ok(())
    }
}
