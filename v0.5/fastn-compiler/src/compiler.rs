struct Compiler {
    symbols: Box<dyn fastn_compiler::SymbolStore>,
    interner: string_interner::DefaultStringInterner,
    bag: std::collections::HashMap<string_interner::DefaultSymbol, fastn_compiler::LookupResult>,
    #[expect(unused)]
    auto_imports: Vec<fastn_section::AutoImport>,
}

impl Compiler {
    fn new(
        symbols: Box<dyn fastn_compiler::SymbolStore>,
        auto_imports: Vec<fastn_section::AutoImport>,
    ) -> Self {
        Self {
            symbols,
            interner: string_interner::StringInterner::new(),
            bag: std::collections::HashMap::new(),
            auto_imports,
        }
    }

    fn update_partially_resolved(&mut self, partially_resolved: Vec<fastn_unresolved::Definition>) {
        for definition in partially_resolved {
            if definition.is_resolved() {
                let _resolved = definition.resolved();
                todo!("update bag")
            } else {
                todo!("update bag")
            }
        }
    }

    async fn fetch_unresolved_symbols(
        &mut self,
        symbols_to_fetch: &mut [fastn_unresolved::SymbolName],
    ) {
        let symbols = self.symbols.lookup(&mut self.interner, symbols_to_fetch);
        for symbol in symbols {
            self.bag.insert(symbol.symbol(), symbol);
        }
    }

    /// try to resolve as many symbols as possible, and return the ones that we made any progress on.
    ///
    /// this function should be called in a loop, until the list of symbols is empty.
    fn resolve_symbols(
        &mut self,
        _doc: &mut fastn_unresolved::Document,
        _symbols: &mut [fastn_unresolved::SymbolName],
    ) -> Vec<fastn_unresolved::Definition> {
        todo!()
    }

    /// try to make as much progress as possibly by resolving as many symbols as possible, and return
    /// the vec of ones that could not be resolved.
    ///
    /// if this returns an empty list of symbols, we can go ahead and generate the JS.
    fn resolve_document(
        &mut self,
        d: &mut fastn_unresolved::Document,
    ) -> Vec<fastn_unresolved::SymbolName> {
        for ci in d.content.iter_mut() {
            if let fastn_unresolved::UR::UnResolved(_c) = ci {
                todo!()
            }
        }

        todo!()
    }

    async fn compile(
        &mut self,
        document_id: &fastn_unresolved::ModuleName,
        source: &str,
    ) -> Result<fastn_compiler::Output, fastn_compiler::Error> {
        let mut d = fastn_unresolved::parse(document_id, source);
        // we only make 10 attempts to resolve the document: we need a warning if we are not able to
        // resolve the document in 10 attempts.
        for _ in 1..10 {
            // resolve_document can internally run in parallel.
            let mut unresolved_symbols = self.resolve_document(&mut d);
            if unresolved_symbols.is_empty() {
                break;
            }

            self.fetch_unresolved_symbols(&mut unresolved_symbols).await;
            // this itself has to happen in a loop. we need a warning if we are not able to resolve all
            // symbols in 10 attempts.
            for _ in 1..10 {
                // resolve_document can internally run in parallel.
                let partially_resolved = self.resolve_symbols(&mut d, &mut unresolved_symbols);
                self.update_partially_resolved(partially_resolved);

                if unresolved_symbols.is_empty() {
                    break;
                }
                self.fetch_unresolved_symbols(&mut unresolved_symbols).await;
            }

            if !unresolved_symbols.is_empty() {
                // we were not able to resolve all symbols
            }
        }

        todo!()
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
    document_id: &fastn_unresolved::ModuleName,
    source: &str,
    auto_imports: Vec<fastn_section::AutoImport>,
) -> Result<fastn_compiler::Output, fastn_compiler::Error> {
    Compiler::new(symbols, auto_imports)
        .compile(document_id, source)
        .await
}
