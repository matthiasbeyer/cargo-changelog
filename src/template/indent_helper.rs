use handlebars::{
    Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, RenderErrorReason,
    ScopedJson,
};

use serde_json::Value;

#[derive(Clone, Copy)]
pub struct IndentHelper;

impl HelperDef for IndentHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<handlebars::ScopedJson<'rc>, RenderError> {
        let body = match h.param(0).map(|p| p.value()) {
            Some(Value::String(body)) => body.to_string(),
            None => return Err(RenderErrorReason::ParamNotFoundForIndex("indent", 1).into()),
            Some(_other) => return Err(RenderErrorReason::InvalidParamType("body").into()),
        };

        let indent = match h.param(1).map(|p| p.value()) {
            Some(Value::String(indent)) => indent.to_string(),
            None => {
                let spaces_indent = h
                    .hash_get("spaces")
                    .map(|p| p.value())
                    .and_then(|v| v.as_number())
                    .and_then(|num| num.as_u64())
                    .ok_or_else(|| RenderErrorReason::InvalidParamType("spaces"))?;

                " ".repeat(spaces_indent as usize)
            }
            Some(_other) => return Err(RenderErrorReason::InvalidParamType("array").into()),
        };

        Ok(ScopedJson::Derived(Value::String(body.lines().fold(
            String::new(),
            |mut acc, line| {
                acc.push_str(&indent);
                acc.push_str(line);
                acc.push('\n');
                acc
            },
        ))))
    }
}
