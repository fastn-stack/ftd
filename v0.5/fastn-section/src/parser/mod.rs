pub(super) mod identifier;
pub(super) mod kind;
pub(super) mod kinded_name;
pub(super) mod module_name;
pub(super) mod package_name;
pub(super) mod qualified_identifier;
pub(super) mod section;
pub(super) mod section_init;
pub(super) mod tes;
pub(super) mod visibility;

impl fastn_section::Document {
    pub fn parse(source: &str) -> fastn_section::Document {
        let mut scanner = fastn_section::Scanner::new(
            source,
            Default::default(),
            fastn_section::Document::default(),
        );
        document(&mut scanner);
        scanner.output
    }
}

pub fn document(scanner: &mut fastn_section::Scanner<fastn_section::Document>) {
    // TODO: parse module_doc, comments etc
    scanner.skip_spaces();
    while let Some(section) = fastn_section::section(scanner) {
        scanner.skip_spaces();
        scanner.skip_new_lines();
        scanner.skip_spaces();
        scanner.output.sections.push(section);
    }
}

#[cfg(test)]
#[track_caller]
fn p<
    T: fastn_section::JDebug,
    F: FnOnce(&mut fastn_section::Scanner<fastn_section::Document>) -> T,
>(
    source: &str,
    f: F,
    debug: serde_json::Value,
    remaining: &str,
) {
    let mut scanner = fastn_section::Scanner::new(
        source,
        Default::default(),
        fastn_section::Document::default(),
    );
    let result = f(&mut scanner);
    assert_eq!(result.debug(source), debug);
    assert_eq!(scanner.remaining(), remaining);
}

#[macro_export]
macro_rules! tt {
    ($f:expr) => {
        #[allow(unused_macros)]
        macro_rules! t {
            ($source:expr, $debug:tt, $remaining:expr) => {
                fastn_section::parser::p($source, $f, serde_json::json!($debug), $remaining);
            };
            ($source:expr, $debug:tt) => {
                fastn_section::parser::p($source, $f, serde_json::json!($debug), "");
            };
        }
        #[allow(unused_macros)]
        macro_rules! f {
            ($source:expr) => {
                fastn_section::parser::p($source, $f, serde_json::json!(null), $source);
            };
        }
    };
}

#[cfg(test)]
mod test {
    fn doc(
        scanner: &mut fastn_section::Scanner<fastn_section::Document>,
    ) -> fastn_section::Document {
        fastn_section::parser::document(scanner);
        scanner.output.clone()
    }

    fastn_section::tt!(doc);
    #[test]
    fn document() {
        t!(
            "-- foo: Hello World",
            {
                "sections": [{
                    "init": {"name": "foo"},
                    "caption": ["Hello World"]
                }]
            }
        );

        t!(
            "-- foo: Hello World from foo\n-- bar: Hello World from bar",
            {
                "sections": [
                    {
                        "init": {"name": "foo"},
                        "caption": ["Hello World from foo"]
                    },
                    {
                        "init": {"name": "bar"},
                        "caption": ["Hello World from bar"]
                    }
                ]
            }
        );
    }
}