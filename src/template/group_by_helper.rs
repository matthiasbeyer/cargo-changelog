use handlebars::{
    Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, RenderErrorReason,
    ScopedJson,
};

use itertools::Itertools;
use serde_json::Value;

#[derive(Clone, Copy)]
pub struct GroupByHelper;

impl HelperDef for GroupByHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<handlebars::ScopedJson<'rc>, RenderError> {
        let group_by_attr = h
            .param(1)
            .map(|p| p.value())
            .ok_or_else(|| RenderErrorReason::ParamNotFoundForIndex("group_by_header", 1))?
            .as_str()
            .ok_or_else(|| RenderErrorReason::InvalidParamType("string"))?;

        let unknown_group = h
            .hash_get("default")
            .map(|p| p.value())
            .map(|v| {
                v.as_str()
                    .map(ToString::to_string)
                    .ok_or_else(|| RenderErrorReason::InvalidParamType("string"))
            })
            .transpose()?;

        match h.param(0).map(|p| p.value()) {
            None => Err(RenderErrorReason::ParamNotFoundForIndex("group_by_header", 0).into()),
            Some(Value::Array(list)) => {
                let mut res: serde_json::Map<String, _> = serde_json::Map::new();

                let object_list = list
                    .iter()
                    .map(|elt| match elt {
                        serde_json::Value::Object(map) => {
                            Ok(serde_json::Value::Object(map.clone()))
                        }
                        _other => Err(RenderErrorReason::InvalidParamType("array of object")),
                    })
                    .collect::<Result<Vec<serde_json::Value>, RenderErrorReason>>()?;

                for (group, list) in object_list
                    .into_iter()
                    .chunk_by(|elt: &serde_json::Value| {
                        elt.get("header")
                            .and_then(|hdr| hdr.get(group_by_attr))
                            .cloned()
                    })
                    .into_iter()
                {
                    let list = list.into_iter().collect();
                    let group = group.map(|v| v.to_string()).or_else(|| unknown_group.clone()).ok_or_else(|| {
                        RenderErrorReason::Other(format!("Failed to group by '{group_by_attr}', not all elements in the list have that attribute,\
                                and no default has been given! Use `group_by_header default=<name>` to set a default. List: {list:?}"))
                    })?;

                    res.insert(group, serde_json::Value::Array(list));
                }

                Ok(ScopedJson::Derived(serde_json::Value::from(res)))
            }
            Some(_other) => Err(RenderErrorReason::InvalidParamType("array of object").into()),
        }
    }
}
