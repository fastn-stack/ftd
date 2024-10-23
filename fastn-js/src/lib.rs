#![deny(unused_crate_dependencies)]

extern crate self as fastn_js;

mod ast;
mod component;
mod component_invocation;
mod component_statement;
mod conditional_component;
mod constants;
mod device;
mod event;
mod loop_component;
mod markup;
mod mutable_variable;
mod or_type;
mod property;
mod record;
mod ssr;
mod static_variable;
mod to_js;
mod udf;
mod udf_statement;
pub mod utils;

pub use ast::Ast;
pub use component::{component0, component1, component2, component_with_params, Component};
pub use component_invocation::{
    ElementKind, InstantiateComponent, InstantiateComponentData, Kernel,
};
pub use component_statement::ComponentStatement;
pub use conditional_component::ConditionalComponent;
pub use constants::*;
pub use device::{DeviceBlock, DeviceType};
pub use event::{Event, EventHandler, Function, FunctionData};
pub use loop_component::ForLoop;
pub use mutable_variable::{mutable_integer, mutable_string, MutableList, MutableVariable};
pub use or_type::OrType;
pub use property::{
    ConditionalValue, Formula, FormulaType, PropertyKind, SetProperty, SetPropertyValue, Value,
};
pub use record::RecordInstance;
pub use ssr::{run_test, ssr, ssr_raw_string_without_test, ssr_str, ssr_with_js_string, SSRError};
pub use static_variable::{static_integer, static_string, StaticVariable};
pub use to_js::to_js;
pub use udf::{udf_with_arguments, UDF};
pub use udf_statement::UDFStatement;

pub fn fastn_assertion_headers(http_status_code: u16, http_location: &str) -> String {
    format!(
        indoc::indoc! {"
            fastn.http_status = {http_status};
            fastn.http_location = \"{http_location}\";
        "},
        http_status = http_status_code,
        http_location = http_location
    )
}

pub fn fastn_test_js() -> &'static str {
    include_str!("../js/fastn_test.js")
}

pub fn all_js_without_test_and_ftd_langugage_js() -> String {
    let fastn_js = include_str_with_debug!("../js/fastn.js");
    let dom_js = include_str_with_debug!("../js/dom.js");
    let utils_js = include_str_with_debug!("../js/utils.js");
    let virtual_js = include_str_with_debug!("../js/virtual.js");
    let ftd_js = include_str_with_debug!("../js/ftd.js");
    let web_component_js = include_str_with_debug!("../js/web-component.js");
    let post_init_js = include_str_with_debug!("../js/postInit.js");
    format!("{fastn_js}{dom_js}{utils_js}{virtual_js}{web_component_js}{ftd_js}{post_init_js}")
}

#[macro_export]
macro_rules! include_str_with_debug {
    ($name:expr) => {{
        let default = include_str!($name);
        if std::env::var("DEBUG").is_ok() {
            std::fs::read_to_string($name).unwrap_or_else(|_| default.to_string())
        } else {
            default.to_string()
        }
    }};
}

pub fn all_js_without_test() -> String {
    let fastn_js = all_js_without_test_and_ftd_langugage_js();
    let ftd_language_js = include_str!("../js/ftd-language.js");
    format!("{ftd_language_js}{fastn_js}\nwindow.ftd = ftd;\n")
}

pub fn all_js_with_test() -> String {
    let test_js = include_str!("../js/test.js");
    let all_js = all_js_without_test_and_ftd_langugage_js();
    format!("{all_js}{test_js}")
}

pub fn markdown_js() -> &'static str {
    include_str!("../marked.js")
}
