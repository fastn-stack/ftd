fn space() -> pretty::RcDoc<'static> {
    pretty::RcDoc::space()
}

fn text(t: &str) -> pretty::RcDoc<'static> {
    pretty::RcDoc::text(t.to_string())
}

fn comma() -> pretty::RcDoc<'static> {
    pretty::RcDoc::text(",".to_string())
}

pub fn to_js(ast: &[fastn_js::Ast], package_name: &str) -> String {
    let mut w = Vec::new();
    let o = pretty::RcDoc::nil().append(pretty::RcDoc::intersperse(
        ast.iter().map(|f| f.to_js(package_name)),
        space(),
    ));
    o.render(80, &mut w).unwrap();
    prettify_js::prettyprint(String::from_utf8(w).unwrap().as_str()).0
}

impl fastn_js::Ast {
    pub fn to_js(&self, package_name: &str) -> pretty::RcDoc<'static> {
        match self {
            fastn_js::Ast::Component(f) => f.to_js(package_name),
            fastn_js::Ast::UDF(f) => f.to_js(package_name),
            fastn_js::Ast::StaticVariable(s) => s.to_js(),
            fastn_js::Ast::MutableVariable(m) => m.to_js(),
            fastn_js::Ast::MutableList(ml) => ml.to_js(),
            fastn_js::Ast::RecordInstance(ri) => ri.to_js(),
            fastn_js::Ast::OrType(ot) => ot.to_js(),
            fastn_js::Ast::Export { from, to } => variable_to_js(
                to,
                &None,
                text(
                    format!(
                        "{}[\"{}\"]",
                        &fastn_js::constants::GLOBAL_VARIABLE_MAP,
                        fastn_js::utils::name_to_js(from)
                    )
                    .as_str(),
                ),
                true,
            ),
        }
    }
}

impl fastn_js::Kernel {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        text("let")
            .append(space())
            .append(text(&self.name))
            .append(space())
            .append(text("="))
            .append(space())
            .append(text("fastn_dom.createKernel("))
            .append(text(&format!("{},", self.parent.clone())))
            .append(space())
            .append(text(self.element_kind.to_js().as_str()))
            .append(text(");"))
    }
}

impl fastn_js::SetProperty {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        text(format!("{}.setProperty(", self.element_name).as_str())
            .append(text(format!("{},", self.kind.to_js()).as_str()))
            .append(space())
            .append(text(
                format!(
                    "{},",
                    &self
                        .value
                        .to_js_with_element_name(&Some(self.element_name.clone()), self.is_code())
                )
                .as_str(),
            ))
            .append(space())
            .append(text(format!("{});", self.inherited).as_str()))
    }
}

impl fastn_js::EventHandler {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        text(format!("{}.addEventHandler(", self.element_name).as_str())
            .append(self.event.to_js())
            .append(comma())
            .append(space())
            .append(text("function()"))
            .append(space())
            .append(text("{"))
            .append(self.action.to_js(&Some(self.element_name.clone())))
            .append(text("});"))
    }
}

impl fastn_js::Event {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        use itertools::Itertools;

        match self {
            fastn_js::Event::Click => text("fastn_dom.Event.Click"),
            fastn_js::Event::MouseEnter => text("fastn_dom.Event.MouseEnter"),
            fastn_js::Event::MouseLeave => text("fastn_dom.Event.MouseLeave"),
            fastn_js::Event::ClickOutside => text("fastn_dom.Event.ClickOutside"),
            fastn_js::Event::GlobalKey(gk) => text(
                format!(
                    "fastn_dom.Event.GlobalKey([{}])",
                    gk.iter()
                        .map(|v| format!("\"{}\"", v))
                        .collect_vec()
                        .join(", ")
                )
                .as_str(),
            ),
            fastn_js::Event::GlobalKeySeq(gk) => text(
                format!(
                    "fastn_dom.Event.GlobalKeySeq([{}])",
                    gk.iter()
                        .map(|v| format!("\"{}\"", v))
                        .collect_vec()
                        .join(", ")
                )
                .as_str(),
            ),
            fastn_js::Event::Input => text("fastn_dom.Event.Input"),
            fastn_js::Event::Change => text("fastn_dom.Event.Change"),
            fastn_js::Event::Blur => text("fastn_dom.Event.Blur"),
            fastn_js::Event::Focus => text("fastn_dom.Event.Focus"),
        }
    }
}

impl fastn_js::FunctionData {
    fn to_js(&self) -> String {
        match self {
            fastn_js::FunctionData::Definition(definition) => {
                format!("{}({})", fastn_js::GET_STATIC_VALUE, definition.to_js())
            }
            fastn_js::FunctionData::Name(name) => {
                fastn_js::utils::name_to_js(name.as_str()).to_string()
            }
        }
    }
}

