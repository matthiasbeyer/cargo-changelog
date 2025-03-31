use handlebars::{
    Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, RenderErrorReason,
    ScopedJson,
};

use serde_json::Value;

#[derive(Clone, Copy)]
pub struct ReverseHelper;

impl HelperDef for ReverseHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<handlebars::ScopedJson<'rc>, RenderError> {
        match h.param(0).map(|p| p.value()) {
            None => Err(RenderErrorReason::ParamNotFoundForIndex("reverse", 0).into()),
            Some(Value::Array(list)) => Ok(ScopedJson::Derived(Value::Array(
                list.iter().cloned().rev().collect(),
            ))),
            Some(_other) => Err(RenderErrorReason::InvalidParamType("array").into()),
        }
    }
}
