use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};

use serde_json::Value;

#[derive(Clone, Copy)]
pub struct ReverseHelper;

impl HelperDef for ReverseHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<handlebars::ScopedJson<'reg, 'rc>, RenderError> {
        match h.param(0).map(|p| p.value()) {
            None => Err(RenderError::new(format!(
                "Insufficient arguments, expected one, got {}",
                h.params().len()
            ))),
            Some(Value::Array(list)) => Ok(ScopedJson::Derived(Value::Array(
                list.into_iter().cloned().rev().collect(),
            ))),
            Some(other) => Err(RenderError::new(format!(
                "Expected array as argument, got {}",
                json_type_name(other)
            ))),
        }
    }
}

fn json_type_name(json: &Value) -> &'static str {
    match json {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}