impl fastn_js::Function {
    pub fn to_js(&self, element_name: &Option<String>) -> pretty::RcDoc<'static> {
        text(format!("{}(", self.name.to_js()).as_str())
            .append(text("{"))
            .append(pretty::RcDoc::intersperse(
                self.parameters.iter().map(|(k, v)| {
                    format!(
                        "{}: {},",
                        fastn_js::utils::name_to_js_(k),
                        v.to_js_with_element_name(element_name, false)
                    )
                }),
                pretty::RcDoc::softline(),
            ))
            .append(text(
                format!(
                    "}}{});",
                    element_name
                        .as_ref()
                        .map(|v| format!(", {}", v))
                        .unwrap_or_default()
                )
                .as_str(),
            ))
    }
}

impl fastn_js::ElementKind {
    pub fn to_js(&self) -> String {
        match self {
            fastn_js::ElementKind::Row => "fastn_dom.ElementKind.Row".to_string(),
            fastn_js::ElementKind::ContainerElement => {
                "fastn_dom.ElementKind.ContainerElement".to_string()
            }
            fastn_js::ElementKind::Column => "fastn_dom.ElementKind.Column".to_string(),
            fastn_js::ElementKind::Integer => "fastn_dom.ElementKind.Integer".to_string(),
            fastn_js::ElementKind::Decimal => "fastn_dom.ElementKind.Decimal".to_string(),
            fastn_js::ElementKind::Boolean => "fastn_dom.ElementKind.Boolean".to_string(),
            fastn_js::ElementKind::Text => "fastn_dom.ElementKind.Text".to_string(),
            fastn_js::ElementKind::Image => "fastn_dom.ElementKind.Image".to_string(),
            fastn_js::ElementKind::Video => "fastn_dom.ElementKind.Video".to_string(),
            fastn_js::ElementKind::IFrame => "fastn_dom.ElementKind.IFrame".to_string(),
            fastn_js::ElementKind::Device => "fastn_dom.ElementKind.Wrapper".to_string(),
            fastn_js::ElementKind::CheckBox => "fastn_dom.ElementKind.CheckBox".to_string(),
            fastn_js::ElementKind::TextInput => "fastn_dom.ElementKind.TextInput".to_string(),
            fastn_js::ElementKind::Rive => "fastn_dom.ElementKind.Rive".to_string(),
            fastn_js::ElementKind::Document => "fastn_dom.ElementKind.Document".to_string(),
            fastn_js::ElementKind::Code => "fastn_dom.ElementKind.Code".to_string(),
            fastn_js::ElementKind::WebComponent(web_component_name) => {
                let name = if let Some((_, name)) = web_component_name.split_once('#') {
                    name.to_string()
                } else {
                    web_component_name.to_string()
                };

                format!(
                    "fastn_dom.ElementKind.WebComponent(\"{name}\", {})",
                    fastn_js::LOCAL_VARIABLE_MAP
                )
            }
        }
    }
}

impl fastn_js::ComponentStatement {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            fastn_js::ComponentStatement::StaticVariable(static_variable) => {
                static_variable.to_js()
            }
            fastn_js::ComponentStatement::MutableVariable(mutable_variable) => {
                mutable_variable.to_js()
            }
            fastn_js::ComponentStatement::CreateKernel(kernel) => kernel.to_js(),
            fastn_js::ComponentStatement::SetProperty(set_property) => set_property.to_js(),
            fastn_js::ComponentStatement::InstantiateComponent(i) => i.to_js(),
            fastn_js::ComponentStatement::AddEventHandler(e) => e.to_js(),
            fastn_js::ComponentStatement::Return { component_name } => {
                text(&format!("return {component_name};"))
            }
            fastn_js::ComponentStatement::ConditionalComponent(c) => c.to_js(),
            fastn_js::ComponentStatement::MutableList(ml) => ml.to_js(),
            fastn_js::ComponentStatement::ForLoop(fl) => fl.to_js(),
            fastn_js::ComponentStatement::RecordInstance(ri) => ri.to_js(),
            fastn_js::ComponentStatement::OrType(ot) => ot.to_js(),
            fastn_js::ComponentStatement::DeviceBlock(db) => db.to_js(),
            fastn_js::ComponentStatement::AnyBlock(ab) => {
                text(format!("if (!ssr) {{{}}}", ab).as_str())
            }
        }
    }
}

impl fastn_js::InstantiateComponentData {
    fn to_js(&self) -> String {
        match self {
            fastn_js::InstantiateComponentData::Definition(definition) => {
                format!("{}({})", fastn_js::GET_STATIC_VALUE, definition.to_js())
            }
            fastn_js::InstantiateComponentData::Name(name) => name.to_owned(),
        }
    }
}

impl fastn_js::InstantiateComponent {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        pretty::RcDoc::text(format!(
            "let {} = {}(",
            self.var_name,
            if self.already_formatted {
                self.component.to_js().to_owned()
            } else {
                fastn_js::utils::name_to_js(self.component.to_js().as_str())
            }
        ))
        .append(pretty::RcDoc::text(self.parent.clone()))
        .append(comma().append(space()))
        .append(pretty::RcDoc::text(self.inherited.clone()))
        .append(if self.arguments.is_empty() {
            pretty::RcDoc::nil()
        } else {
            comma().append(space()).append(
                text("{")
                    .append(
                        pretty::RcDoc::intersperse(
                            self.arguments.iter().map(|(k, value, is_mutable)| {
                                format!(
                                    "{}: {}",
                                    fastn_js::utils::name_to_js_(k),
                                    if *is_mutable {
                                        format!("fastn.wrapMutable({})", value.to_js())
                                    } else {
                                        value.to_js()
                                    }
                                )
                            }),
                            comma().append(space()),
                        )
                        .group(),
                    )
                    .append(text("}")),
            )
        })
        .append(text(");"))
    }
}

