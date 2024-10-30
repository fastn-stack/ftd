pub fn kind(scanner: &mut fastn_p1::parser::Scanner) -> Option<fastn_p1::Kind> {
    dbg!(scanner.remaining());
    let qi = match dbg!(fastn_p1::parser::qualified_identifier(scanner)) {
        Some(qi) => dbg!(qi),
        None => return None,
    };

    // By scoping `index` here, it becomes eligible for garbage collection as soon
    // as it’s no longer needed, reducing memory usage. This block performs a look-ahead
    // to check for an optional `<>` part.
    {
        let index = scanner.index();
        scanner.skip_spaces();

        // Check if there's a `<`, indicating the start of generic arguments.
        if !scanner.take('<') {
            scanner.reset(index);
            // No generics, return as simple `Kind`
            return Some(qi.into());
        }
    }

    scanner.skip_spaces();
    // Parse arguments within the `<...>`
    let mut args = Vec::new();

    // Continue parsing arguments until `>` is reached
    while let Some(arg) = kind(scanner) {
        args.push(arg);

        scanner.skip_spaces();

        // If a `>` is found, end of arguments
        if scanner.take('>') {
            break;
        }

        // If a comma is expected between arguments, consume it and move to the next
        if !scanner.take(',') {
            // If no comma and no `>`, the syntax is invalid
            return None;
        }

        scanner.skip_spaces();
    }

    // Return a `Kind` with the parsed `name` and `args`
    Some(fastn_p1::Kind {
        name: qi,
        args: Some(args),
        doc: None,        // Documentation not parsed here
        visibility: None, // Visibility not parsed here
    })
}

#[cfg(test)]
mod test {
    fastn_p1::tt!(super::kind);

    #[test]
    fn kind() {
        t!("string", "string");
        t!("list<string>", {"name": "list", "args": ["string"]});
        t!("foo<a, b>", {"name": "foo", "args": ["a", "b"]});
        t!(
            "foo<bar   <k >>",
            {"name": "foo", "args": [{"name": "bar", "args": ["k"]}]}
        );
        t!(
            "foo \t <a, b< asd                 >, c, d>",
            {
                "name": "foo",
                "args": [
                    "a",
                    {"name": "b", "args": ["asd"]},
                    "c",
                    "d"
                ]
            }
        );
        t!(
            "foo<a        , b\t,\tc, d, e>",
            {
                "name": "foo",
                "args": ["a","b","c","d","e"]
            }
        );

        t!(
            "foo < bar<k>> ",
            {"name": "foo", "args": [{"name": "bar", "args": ["k"]}]},
            " "
        );
        t!(
            "foo<bar<k>>  moo",
            {"name": "foo", "args": [{"name": "bar", "args": ["k"]}]},
            "  moo"
        );
        f!(" string");
        // t!(
        //     "foo<\n  bar \n <\n k>\n>  moo",
        //     {"name": "foo", "args": [{"name": "bar", "args": ["k"]}]},
        //     "  moo"
        // );
        // t!(
        //     "foo<\n  ;; some comment\n bar \n ;; more comments \n<\n k>\n>  moo",
        //     {"name": "foo", "args": [{"name": "bar", "args": ["k"]}]},
        //     "  moo"
        // );
    }
}
