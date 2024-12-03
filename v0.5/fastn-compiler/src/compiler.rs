const ITERATION_THRESHOLD: usize = 100;
// foo.ftd
// -- import: foo as f (f => foo)
//
// -- import: bar      (bar => Module<bar>, x => Symbol<bar.y>) (bar => bar, x => bar.y)
// exposing: y as x
//
pub(crate) struct Compiler {
    symbols: Box<dyn fastn_compiler::SymbolStore>,
    pub(crate) definitions_used: std::collections::HashSet<fastn_unresolved::Symbol>,
    pub(crate) arena: fastn_unresolved::Arena,
    pub(crate) definitions: std::collections::HashMap<String, fastn_unresolved::URD>,
    /// we keep track of every module found (or not found), if not in dict we don't know
    /// if module exists, if in dict bool tells if it exists.
    pub(crate) modules: std::collections::HashMap<fastn_unresolved::Module, bool>,
    /// checkout resolve_document for why this is an Option
    content: Option<Vec<fastn_unresolved::URCI>>,
    pub(crate) document: fastn_unresolved::Document,
    auto_imports: fastn_unresolved::AliasesID,
}

impl Compiler {
    fn new(
        symbols: Box<dyn fastn_compiler::SymbolStore>,
        source: &str,
        package: &str,
        module: Option<&str>,
        auto_imports: fastn_unresolved::AliasesID,
        mut arena: fastn_unresolved::Arena,
    ) -> Self {
        let mut document = fastn_unresolved::parse(
            fastn_unresolved::Module::new(package, module, &mut arena),
            source,
            &mut arena,
            auto_imports,
        );
        let content = Some(document.content);
        document.content = vec![];

        Self {
            symbols,
            arena,
            definitions: std::collections::HashMap::new(),
            modules: std::collections::HashMap::new(),
            content,
            document,
            auto_imports,
            definitions_used: Default::default(),
        }
    }

    async fn fetch_unresolved_symbols(
        &mut self,
        symbols_to_fetch: &std::collections::HashSet<fastn_unresolved::Symbol>,
    ) {
        self.definitions_used
            .extend(symbols_to_fetch.iter().cloned());
        let definitions = self
            .symbols
            .lookup(&mut self.arena, symbols_to_fetch, self.auto_imports)
            .await;
        for definition in definitions {
            // the following is only okay if our symbol store only returns unresolved definitions,
            // some other store might return resolved definitions, and we need to handle that.
            self.definitions.insert(
                definition
                    .unresolved()
                    .unwrap()
                    .symbol
                    .clone()
                    .unwrap()
                    .string(&self.arena),
                definition,
            );
        }
    }

    /// try to resolve as many symbols as possible, and return the ones that we made any progress on.
    ///
    /// this function should be called in a loop, until the list of symbols is empty.
    fn resolve_symbols(
        &mut self,
        symbols: std::collections::HashSet<fastn_unresolved::Symbol>,
    ) -> ResolveSymbolsResult {
        let mut r = ResolveSymbolsResult::default();
        for symbol in symbols {
            // what if this is a recursive definition?
            // `foo` calling `foo`?
            // we will not find `foo` in the `bag` anymore, so we have to explicitly check for that.
            // but what if `foo` calls `bar` and `bar` calls `foo`?
            // we will not be able to resolve that.
            // it won't be a problem because `definition.resolve()` is not recursive, meaning if
            // `foo` is being resolved,
            // and it encounters `bar`, we will not try to internally
            // resolve `bar`, we will stop till bar is fully resolved.
            // in case of recursion, the foo will have first resolved its signature, and then,
            // when `bar` needs signature of `foo,`
            // it will find it from the partially resolved
            // `foo` in the `bag`.
            // to make sure this happens better, we have to ensure that the definition.resolve()
            // tries to resolve the signature first, and then the body.
            let mut definition = self.definitions.remove(symbol.str(&self.arena));
            match definition.as_mut() {
                Some(fastn_unresolved::UR::UnResolved(definition)) => {
                    let mut o = Default::default();
                    definition.resolve(&self.definitions, &self.modules, &mut self.arena, &mut o);
                    r.need_more_symbols.extend(o.stuck_on);
                    self.document.merge(o.errors, o.warnings, o.comments);
                }
                Some(fastn_unresolved::UR::Resolved(_)) => unreachable!(),
                _ => {
                    r.unresolvable.insert(symbol.clone());
                }
            }
            if let Some(fastn_unresolved::UR::UnResolved(definition)) = definition {
                match definition.resolved() {
                    Ok(resolved) => {
                        self.definitions.insert(
                            symbol.string(&self.arena),
                            fastn_unresolved::UR::Resolved(Some(resolved)),
                        );
                    }
                    Err(s) => {
                        r.need_more_symbols.insert(symbol.clone());
                        self.definitions.insert(
                            symbol.string(&self.arena),
                            fastn_unresolved::UR::UnResolved(s),
                        );
                    }
                }
            }
        }

        r
    }