impl fastn_js::DeviceBlock {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        text(
            format!(
                "{}fastn_dom.conditionalDom(",
                if self.should_return { "return " } else { "" }
            )
            .as_str(),
        )
        .append(text(self.parent.as_str()))
        .append(comma())
        .append(space())
        .append(text("["))
        .append(text("ftd.device"))
        .append(text("]"))
        .append(comma())
        .append(space())
        .append(text("function () {"))
        .append(text("return (ftd.device.get()"))
        .append(space())
        .append(text("==="))
        .append(self.device.to_js())
        .append(text(");"))
        .append(pretty::RcDoc::softline())
        .append(text("},"))
        .append(text("function (root) {"))
        .append(
            pretty::RcDoc::intersperse(
                self.statements.iter().map(|v| v.to_js()),
                pretty::RcDoc::softline(),
            )
            .group(),
        )
        .append(text(
            format!(
                "}}){};",
                if self.should_return {
                    ".getParent()"
                } else {
                    ""
                }
            )
            .as_str(),
        ))
    }
}

impl fastn_js::ConditionalComponent {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        text(
            format!(
                "{}fastn_dom.conditionalDom(",
                if self.should_return { "return " } else { "" }
            )
            .as_str(),
        )
        .append(text(self.parent.as_str()))
        .append(comma())
        .append(space())
        .append(text("["))
        .append(
            pretty::RcDoc::intersperse(
                self.deps
                    .iter()
                    .map(|v| text(fastn_js::utils::reference_to_js(v).as_str())),
                comma().append(space()),
            )
            .group(),
        )
        .append(text("]"))
        .append(comma())
        .append(space())
        .append(text("function () {"))
        .append(pretty::RcDoc::text(
            fastn_js::to_js::ExpressionGenerator.to_js(&self.condition),
        ))
        .append(text("},"))
        .append(text("function (root) {"))
        .append(
            pretty::RcDoc::intersperse(
                self.statements.iter().map(|v| v.to_js()),
                pretty::RcDoc::softline(),
            )
            .group(),
        )
        .append(text(
            format!(
                "}}){};",
                if self.should_return {
                    ".getParent()"
                } else {
                    ""
                }
            )
            .as_str(),
        ))
    }
}

impl fastn_js::ForLoop {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        text(
            format!(
                "{}{}.forLoop(",
                if self.should_return { "return " } else { "" },
                self.list_variable.to_js() //Todo: if self.list_variable is fastn_js::SetPropertyValue::Value then convert it to fastn.mutableList()
            )
            .as_str(),
        )
        .append(text(self.parent.as_str()))
        .append(comma())
        .append(space())
        .append(text("function (root, item, index) {"))
        .append(
            pretty::RcDoc::intersperse(
                self.statements.iter().map(|v| v.to_js()),
                pretty::RcDoc::softline(),
            )
            .group(),
        )
        .append(text(
            format!(
                "}}){};",
                if self.should_return {
                    ".getParent()"
                } else {
                    ""
                }
            )
            .as_str(),
        ))
    }
}

fn func(
    name: &str,
    params: &[String],
    body: pretty::RcDoc<'static>,
    package_name: &str,
    add_catch_statement: bool,
) -> pretty::RcDoc<'static> {
    let package_name = fastn_js::utils::name_to_js_(package_name);
    let name = fastn_js::utils::name_to_js(name);
    // `.` means the function is placed in object so no need of `let`
    // e.g. ftd.toggle
    if name.contains('.') {
        pretty::RcDoc::nil()
    } else {
        text("let").append(space())
    }
    .append(text(name.as_str()))
    .append(space())
    .append(text("="))
    .append(space())
    .append(text("function"))
    .append(space())
    .append(text("("))
    .append(
        pretty::RcDoc::intersperse(
            params.iter().map(|v| text(v.as_str())),
            comma().append(space()),
        )
        .nest(4)
        .group(),
    )
    .append(text(")"))
    .append(pretty::RcDoc::softline_())
    .append(
        pretty::RcDoc::softline()
            .append(text("{"))
            .append(pretty::RcDoc::softline_())
            .append(text(
                "let __fastn_super_package_name__ = __fastn_package_name__;",
            ))
            .append(pretty::RcDoc::softline_())
            .append(text(&format!(
                "__fastn_package_name__ = \"{}\";",
                package_name
            )))
            .append(pretty::RcDoc::softline_())
            .append(text("try {"))
            .append(pretty::RcDoc::softline_())
            .append(body.nest(4))
            .append(pretty::RcDoc::softline_())
            .append(text(
                format!(
                    "}} {} finally {{ __fastn_package_name__ = __fastn_super_package_name__;}}",
                    if add_catch_statement {
                        "catch (e) {if(!ssr){throw e;}}"
                    } else {
                        ""
                    }
                )
                .as_str(),
            ))
            .append(pretty::RcDoc::softline_())
            .append(text("}"))
            .group(),
    )
    .append(if name.contains('.') {
        pretty::RcDoc::nil()
    } else {
        pretty::RcDoc::softline().append(text(
            format!("{}[\"{name}\"] = {name};", fastn_js::GLOBAL_VARIABLE_MAP).as_str(),
        ))
    })
}

