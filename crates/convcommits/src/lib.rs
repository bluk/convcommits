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
    Prefix,
    Desc,
    Body,
}

impl fmt::Display for InvalidElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidElement::Type => f.write_str("invalid element"),
            InvalidElement::Scope => f.write_str("invalid scope"),
            InvalidElement::Prefix => f.write_str("invalid prefix"),
            InvalidElement::Desc => f.write_str("invalid description"),
            InvalidElement::Body => f.write_str("invalid body"),
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

    /// If the prefix element is invalid
    #[must_use]
    pub fn is_invalid_prefix(&self) -> bool {
        matches!(self.code, InvalidElement::Prefix)
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
    footer: &'a str,
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

    /// Returns the footer of the conventional commit.
    pub fn footer(&self) -> &str {
        self.footer
    }
}

/// Result type with the crate's [Error] type.
pub type Result<T> = core::result::Result<T, Error>;

fn skip_whitespace<I>(char_indices: &mut I) -> Option<(usize, char)>
where
    I: Iterator<Item = (usize, char)>,
{
    loop {
        let (pos, ch) = char_indices.next()?;

        if !ch.is_whitespace() {
            return Some((pos, ch));
        }
    }
}

fn expect_colon<I>(char_indices: &mut I) -> Option<(usize, char)>
where
    I: Iterator<Item = (usize, char)>,
{
    let (pos, ch) = char_indices.next()?;
    if ch == ':' { Some((pos, ch)) } else { None }
}

fn expect_colon_with_whitespace<I>(char_indices: &mut I) -> Option<(usize, char)>
where
    I: Iterator<Item = (usize, char)>,
{
    loop {
        let (pos, ch) = char_indices.next()?;

        match ch {
            ':' => {
                return Some((pos, ch));
            }
            _ => {
                if ch.is_whitespace() {
                    if is_newline_ch(ch) {
                        return None;
                    }
                } else {
                    return None;
                }
            }
        }
    }
}

fn expect_space<I>(char_indices: &mut I) -> Option<(usize, char)>
where
    I: Iterator<Item = (usize, char)>,
{
    let (pos, ch) = char_indices.next()?;
    if ch == ' ' { Some((pos, ch)) } else { None }
}

fn expect_non_whitespace<I>(char_indices: &mut I) -> Option<(usize, char)>
where
    I: Iterator<Item = (usize, char)>,
{
    let (pos, ch) = char_indices.next()?;
    if ch.is_whitespace() {
        return None;
    }
    Some((pos, ch))
}

/// Parses the scope after the '('.
fn parse_scope<I>(char_indices: &mut I) -> Option<(usize, usize)>
where
    I: Iterator<Item = (usize, char)>,
{
    let end_scope_idx;

    let (pos, ch) = char_indices.next()?;
    let start_scope_idx = pos;
    match ch {
        ')' => {
            end_scope_idx = pos;
        }
        _ if ch.is_whitespace() => {
            return None;
        }
        _ => loop {
            let (pos, ch) = char_indices.next()?;
            match ch {
                ')' => {
                    end_scope_idx = pos;
                    break;
                }
                _ if ch.is_whitespace() => {
                    return None;
                }
                _ => {}
            }
        },
    }
    Some((start_scope_idx, end_scope_idx))
}

