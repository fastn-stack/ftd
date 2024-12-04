impl fastn::commands::Render {
    pub async fn run(self, config: &mut fastn_core::Config, arena: &fastn_unresolved::Arena) {
        let route = config.resolve(self.path.as_str()).await;
        match route {
            fastn_core::Route::Document(path, data) => {
                let html =
                    render_document(config.auto_imports, arena, path.as_str(), data, self.strict)
                        .await;
                std::fs::write(path.replace(".ftd", ".html"), html).unwrap();
            }
            _ => todo!(),
        };
    }
}

pub(crate) async fn render_document(
    auto_imports: fastn_unresolved::AliasesID,
    global_arena: &fastn_unresolved::Arena,
    path: &str,
    _data: serde_json::Value,
    _strict: bool,
) -> String {
    let source = std::fs::File::open(path)
        .and_then(std::io::read_to_string)
        .unwrap();
    let o = fastn_compiler::compile(
        Box::new(fastn::Symbols {}),
        &source,
        "main",
        None,
        auto_imports,
        global_arena,
    )
    .await
    .unwrap();

    let h = fastn_runtime::HtmlData::from_cd(o);
    h.to_test_html()
}