impl fastn_js::Component {
    pub fn to_js(&self, package_name: &str) -> pretty::RcDoc<'static> {
        let body = if self.name.eq(fastn_js::MAIN_FUNCTION) {
            pretty::RcDoc::nil()
        } else {
            let mut local_arguments = vec![];
            let mut local_arguments_dependent = vec![];
            let mut arguments = vec![];
            for (argument_name, value, is_mutable) in self.args.iter() {
                if value.is_local_value() {
                    // Todo: Fix order
                    // -- component show-name:
                    // caption name:
                    // string full-name: $show-name.nickname
                    // string nickname: $show-name.name
                    local_arguments.push((argument_name.to_owned(), value.to_owned()));
                } else if value.is_local_value_dependent() {
                    // Todo: Fix order
                    local_arguments_dependent.push((argument_name.to_owned(), value.to_owned()));
                } else {
                    let value = if *is_mutable {
                        format!("fastn.wrapMutable({})", value.to_js())
                    } else {
                        value.to_js()
                    };

                    arguments.push((argument_name.to_owned(), value));
                }
            }

            text("let")
                .append(space())
                .append(text(fastn_js::LOCAL_VARIABLE_MAP))
                .append(space())
                .append(text("="))
                .append(space())
                .append(text("{"))
                .append(pretty::RcDoc::intersperse(
                    arguments
                        .iter()
                        .map(|(k, v)| format!("{}: {},", fastn_js::utils::name_to_js_(k), v)),
                    pretty::RcDoc::softline(),
                ))
                .append(text("};"))
                .append(
                    text(fastn_js::INHERITED_VARIABLE)
                        .append(space())
                        .append(text("="))
                        .append(space())
                        .append(format!(
                            "fastn_utils.getInheritedValues({}, {}, {});",
                            fastn_js::LOCAL_VARIABLE_MAP,
                            fastn_js::INHERITED_VARIABLE,
                            fastn_js::FUNCTION_ARGS
                        ))
                        .append(pretty::RcDoc::softline())
                        .append(text(fastn_js::LOCAL_VARIABLE_MAP))
                        .append(space())
                        .append(text("="))
                        .append(space())
                        .append(format!(
                            "fastn_utils.getArgs({}, {});",
                            fastn_js::LOCAL_VARIABLE_MAP,
                            fastn_js::FUNCTION_ARGS,
                        ))
                        .append(pretty::RcDoc::softline())
                        .append(pretty::RcDoc::intersperse(
                            local_arguments.iter().map(|(k, v)| {
                                format!(
                                    indoc::indoc! {
                                        "{l}.{k} =  {l}.{k}? {l}.{k}: {v};"
                                    },
                                    l = fastn_js::LOCAL_VARIABLE_MAP,
                                    v = v.to_js(),
                                    k = fastn_js::utils::name_to_js_(k)
                                )
                            }),
                            pretty::RcDoc::softline(),
                        ))
                        .append(pretty::RcDoc::softline())
                        .append(pretty::RcDoc::intersperse(
                            local_arguments_dependent.iter().map(|(k, v)| {
                                format!(
                                    indoc::indoc! {
                                        "{l}.{k} =  {l}.{k}? {l}.{k}: {v};"
                                    },
                                    l = fastn_js::LOCAL_VARIABLE_MAP,
                                    v = v.to_js(),
                                    k = fastn_js::utils::name_to_js_(k)
                                )
                            }),
                            pretty::RcDoc::softline(),
                        ))
                        .append(pretty::RcDoc::softline()),
                )
        }
        .append(
            pretty::RcDoc::intersperse(
                self.body.iter().map(|f| f.to_js()),
                pretty::RcDoc::softline(),
            )
            .group(),
        );

        func(self.name.as_str(), &self.params, body, package_name, false)
    }
}

impl fastn_js::MutableVariable {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        variable_to_js(
            self.name.as_str(),
            &self.prefix,
            text("fastn.mutable(")
                .append(text(&self.value.to_js()))
                .append(text(")")),
            false,
        )
    }
}

impl fastn_js::MutableList {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        variable_to_js(
            self.name.as_str(),
            &self.prefix,
            text(self.value.to_js().as_str()),
            false,
        )
    }
}

impl fastn_js::RecordInstance {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        variable_to_js(
            self.name.as_str(),
            &self.prefix,
            text(self.fields.to_js().as_str()),
            false,
        )
    }
}

