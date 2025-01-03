// # note on error handling.
//
// we can handle error in this parser in such a way that our rest of parsers that come after,
// like router parser, and the actual compiler, can run even if there are some issues (error, not
// warning) encountered in this phase.
//
// but for simplicity’s sake, we are going to not do that now, and return either a package object
// if there are no errors (maybe warnings).
//
// even within this parser, we bail early if any one FASTN.ftd is found with errors in it, this is
// kind of acceptable as all FASTN.ftd files, other the one in the current package, must not have
// errors because they are published dependencies.
//
// though this is not strictly true, say if one of the dependencies needed some system, and main
// package has not provided it, then dependency can also have errors.

#[derive(Debug, Default)]
pub struct State {
    name: fastn_package::UR<(), String>,
    systems: Vec<fastn_package::UR<String, fastn_package::System>>,
    dependencies: Vec<fastn_package::UR<String, fastn_package::Dependency>>,
    pub auto_imports: Vec<fastn_package::AutoImport>,
    apps: Vec<fastn_package::UR<String, fastn_package::App>>,
    packages: std::collections::HashMap<String, fastn_package::Package>,
    pub diagnostics: Vec<fastn_section::Diagnostic>,
}

type PResult<T> =
    std::result::Result<(T, Vec<fastn_section::Warning>), Vec<fastn_section::Diagnostic>>;

impl fastn_package::Package {
    pub fn reader() -> fastn_continuation::Result<State> {
        fastn_continuation::Result::Stuck(Default::default(), vec!["FASTN.ftd".to_string()])
    }
}

impl fastn_continuation::Continuation for State {
    // we return a package object if we parsed, even a partial package.
    type Output = PResult<fastn_package::MainPackage>;
    type Needed = Vec<String>; // vec of file names
    type Found = Vec<(
        String, // file name
        Result<Option<(fastn_section::Document, Vec<String>)>, fastn_section::Error>,
    )>;

    fn continue_after(
        mut self,
        n: Vec<(
            String,
            Result<Option<(fastn_section::Document, Vec<String>)>, fastn_section::Error>,
        )>,
    ) -> fastn_continuation::Result<Self> {
        match self.name {
            // if the name is not resolved means this is the first attempt.
            fastn_package::UR::UnResolved(()) => {
                assert_eq!(n.len(), 1);
                assert_eq!(n[0].0, "FASTN.ftd");

                match n.into_iter().next() {
                    Some((_name, Ok(Some((doc, file_list))))) => {
                        let _package = match parse_package(doc, file_list) {
                            Ok((package, warnings)) => {
                                self.diagnostics.extend(
                                    warnings.into_iter().map(fastn_section::Diagnostic::Warning),
                                );
                                package
                            }
                            Err(diagnostics) => {
                                self.diagnostics.extend(diagnostics);
                                return fastn_continuation::Result::Done(Err(self.diagnostics));
                            }
                        };
                        // self.name = fastn_package::UR::Resolved();
                        todo!()
                    }
                    Some((_name, Ok(None))) | Some((_name, Err(_))) => {
                        // Ok(None) means we failed to find a file named FASTN.ftd.
                        // Err(e) means we failed to parse the content of FASTN.ftd.
                        todo!()
                    }
                    None => unreachable!("we did a check for this already, list has 1 element"),
                }
            }
            // even if we failed to find name, we still continue to process as many dependencies,
            // etc. as possible.
            // so this case handles both name found and name error cases.
            _ => {
                todo!()
            }
        }
    }
}

fn parse_package(
    doc: fastn_section::Document,
    _file_list: Vec<String>,
) -> PResult<fastn_package::Package> {
    let mut warnings = vec![];
    let mut package = fastn_package::Package::default();
    let sections = doc.sections.iter();
    match sections
        .clone()
        .find(|&section| section.simple_name() == Some("name"))
    {
        Some(section) => match section.simple_caption() {
            Some(name) => package.name = name.to_string(),
            None => {
                // TODO: keep track of which FASTN.ftd has this issue, we are not keeping track
                //       of error and warning locations / file names so far
                warnings.push(fastn_section::Warning::PackageNameNotInCaption);
                // we do not bail at this point, missing package name is just a warning for now
            }
        },
        None => {
            warnings.push(fastn_section::Warning::PackageNameNotInCaption);
            // we do not bail at this point, missing package name is just a warning for now
        }
    }

    Ok((package, warnings))
}