    /// try to make as much progress as possibly by resolving as many symbols as possible, and return
    /// the vec of ones that could not be resolved.
    ///
    /// if this returns an empty list of symbols, we can go ahead and generate the JS.
    fn resolve_document(&mut self) -> std::collections::HashSet<fastn_unresolved::Symbol> {
        let mut stuck_on_symbols = std::collections::HashSet::new();

        let content = self.content.replace(vec![]).unwrap();
        dbg!(&content);
        let mut new_content = vec![];

        for mut ci in content {
            if let fastn_unresolved::UR::UnResolved(ref mut c) = ci {
                let mut needed = Default::default();
                c.resolve(
                    &self.definitions,
                    &self.modules,
                    &mut self.arena,
                    &mut needed,
                );
                stuck_on_symbols.extend(needed.stuck_on);
                self.document
                    .merge(needed.errors, needed.warnings, needed.comments);
            }
            new_content.push(ci);
        }
        self.content = Some(new_content);

        stuck_on_symbols
    }

    async fn compile(mut self) -> Result<fastn_resolved::CompiledDocument, fastn_compiler::Error> {
        // we only make 10 attempts to resolve the document: we need a warning if we are not able to
        // resolve the document in 10 attempts.
        let mut unresolvable = std::collections::HashSet::new();
        // let mut ever_used = std::collections::HashSet::new();
        let mut iterations = 0;
        while iterations < ITERATION_THRESHOLD {
            // resolve_document can internally run in parallel.
            // TODO: pass unresolvable to self.resolve_document() and make sure they don't come back
            dbg!(&self.content);
            let unresolved_symbols = self.resolve_document();
            if unresolved_symbols.is_empty() {
                dbg!(&self.content);
                break;
            }
            dbg!(&self.content);
            // ever_used.extend(&unresolved_symbols);
            self.fetch_unresolved_symbols(&unresolved_symbols).await;
            // this itself has to happen in a loop. we need a warning if we are not able to resolve all
            // symbols in 10 attempts.
            let mut r = ResolveSymbolsResult::default();
            r.need_more_symbols.extend(unresolved_symbols);

            while iterations < ITERATION_THRESHOLD {
                // resolve_document can internally run in parallel.
                // TODO: pass unresolvable to self.resolve_symbols() and make sure they don't come back
                r = self.resolve_symbols(r.need_more_symbols);
                unresolvable.extend(r.unresolvable);
                if r.need_more_symbols.is_empty() {
                    break;
                }
                // ever_used.extend(r.need_more_symbols);
                self.fetch_unresolved_symbols(&r.need_more_symbols).await;
                iterations += 1;
            }

            iterations += 1;
        }

        // we are here means ideally we are done.
        // we could have some unresolvable symbols or self.document.errors may not be empty.
        if !unresolvable.is_empty()
            || !self.document.errors.is_empty()
            || iterations == ITERATION_THRESHOLD
        {
            // we were not able to resolve all symbols or there were errors
            // return Err(fastn_compiler::Error {
            //     messages: todo!(),
            //     resolved: todo!(),
            //     symbol_errors: todo!(),
            // });
            todo!();
        }

        // there were no errors, etc.
        Ok(dbg!(fastn_resolved::CompiledDocument {
            content: fastn_compiler::utils::resolved_content(self.content.unwrap()),
            definitions: fastn_compiler::utils::used_definitions(
                self.definitions,
                self.definitions_used,
                self.arena,
            ),
        }))
    }
}

/// this is our main compiler
///
/// it should be called with a parsed document, and it returns generated JS.
///
/// on success, we return the JS, and list of warnings, and on error, we return the list of
/// diagnostics, which is an enum containing warning and error.
///
/// earlier we had strict mode here, but to simplify things, now we let the caller convert non-empty
/// warnings from OK part as error, and discard the generated JS.
pub async fn compile(
    symbols: Box<dyn fastn_compiler::SymbolStore>,
    source: &str,
    package: &str,
    module: Option<&str>,
    auto_imports: fastn_unresolved::AliasesID,
    arena: fastn_unresolved::Arena,
) -> Result<fastn_resolved::CompiledDocument, fastn_compiler::Error> {
    Compiler::new(symbols, source, package, module, auto_imports, arena)
        .compile()
        .await
}

#[derive(Default)]
struct ResolveSymbolsResult {
    need_more_symbols: std::collections::HashSet<fastn_unresolved::Symbol>,
    unresolvable: std::collections::HashSet<fastn_unresolved::Symbol>,
}
