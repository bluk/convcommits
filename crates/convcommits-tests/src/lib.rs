#[cfg(test)]
mod tests {
    #[derive(serde::Serialize)]
    pub struct Info<'a> {
        pub msg: &'a str,
    }

    #[test]
    fn parse_commit_msgs() {
        use std::fs;

        insta::glob!("../tests/resources/commits", "*.txt", |path| {
            let msg = fs::read_to_string(path).unwrap();

            let info = Info { msg: &msg };
            insta::with_settings!({
                info => &info,
            }, {
                insta::assert_debug_snapshot!(convcommits::parse(&msg));
            });
        })
    }
}