impl fastn_js::OrType {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        variable_to_js(
            self.name.as_str(),
            &self.prefix,
            text(self.variant.to_js().as_str()),
            false,
        )
    }
}

impl fastn_js::StaticVariable {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        let mut value = self.value.to_js();
        value = value.replace("__DOT__", ".").replace("__COMMA__", ",");
        variable_to_js(
            self.name.as_str(),
            &self.prefix,
            text(value.as_str()),
            false,
        )
    }
}

fn variable_to_js(
    variable_name: &str,
    prefix: &Option<String>,
    value: pretty::RcDoc<'static>,
    add_global: bool,
) -> pretty::RcDoc<'static> {
    let name = {
        let (doc_name, remaining) = fastn_js::utils::get_doc_name_and_remaining(variable_name);
        let mut name = fastn_js::utils::name_to_js(doc_name.as_str());
        if let Some(remaining) = remaining {
            let remaining = if fastn_js::utils::is_asset_path(doc_name.as_str()) {
                remaining.replace('.', "_")
            } else {
                remaining
            };
            name = format!(
                "{}.{}",
                name,
                fastn_js::utils::kebab_to_snake_case(remaining.as_str())
            );
        }
        name
    };

    if let Some(ref prefix) = prefix {
        text(format!("fastn_utils.createNestedObject({}, \"{}\",", prefix, name,).as_str())
            .append(value)
            .append(text(");"))
    } else {
        if name.contains('.') {
            // `.` means the variable is placed in object so no need of `let`.
            // e.g: ftd.device
            pretty::RcDoc::nil()
        } else {
            text("let").append(space())
        }
        .append(text(name.as_str()))
        .append(space())
        .append(text("="))
        .append(space())
        .append(value)
        .append(text(";"))
        .append(if add_global {
            pretty::RcDoc::softline().append(text(
                format!("{}[\"{name}\"] = {name};", fastn_js::GLOBAL_VARIABLE_MAP).as_str(),
            ))
        } else {
            pretty::RcDoc::nil()
        })
    }
}

impl fastn_js::DeviceType {
    pub fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            fastn_js::DeviceType::Desktop => text("\"desktop\""),
            fastn_js::DeviceType::Mobile => text("\"mobile\""),
        }
    }
}

impl fastn_js::UDF {
    pub fn to_js(&self, package_name: &str) -> pretty::RcDoc<'static> {
        use itertools::Itertools;

        let body = text("let")
            .append(space())
            .append(text(fastn_js::LOCAL_VARIABLE_MAP))
            .append(space())
            .append(text("="))
            .append(space())
            .append(text("fastn_utils.getArgs("))
            .append(text("{"))
            .append(pretty::RcDoc::intersperse(
                self.args.iter().filter_map(|(k, v)| {
                    if v.is_undefined() {
                        None
                    } else {
                        Some(format!(
                            "{}: {},",
                            fastn_js::utils::name_to_js_(k),
                            v.to_js()
                        ))
                    }
                }),
                pretty::RcDoc::softline(),
            ))
            .append(text("}"))
            .append(format!(", {});", fastn_js::FUNCTION_ARGS))
            .append(pretty::RcDoc::intersperse(
                self.body.iter().map(|f| {
                    pretty::RcDoc::text(
                        fastn_js::to_js::ExpressionGenerator.to_js_(
                            f,
                            true,
                            self.args
                                .iter()
                                .map(|v| {
                                    (
                                        v.0.to_string(),
                                        Some(fastn_js::LOCAL_VARIABLE_MAP.to_string()),
                                    )
                                })
                                .collect_vec()
                                .as_slice(),
                            false,
                        ),
                    )
                }),
                pretty::RcDoc::softline(),
            ));

        func(
            self.name.as_str(),
            &self.params,
            body,
            package_name,
            self.is_external_js_present,
        )
    }
}

