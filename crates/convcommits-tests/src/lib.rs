#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    #[derive(serde::Serialize)]
    pub struct Info<'a> {
        pub msg: &'a str,
    }

    #[test]
    fn parse_commit_msgs() {
        use std::fs;

        use serde::Serialize;

        #[derive(Debug, Serialize)]
        struct ParsedMsg<'a> {
            pub ty: &'a str,
            pub scope: Option<&'a str>,
            pub is_breaking_change: bool,
            pub desc: &'a str,
            pub body: &'a str,
            pub footer: &'a str,
        }

        #[derive(Debug, Serialize)]
        enum Error {
            Ty,
            Scope,
            Desc,
            Prefix,
            Body,
        }

        insta::glob!("../tests/resources/commits", "*.txt", |path| {
            let msg = fs::read_to_string(path).unwrap();
            let info = Info { msg: &msg };

            let parsed_msg = convcommits::parse(&msg)
                .map(|m| ParsedMsg {
                    ty: m.ty(),
                    scope: m.scope(),
                    is_breaking_change: m.is_breaking_change(),
                    desc: m.desc(),
                    body: m.body(),
                    footer: m.footer(),
                })
                .map_err(|e| {
                    if e.is_invalid_body() {
                        Error::Body
                    } else if e.is_invalid_desc() {
                        Error::Desc
                    } else if e.is_invalid_prefix() {
                        Error::Prefix
                    } else if e.is_invalid_scope() {
                        Error::Scope
                    } else if e.is_invalid_type() {
                        Error::Ty
                    } else {
                        panic!()
                    }
                });

            insta::with_settings!({
                info => &info,
            }, {
                insta::assert_yaml_snapshot!(&parsed_msg);
            });

            if let Ok(parsed_msg) = parsed_msg {
                assert_eq!(parsed_msg.ty, parsed_msg.ty.trim());
                if let Some(scope) = parsed_msg.scope {
                    assert_eq!(scope, scope.trim());
                }
                assert_eq!(parsed_msg.desc, parsed_msg.desc.trim());
            }
        })
    }

    proptest! {
        #[test]
        fn fuzz_parse(s in any::<String>()) {
            let _ = convcommits::parse(&s);
        }
    }
}
