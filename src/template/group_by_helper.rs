use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};

use itertools::Itertools;
use serde_json::Value;

#[derive(Clone, Copy)]
pub struct GroupByHelper;

impl HelperDef for GroupByHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<handlebars::ScopedJson<'reg, 'rc>, RenderError> {
        let group_by_attr = h
            .param(1)
            .map(|p| p.value())
            .ok_or_else(|| {
                RenderError::new(format!(
                    "Insufficient arguments, expected two, got {}",
                    h.params().len()
                ))
            })?
            .as_str()
            .ok_or_else(|| RenderError::new("Expected String as key to group by"))?;

        match h.param(0).map(|p| p.value()) {
            None => Err(RenderError::new(format!(
                "Insufficient arguments, expected two, got {}",
                h.params().len()
            ))),
            Some(Value::Array(list)) => {
                let mut res: serde_json::Map<String, _> = serde_json::Map::new();

                let object_list = list
                    .iter()
                    .map(|elt| match elt {
                        serde_json::Value::Object(map) => {
                            Ok(serde_json::Value::Object(map.clone()))
                        }
                        other => Err(RenderError::new(format!(
                            "At least one of the elements is not an object, but a {}",
                            crate::template::common::json_type_name(other)
                        ))),
                    })
                    .collect::<Result<Vec<serde_json::Value>, RenderError>>()?;

                for (group, list) in object_list
                    .into_iter()
                    .group_by(|elt: &serde_json::Value| {
                        elt.get("header")
                            .and_then(|hdr| hdr.get(&group_by_attr))
                            .cloned()
                    })
                    .into_iter()
                {
                    let list = list.into_iter().collect();
                    let group = group.ok_or_else(|| {
                        RenderError::new(format!("Failed to group by '{}', not all elements in the list have that attribute! List: {:?}", group_by_attr, list))
                    })?;

                    res.insert(group.to_string(), serde_json::Value::Array(list));
                }

                Ok(ScopedJson::Derived(serde_json::Value::from(res)))
            }
            Some(other) => Err(RenderError::new(format!(
                "Expected array as argument, got {}",
                crate::template::common::json_type_name(other)
            ))),
        }
    }
}