/*fn binary(op: &str, left: &UDFStatement, right: &UDFStatement) -> pretty::RcDoc<'static> {
    left.to_js()
        .append(space())
        .append(text(op))
        .append(space())
        .append(right.to_js())
}

impl UDFStatement {
    fn to_js(&self) -> pretty::RcDoc<'static> {
        match self {
            UDFStatement::Integer { value } => text(&value.to_string()),
            UDFStatement::Decimal { value } => text(&value.to_string()),
            UDFStatement::Boolean { value } => text(&value.to_string()),
            UDFStatement::String { value } => quote(value.as_str()),
            UDFStatement::Return { value } => text("return")
                .append(space())
                .append(value.to_js())
                .append(text(";")),
            UDFStatement::VariableDeclaration { name, value } => text("let")
                .append(space())
                .append(text(name.as_str()))
                .append(space())
                .append(text("="))
                .append(space())
                .append(value.to_js())
                .append(text(";")),
            UDFStatement::VariableAssignment { name, value } => text(name.as_str())
                .append(space())
                .append(text("="))
                .append(space())
                .append(value.to_js())
                .append(text(";")),
            UDFStatement::Addition { left, right } => binary("+", left, right),
            UDFStatement::Subtraction { left, right } => binary("-", left, right),
            UDFStatement::Multiplication { left, right } => binary("*", left, right),
            UDFStatement::Division { left, right } => binary("/", left, right),
            UDFStatement::Exponentiation { left, right } => binary("**", left, right),
            UDFStatement::And { left, right } => binary("&&", left, right),
            UDFStatement::Or { left, right } => binary("||", left, right),
            UDFStatement::Not { value } => text("!").append(value.to_js()),
            UDFStatement::Parens { value } => text("(").append(value.to_js()).append(text(")")),
            UDFStatement::Variable { name } => text(name.as_str()),
            UDFStatement::Ternary {
                condition,
                then,
                otherwise,
            } => condition
                .to_js()
                .append(space())
                .append(text("?"))
                .append(space())
                .append(then.to_js())
                .append(space())
                .append(text(":"))
                .append(space())
                .append(otherwise.to_js()),
            UDFStatement::If {
                condition,
                then,
                otherwise,
            } => text("if")
                .append(space())
                .append(text("("))
                .append(condition.to_js())
                .append(text(")"))
                .append(space())
                .append(text("{"))
                .append(then.to_js())
                .append(text("}"))
                .append(space())
                .append(text("else"))
                .append(space())
                .append(text("{"))
                .append(otherwise.to_js())
                .append(text("}")),
            UDFStatement::Call { name, args } => text(name.as_str())
                .append(text("("))
                .append(
                    pretty::RcDoc::intersperse(
                        args.iter().map(|f| f.to_js()),
                        comma().append(space()),
                    )
                    .group(),
                )
                .append(text(")")),
            UDFStatement::Block { .. } => todo!(),
        }
    }
}
*/

pub struct ExpressionGenerator;

impl ExpressionGenerator {
    pub fn to_js(&self, node: &fastn_grammar::evalexpr::ExprNode) -> String {
        self.to_js_(node, true, &[], false)
    }

    pub fn to_js_(
        &self,
        node: &fastn_grammar::evalexpr::ExprNode,
        root: bool,
        arguments: &[(String, Option<String>)],
        no_getter: bool,
    ) -> String {
        use itertools::Itertools;

        if self.is_root(node.operator()) {
            let result = node
                .children()
                .iter()
                .map(|children| self.to_js_(children, false, arguments, no_getter))
                .collect_vec();
            let (is_assignment_or_chain, only_one_child) =
                node.children().first().map_or((false, true), |first| {
                    /*has_operator(dbg!(&first.operator())).is_none()*/
                    let is_assignment_or_chain =
                        self.is_assignment(first.operator()) || self.is_chain(first.operator());
                    (
                        is_assignment_or_chain,
                        is_assignment_or_chain
                            || self.has_value(first.operator()).is_some()
                            || self.is_tuple(first.operator()),
                    )
                });
            let f = if only_one_child {
                result.join("")
            } else {
                format!("({})", result.join(""))
            };

            return if root && !is_assignment_or_chain && !f.is_empty() {
                format!("return {};", f)
            } else {
                f
            };
        }

        if self.is_chain(node.operator()) {
            let mut result = vec![];
            for children in node.children() {
                let val = fastn_js::utils::trim_brackets(
                    self.to_js_(children, true, arguments, false).trim(),
                );
                if !val.trim().is_empty() {
                    result.push(format!(
                        "{}{}",
                        val,
                        if val.ends_with(';') { "" } else { ";" }
                    ));
                }
            }
            return result.join("\n");
        }

        if self.is_tuple(node.operator()) {
            let mut result = vec![];
            for children in node.children() {
                result.push(self.to_js_(children, false, arguments, no_getter));
            }
            return format!("[{}]", result.join(","));
        }

        if let Some(function_name) = self.function_name(node.operator()) {
            let mut result = vec![];
            if let Some(child) = node.children().first() {
                for children in child.children() {
                    let mut value = self.to_js_(children, false, arguments, true);
                    if self.is_tuple(children.operator()) {
                        value = value[1..value.len() - 1].to_string();
                    }
                    result.push(value);
                }
            }
            return format!("{}({})", function_name, result.join(","));
        }

        if self.is_assignment(node.operator()) {
            // Todo: if node.children().len() != 2 {throw error}
            let first = node.children().first().unwrap(); //todo remove unwrap()
            let second = node.children().get(1).unwrap(); //todo remove unwrap()
            if arguments.iter().any(|v| first.to_string().eq(&v.0)) {
                let var = self.to_js_(first, false, arguments, false);
                let val = self.to_js_(second, false, arguments, true);
                return format!(
                    indoc::indoc! {
                        "let fastn_utils_val_{refined_var} = fastn_utils.clone({val});
                        if (fastn_utils_val_{refined_var} instanceof fastn.mutableClass) {{
                            fastn_utils_val_{refined_var} = fastn_utils_val_{refined_var}.get();
                        }}
                        if (!fastn_utils.setter({var}, fastn_utils_val_{refined_var})) {{
                            {var} = fastn_utils_val_{refined_var};
                        }}"
                    },
                    val = val,
                    var = var,
                    refined_var = fastn_js::utils::name_to_js_(var.as_str())
                );
            } else if first.operator().get_variable_identifier_write().is_some() {
                return [
                    "let ".to_string(),
                    self.to_js_(first, false, arguments, false),
                    node.operator().to_string(),
                    self.to_js_(second, false, arguments, false),
                ]
                .join("");
            };
            return [
                self.to_js_(first, false, arguments, false),
                node.operator().to_string(),
                self.to_js_(second, false, arguments, false),
            ]
            .join("");
        }

        if let Some(mut operator) = self.has_operator(node.operator()) {
            // Todo: if node.children().len() != 2 {throw error}
            let first = node.children().first().unwrap(); //todo remove unwrap()
            if matches!(node.operator(), fastn_grammar::evalexpr::Operator::Not)
                || matches!(node.operator(), fastn_grammar::evalexpr::Operator::Neg)
            {
                return [operator, self.to_js_(first, false, arguments, false)].join("");
            }
            if matches!(node.operator(), fastn_grammar::evalexpr::Operator::Neq) {
                // For js conversion
                operator = "!==".to_string();
            }
            let second = node.children().get(1).unwrap(); //todo remove unwrap()
            return [
                self.to_js_(first, false, arguments, false),
                operator,
                self.to_js_(second, false, arguments, false),
            ]
            .join("");
        }

        if let Some(operator) = self.has_function(node.operator()) {
            let mut result = vec![];
            for children in node.children() {
                result.push(self.to_js_(children, false, arguments, false));
            }
            return format!("{}{}", operator.trim(), result.join(" "));
        }

        let value = if self.is_null(node.operator()) {
            "null".to_string()
        } else {
            let value = node.operator().to_string();
            let prefix = arguments
                .iter()
                .find_map(|v| {
                    if value.to_string().eq(&v.0) || value.starts_with(format!("{}.", v.0).as_str())
                    {
                        v.1.clone()
                    } else {
                        None
                    }
                })
                .map(|v| format!("{}.", v))
                .unwrap_or_default();
            format!("{}{}", prefix, value)
        };

        if node.operator().get_variable_identifier_read().is_some() && !no_getter {
            let chain_dot_operator_count = value.matches('.').count();
            // When there are chained dot operator value
            // like person.name, person.meta.address
            if chain_dot_operator_count > 1 {
                return format!(
                    "fastn_utils.getStaticValue({})",
                    get_chained_getter_string(value.as_str())
                );
            }

            // When there is no chained dot operator value
            format!("fastn_utils.getStaticValue({})", value)
        } else {
            value
        }
    }

