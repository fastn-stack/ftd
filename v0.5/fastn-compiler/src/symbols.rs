#[async_trait::async_trait]
pub trait SymbolStore {
    /// it is okay / acceptable to return more symbols than asked.
    ///
    /// this is because if we are fetching symbols by parsing a ftd file, it makes sense to store
    /// all the symbols found in that file in one go.
    /// instead of parsing the file multiple times, or storing the symbols on the type implementing
    /// this trait.
    ///
    /// or maybe the system can predict that if you asked for one symbol, you are going to ask
    /// for some related symbols soon.
    // TODO: should we make it async?
    async fn lookup(
        &mut self,
        arena: &mut fastn_unresolved::Arena,
        symbols: &std::collections::HashSet<fastn_unresolved::Symbol>,
        auto_imports: &Option<fastn_unresolved::SFId>,
    ) -> Vec<fastn_unresolved::URD>;
}