fn parse_desc<I>(char_indices: &mut I) -> Option<(usize, Option<usize>)>
where
    I: Iterator<Item = (usize, char)>,
{
    let (start_desc_idx, _) = expect_non_whitespace(char_indices)?;

    loop {
        if let Some((pos, ch)) = char_indices.next() {
            if is_newline_ch(ch) {
                return Some((start_desc_idx, Some(pos)));
            }
        } else {
            return Some((start_desc_idx, None));
        }
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
pub fn parse(text: &str) -> Result<Commit<'_>> {
    let mut is_breaking_change = false;

    let mut char_indices = text.char_indices();
    let (start_ty_idx, _) = skip_whitespace(&mut char_indices).ok_or(Error {
        code: InvalidElement::Type,
    })?;

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

                let (start_scope_idx, end_scope_idx) =
                    parse_scope(&mut char_indices).ok_or(Error {
                        code: InvalidElement::Scope,
                    })?;

                scope = Some(&text[start_scope_idx..end_scope_idx]);

                let (_, ch) = char_indices.next().ok_or(Error {
                    code: InvalidElement::Prefix,
                })?;
                match ch {
                    ':' => {}
                    '!' => {
                        is_breaking_change = true;
                        expect_colon(&mut char_indices).ok_or(Error {
                            code: InvalidElement::Prefix,
                        })?;
                    }
                    _ => {
                        return Err(Error {
                            code: InvalidElement::Prefix,
                        });
                    }
                }
            }
            '!' => {
                end_ty_idx = pos;
                scope = None;
                is_breaking_change = true;

                expect_colon(&mut char_indices).ok_or(Error {
                    code: InvalidElement::Prefix,
                })?;
            }
            _ => {
                if ch.is_whitespace() {
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

    expect_space(&mut char_indices).ok_or(Error {
        code: InvalidElement::Prefix,
    })?;

    let desc = match parse_desc(&mut char_indices).ok_or(Error {
        code: InvalidElement::Desc,
    })? {
        (start_desc_idx, Some(end_desc_idx)) => &text[start_desc_idx..end_desc_idx],
        (start_desc_idx, None) => {
            return Ok(Commit {
                ty,
                scope,
                is_breaking_change,
                desc: &text[start_desc_idx..],
                body: "",
                footer: "",
            });
        }
    };

    let Some((_, ch)) = char_indices.next() else {
        return Ok(Commit {
            ty,
            scope,
            is_breaking_change,
            desc,
            body: "",
            footer: "",
        });
    };

    if !is_newline_ch(ch) {
        return Err(Error {
            code: InvalidElement::Body,
        });
    }

    let mut char_indices = char_indices.peekable();

    let start_body_idx;

    let mut start_line_idx;

    // used to advance the body
    let mut is_last_ch_whitespace = false;

    let mut is_prev_line_blank = true;

    let mut is_cur_line_blank = true;
    let mut is_line_possibly_trailer = true;

    let mut is_prev_ch_newline;

    loop {
        let Some((pos, ch)) = char_indices.next() else {
            return Ok(Commit {
                ty,
                scope,
                is_breaking_change,
                desc,
                body: "",
                footer: "",
            });
        };

        if !is_newline_ch(ch) {
            start_body_idx = pos;
            start_line_idx = pos;
            is_prev_ch_newline = false;

            if ch.is_whitespace() {
                is_last_ch_whitespace = true;
                is_line_possibly_trailer = false;
            } else {
                is_cur_line_blank = false;
            }

            break;
        }
    }

    let mut end_body_idx = start_body_idx;

    loop {
        let Some((pos, ch)) = char_indices.next() else {
            if !is_last_ch_whitespace {
                end_body_idx = text.len();
            }
            return Ok(Commit {
                ty,
                scope,
                is_breaking_change,
                desc,
                body: &text[start_body_idx..end_body_idx],
                footer: "",
            });
        };

        if is_prev_ch_newline {
            is_prev_ch_newline = false;
            start_line_idx = pos;
        }

        if ch.is_whitespace() {
            if !is_line_possibly_trailer && !is_last_ch_whitespace {
                end_body_idx = pos;
            }
            is_last_ch_whitespace = true;

            if is_newline_ch(ch) {
                if is_cur_line_blank {
                    is_prev_line_blank = true;
                }
                is_cur_line_blank = true;
                is_line_possibly_trailer = true;
                is_prev_ch_newline = true;
            } else {
                if is_line_possibly_trailer && is_prev_line_blank && ch == ' ' {
                    if let Some((_, peek)) = char_indices.peek() {
                        if *peek == '#' {
                            // footer here
                            break;
                        }
                    }
                }

                is_line_possibly_trailer = false;
            }
        } else {
            is_last_ch_whitespace = false;
            is_cur_line_blank = false;

            if is_line_possibly_trailer && is_prev_line_blank && ch == ':' {
                if let Some((_, peek)) = char_indices.peek() {
                    if *peek == ' ' {
                        // footer here
                        break;
                    }
                }
            }
        }
    }

    let start_footer_idx = start_line_idx;
    let mut end_footer_idx = start_line_idx;
    is_last_ch_whitespace = false;

    loop {
        let Some((pos, ch)) = char_indices.next() else {
            if !is_last_ch_whitespace {
                end_footer_idx = text.len();
            }
            break;
        };

        if ch.is_whitespace() {
            if !is_last_ch_whitespace {
                end_footer_idx = pos;
            }
            is_last_ch_whitespace = true;
        } else {
            is_last_ch_whitespace = false;
        }
    }

    Ok(Commit {
        ty,
        scope,
        is_breaking_change,
        desc,
        body: &text[start_body_idx..end_body_idx],
        footer: &text[start_footer_idx..end_footer_idx],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_msg() -> Result<()> {
        let msg = r#"feat: add parser to implementation"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "feat");
        assert_eq!(c.scope(), None);
        assert!(!c.is_breaking_change());
        assert_eq!(c.desc(), "add parser to implementation");
        assert_eq!(c.body(), "");
        assert_eq!(c.footer(), "");
        Ok(())
    }

    #[test]
    fn parse_breaking_change() -> Result<()> {
        let msg = r#"fix!: remove whitespace in scope"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "fix");
        assert_eq!(c.scope(), None);
        assert!(c.is_breaking_change());
        assert_eq!(c.desc(), "remove whitespace in scope");
        assert_eq!(c.body(), "");
        assert_eq!(c.footer(), "");
        Ok(())
    }

    #[test]
    fn parse_scope_and_breaking_change() -> Result<()> {
        let msg = r#"fix(api)!: remove whitespace in scope"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "fix");
        assert_eq!(c.scope(), Some("api"));
        assert!(c.is_breaking_change());
        assert_eq!(c.desc(), "remove whitespace in scope");
        assert_eq!(c.body(), "");
        assert_eq!(c.footer(), "");
        Ok(())
    }

    #[test]
    fn parse_no_body() -> Result<()> {
        let msg = r#"docs: fix spelling in README"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "docs");
        assert_eq!(c.scope(), None);
        assert!(!c.is_breaking_change());
        assert_eq!(c.desc(), "fix spelling in README");
        assert_eq!(c.body(), "");
        assert_eq!(c.footer(), "");
        Ok(())
    }

    #[test]
    fn parse_scope() -> Result<()> {
        let msg = r#"feat(api): add scope parsing"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "feat");
        assert_eq!(c.scope(), Some("api"));
        assert!(!c.is_breaking_change());
        assert_eq!(c.desc(), "add scope parsing");
        assert_eq!(c.body(), "");
        assert_eq!(c.footer(), "");
        Ok(())
    }

    #[test]
    fn parse_single_paragraph_body() -> Result<()> {
        let msg = r#"feat: add scope parsing

Parse the scope in parentheses.
"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "feat");
        assert_eq!(c.scope(), None);
        assert!(!c.is_breaking_change());
        assert_eq!(c.desc(), "add scope parsing");
        assert_eq!(c.body(), "Parse the scope in parentheses.");
        assert_eq!(c.footer(), "");
        Ok(())
    }

    #[test]
    fn parse_multi_paragraph_body() -> Result<()> {
        let msg = r#"feat: add scope parsing

Parse the scope in parentheses.

More description.
"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "feat");
        assert_eq!(c.scope(), None);
        assert!(!c.is_breaking_change());
        assert_eq!(c.desc(), "add scope parsing");
        assert_eq!(
            c.body(),
            "Parse the scope in parentheses.\n\nMore description."
        );
        assert_eq!(c.footer(), "");
        Ok(())
    }

    #[test]
    fn parse_single_footer() -> Result<()> {
        let msg = r#"feat: add scope parsing

Parse the scope in parentheses.

Reviewed-By: example
"#;
        let c = parse(msg)?;
        assert_eq!(c.ty(), "feat");
        assert_eq!(c.scope(), None);
        assert!(!c.is_breaking_change());
        assert_eq!(c.desc(), "add scope parsing");
        assert_eq!(c.body(), "Parse the scope in parentheses.");
        assert_eq!(c.footer(), "Reviewed-By: example");
        Ok(())
    }
}