    pub fn has_value(&self, operator: &fastn_grammar::evalexpr::Operator) -> Option<String> {
        match operator {
            fastn_grammar::evalexpr::Operator::Const { .. }
            | fastn_grammar::evalexpr::Operator::VariableIdentifierRead { .. }
            | fastn_grammar::evalexpr::Operator::VariableIdentifierWrite { .. } => {
                Some(operator.to_string())
            }
            _ => None,
        }
    }

    pub fn has_function(&self, operator: &fastn_grammar::evalexpr::Operator) -> Option<String> {
        match operator {
            fastn_grammar::evalexpr::Operator::FunctionIdentifier { .. } => {
                Some(operator.to_string())
            }
            _ => None,
        }
    }

    pub fn is_assignment(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(operator, fastn_grammar::evalexpr::Operator::Assign)
    }

    pub fn is_chain(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(operator, fastn_grammar::evalexpr::Operator::Chain)
    }

    pub fn is_tuple(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(operator, fastn_grammar::evalexpr::Operator::Tuple)
    }

    pub fn is_null(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(
            operator,
            fastn_grammar::evalexpr::Operator::Const {
                value: fastn_grammar::evalexpr::Value::Empty,
            }
        )
    }

    pub fn function_name(&self, operator: &fastn_grammar::evalexpr::Operator) -> Option<String> {
        if let fastn_grammar::evalexpr::Operator::FunctionIdentifier { identifier } = operator {
            Some(identifier.to_string())
        } else {
            None
        }
    }

    pub fn has_operator(&self, operator: &fastn_grammar::evalexpr::Operator) -> Option<String> {
        if self.has_value(operator).is_none()
            && self.has_function(operator).is_none()
            && !self.is_chain(operator)
            && !self.is_root(operator)
            && !self.is_tuple(operator)
            && !self.is_assignment(operator)
        {
            Some(operator.to_string())
        } else {
            None
        }
    }

    pub fn is_root(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(operator, fastn_grammar::evalexpr::Operator::RootNode)
    }
}

pub fn get_chained_getter_string(value: &str) -> String {
    let chain_dot_operator_count = value.matches('.').count();
    if chain_dot_operator_count > 1 {
        if let Some((variable, key)) = value.rsplit_once('.') {
            // Ignore values which are already resolved with get()
            if key.contains("get") {
                return value.to_string();
            }
            return format!(
                "fastn_utils.getterByKey({}, \"{}\")",
                get_chained_getter_string(variable),
                key.replace('-', "_") // record fields are stored in snake case
            );
        }
    }
    value.to_string()
}

