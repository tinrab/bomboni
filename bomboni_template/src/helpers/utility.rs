use handlebars::{Helper, RenderErrorReason};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub fn get_param<T: DeserializeOwned>(
    h: &Helper,
    index: usize,
    name: &str,
) -> Result<T, RenderErrorReason> {
    get_param_opt(h, index, name)?.ok_or_else(|| {
        RenderErrorReason::Other(format!(
            "helper {} param at index {} with name {} not found",
            h.name(),
            index,
            name
        ))
    })
}

pub fn get_param_opt<T: DeserializeOwned>(
    h: &Helper,
    index: usize,
    name: &str,
) -> Result<Option<T>, RenderErrorReason> {
    if let Some(param) = h.param(index).map(|param| param.value().clone()) {
        let value = serde_json::from_value(param).map_err(|_| {
            RenderErrorReason::Other(format!(
                "helper {} param at index {} with name {} failed to parse",
                h.name(),
                index,
                name
            ))
        })?;
        Ok(value)
    } else {
        Ok(None)
    }
}

pub fn get_hash<T: DeserializeOwned>(h: &Helper, name: &str) -> Result<T, RenderErrorReason> {
    get_hash_opt(h, name)?.ok_or_else(|| {
        RenderErrorReason::Other(format!(
            "helper {} hash with name {} not found",
            h.name(),
            name
        ))
    })
}

pub fn get_hash_opt<T: DeserializeOwned>(
    h: &Helper,
    name: &str,
) -> Result<Option<T>, RenderErrorReason> {
    if let Some(param) = h.hash_get(name).map(|param| param.value().clone()) {
        let value = serde_json::from_value(param).map_err(|_| {
            RenderErrorReason::Other(format!(
                "helper {} hash with name {} failed to parse",
                h.name(),
                name
            ))
        })?;
        Ok(value)
    } else {
        Ok(None)
    }
}

pub fn get_param_value<'a>(
    h: &'a Helper,
    index: usize,
    name: &str,
) -> Result<&'a Value, RenderErrorReason> {
    get_param_value_opt(h, index).ok_or_else(|| {
        RenderErrorReason::Other(format!(
            "helper {} param at index {} with name {} not found",
            h.name(),
            name,
            index
        ))
    })
}

#[must_use]
pub fn get_param_value_opt<'a>(h: &'a Helper, index: usize) -> Option<&'a Value> {
    h.param(index).map(handlebars::PathAndJson::value)
}
