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
}

impl fmt::Display for InvalidElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidElement::Type => f.write_str("invalid element"),
            InvalidElement::Scope => f.write_str("invalid scope"),
            InvalidElement::Desc => f.write_str("invalid description"),
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
    rest: &'a str,
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
        self.ty.trim()
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
        self.scope.map(str::trim)
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
        self.desc.trim()
    }

    /// Returns the type of the conventional commit.
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

        todo!()
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
                rest: "",
            });
        }
    }

    Ok(Commit {
        ty,
        scope,
        is_breaking_change,
        desc,
        rest: &text[end_desc_idx..],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_msg_parse() -> Result<(), Error> {
        let msg = r#"feat: add parser to implementation"#;
        let c = parse(msg)?;
        assert_eq!(c.ty, "feat");
        Ok(())
    }

    #[test]
    fn simple_msg_ty_fix() -> Result<(), Error> {
        let msg = r#"fix: modify ty to ignore spaces"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "fix");
        Ok(())
    }
}