#[cfg(test)]
#[track_caller]
pub fn e(f: fastn_js::Ast, s: &str) {
    let g = to_js(&[f], "foo");
    println!("got: {}", g);
    println!("expected: {}", s);
    assert_eq!(g, s);
}

#[cfg(test)]
mod tests {
    /*
    #[test]
    fn udf() {
        fastn_js::to_js::e(fastn_js::udf0("foo", vec![]), "function foo() {}");
        fastn_js::to_js::e(fastn_js::udf1("foo", "p", vec![]), "function foo(p) {}");
        fastn_js::to_js::e(
            fastn_js::udf2("foo", "p", "q", vec![]),
            "function foo(p, q) {}",
        );

        fastn_js::to_js::e(
            fastn_js::udf0(
                "foo",
                vec![fastn_js::UDFStatement::Return {
                    value: Box::new(fastn_js::UDFStatement::Integer { value: 10 }),
                }],
            ),
            "function foo() {return 10;}",
        );
        fastn_js::to_js::e(
            fastn_js::udf0(
                "foo",
                vec![fastn_js::UDFStatement::Return {
                    value: Box::new(fastn_js::UDFStatement::Decimal { value: 10.1 }),
                }],
            ),
            "function foo() {return 10.1;}",
        );
        fastn_js::to_js::e(
            fastn_js::udf0(
                "foo",
                vec![fastn_js::UDFStatement::Return {
                    value: Box::new(fastn_js::UDFStatement::Boolean { value: true }),
                }],
            ),
            "function foo() {return true;}",
        );
        fastn_js::to_js::e(
            fastn_js::udf0(
                "foo",
                vec![fastn_js::UDFStatement::Return {
                    value: Box::new(fastn_js::UDFStatement::String {
                        value: "hello".to_string(),
                    }),
                }],
            ),
            r#"function foo() {return "hello";}"#,
        );
        fastn_js::to_js::e(
            fastn_js::udf0(
                "foo",
                vec![fastn_js::UDFStatement::Call {
                    name: "bar".to_string(),
                    args: vec![fastn_js::UDFStatement::String {
                        value: "hello".to_string(),
                    }],
                }],
            ),
            r#"function foo() {bar("hello")}"#,
        );
    }*/
    #[test]
    #[ignore]
    fn test_func() {
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![]),
            "function foo(parent) {}",
        );
        fastn_js::to_js::e(
            fastn_js::component1("foo", "p", vec![]),
            "function foo(parent, p) {}",
        );
        fastn_js::to_js::e(
            fastn_js::component2("foo", "p", "q", vec![]),
            "function foo(parent, p, q) {}",
        );
    }

    #[test]
    #[ignore]
    fn unquoted() {
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_integer("bar", 10)]),
            r#"function foo(parent) {let bar;bar = fastn.mutable(10);}"#,
        );
    }

    #[test]
    #[ignore]
    fn quoted() {
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_string("bar", "10")]),
            r#"function foo(parent) {let bar;bar = fastn.mutable("10");}"#,
        );
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_string("bar", "hello world")]),
            r#"function foo(parent) {let bar;bar = fastn.mutable("hello world");}"#,
        );
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_string("bar", "hello world, a long long long long long string which keeps going on and on and on and on till we run out of line space and still keeps going on and on")]),
            indoc::indoc!(
                r#"function foo(parent) {
                let bar;bar = fastn.mutable("hello world, a long long long long long string which keeps going on and on and on and on till we run out of line space and still keeps going on and on");
                }"#),
        );
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::mutable_string("bar", "hello\nworld")]),
            r#"function foo(parent) {let bar;bar = fastn.mutable("hello\nworld");}"#,
        );
        // std::fs::write(
        //     "test.js",
        //     r#"function foo(parent) {let bar = "hello\nworld";}"#,
        // )
        // .unwrap();
    }

    #[test]
    #[ignore]
    fn static_unquoted() {
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::static_integer("bar", 10)]),
            r#"function foo(parent) {let bar;bar = 10;}"#,
        );
    }

    #[test]
    #[ignore]
    fn static_quoted() {
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::static_string("bar", "10")]),
            r#"function foo(parent) {let bar;bar = "10";}"#,
        );
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::static_string("bar", "hello world")]),
            r#"function foo(parent) {let bar;bar = "hello world";}"#,
        );
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::static_string("bar", "hello world, a long long long long long string which keeps going on and on and on and on till we run out of line space and still keeps going on and on")]),
            indoc::indoc!(
                r#"function foo(parent) {
                let bar;bar = "hello world, a long long long long long string which keeps going on and on and on and on till we run out of line space and still keeps going on and on";
                }"#),
        );
        fastn_js::to_js::e(
            fastn_js::component0("foo", vec![fastn_js::static_string("bar", "hello\nworld")]),
            r#"function foo(parent) {let bar;bar = "hello\nworld";}"#,
        );
        // std::fs::write(
        //     "test.js",
        //     r#"function foo(parent) {let bar = "hello\nworld";}"#,
        // )
        // .unwrap();
    }
}
