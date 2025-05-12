// SPDX-License-Identifier: Apache-2.0

mod parse {
    use std::str::FromStr;

    use expect_test::{
        expect,
        Expect,
    };

    use crate::SemanticVersion;

    /// this is just a fancy way to do
    /// ```no_run
    /// fn tests() {
    ///     let cases = [("0.1.2", expect![]), ("21.0.0-alpha", expect![])];
    ///     for (semver, expect) in cases {
    ///         test(semver, expect)
    ///     }
    /// }
    /// ```
    ///
    /// The advantage this has over that approach is that every test is its own function still,
    /// which allows you to actually know what the _name_ of a failing testcase is.
    ///
    /// # Usage
    /// Mirroring the above situation
    /// ```no_run
    /// tests![
    ///     fn basic("0.1.2") = expect![],
    ///     fn with_prerelease("21.0.0-alpha", expect![]),
    /// ]
    /// ```
    /// which is equivalent to writing:
    /// ```no_run
    /// #[test]
    /// fn basic() {
    ///     test("0.1.2", expect![])
    /// }
    ///
    /// #[test]
    /// fn with_prerelease() {
    ///     test("21.0.0-alpha", expect![])
    /// }
    macro_rules! tests {
        ( $( fn $name:ident ($lit:literal) = $e:expr ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    test($lit, $e)
                }
            )*
        };
    }

    #[track_caller]
    fn test(s: &str, expect: Expect) {
        let semver: SemanticVersion = s.parse().unwrap();

        expect.assert_debug_eq(&semver)
    }

    #[track_caller]
    fn err(s: &str, expect: Expect) {
        match SemanticVersion::from_str(s) {
            Ok(semver) => panic!("invalid semver `{s}` parsed as valid ({semver:?})"),
            Err(e) => expect.assert_eq(&e.to_string()),
        }
    }

    tests![
        fn basic("32.10.53") = expect![[r#"
            SemanticVersion {
                major: 32,
                minor: 10,
                patch: 53,
                prerelease: "",
                build: "",
            }
        "#]],

        fn with_prerelease("31.9.52-1") = expect![[r#"
            SemanticVersion {
                major: 31,
                minor: 9,
                patch: 52,
                prerelease: "1",
                build: "",
            }
        "#]],

        fn with_build("314.159.65+35") = expect![[r#"
            SemanticVersion {
                major: 314,
                minor: 159,
                patch: 65,
                prerelease: "",
                build: "35",
            }
        "#]],

        fn with_prerelease_and_build("2.4.8-16+32") = expect![[r#"
            SemanticVersion {
                major: 2,
                minor: 4,
                patch: 8,
                prerelease: "16",
                build: "32",
            }
        "#]],

        fn spec_prerelease_example1("1.0.0-alpha") = expect![[r#"
            SemanticVersion {
                major: 1,
                minor: 0,
                patch: 0,
                prerelease: "alpha",
                build: "",
            }
        "#]],

        fn spec_prerelease_example2("1.0.0-alpha.1") = expect![[r#"
            SemanticVersion {
                major: 1,
                minor: 0,
                patch: 0,
                prerelease: "alpha.1",
                build: "",
            }
        "#]],

        fn spec_prerelease_example3("1.0.0-0.3.7") = expect![[r#"
            SemanticVersion {
                major: 1,
                minor: 0,
                patch: 0,
                prerelease: "0.3.7",
                build: "",
            }
        "#]],

        fn spec_prerelease_example4("1.0.0-x.7.z.92") = expect![[r#"
            SemanticVersion {
                major: 1,
                minor: 0,
                patch: 0,
                prerelease: "x.7.z.92",
                build: "",
            }
        "#]],

        fn spec_prerelease_example5("1.0.0-x-y-z.--") = expect![[r#"
            SemanticVersion {
                major: 1,
                minor: 0,
                patch: 0,
                prerelease: "x-y-z.--",
                build: "",
            }
        "#]],

        fn spec_build_example1("1.0.0-alpha+001") = expect![[r#"
            SemanticVersion {
                major: 1,
                minor: 0,
                patch: 0,
                prerelease: "alpha",
                build: "001",
            }
        "#]],

        fn spec_build_example2("1.0.0+20130313144700") = expect![[r#"
            SemanticVersion {
                major: 1,
                minor: 0,
                patch: 0,
                prerelease: "",
                build: "20130313144700",
            }
        "#]],

        fn spec_build_example3("1.0.0-beta+exp.sha.5114f85") = expect![[r#"
            SemanticVersion {
                major: 1,
                minor: 0,
                patch: 0,
                prerelease: "beta",
                build: "exp.sha.5114f85",
            }
        "#]],

        fn spec_build_example4("1.0.0+21AF26D3---117B344092BD") = expect![[r#"
            SemanticVersion {
                major: 1,
                minor: 0,
                patch: 0,
                prerelease: "",
                build: "21AF26D3---117B344092BD",
            }
        "#]],
    ];

    #[test]
    fn err_major_leading_zero() {
        err(
            "00.1.2",
            expect!["failed to parse: semver section `major` starts with leading 0: `00`"],
        )
    }

    #[test]
    fn err_minor_leading_zero() {
        err(
            "0.01.2",
            expect!["failed to parse: semver section `minor` starts with leading 0: `01`"],
        )
    }

    #[test]
    fn err_patch_leading_zero() {
        err(
            "0.1.0002",
            expect!["failed to parse: semver section `patch` starts with leading 0: `0002`"],
        )
    }
}

mod display {
    use expect_test::{
        expect,
        Expect,
    };

    use crate::SemanticVersion;

    #[track_caller]
    fn test(semver: SemanticVersion, expect: Expect) {
        expect.assert_eq(&semver.to_string())
    }

    // there aren't enough of these to get the macro treatment

    #[test]
    fn basic() {
        test(
            SemanticVersion {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: String::new(),
                build: String::new(),
            },
            expect!["1.2.3"],
        )
    }

    #[test]
    fn with_prerelease() {
        test(
            SemanticVersion {
                major: 3,
                minor: 1,
                patch: 4,
                prerelease: "15.92".to_owned(),
                build: String::new(),
            },
            expect!["3.1.4-15.92"],
        )
    }

    #[test]
    fn with_build() {
        test(
            SemanticVersion {
                major: 1,
                minor: 41,
                patch: 1,
                prerelease: String::new(),
                build: "6535asd".to_owned(),
            },
            expect!["1.41.1+6535asd"],
        )
    }

    #[test]
    fn with_prerelease_and_build() {
        test(
            SemanticVersion {
                major: 0,
                minor: 1,
                patch: 4,
                prerelease: "0.9a2".to_owned(),
                build: "sha.25531c".to_owned(),
            },
            expect!["0.1.4-0.9a2+sha.25531c"],
        )
    }
}
