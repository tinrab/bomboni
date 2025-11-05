use handlebars::{Helper, RenderErrorReason};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Gets a required parameter from a helper.
///
/// # Errors
///
/// Returns a render error if the parameter is not found or cannot be deserialized.
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

/// Gets an optional parameter from a helper.
///
/// # Errors
///
/// Returns a render error if the parameter cannot be deserialized.
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

/// Gets a required hash parameter from a helper.
///
/// # Errors
///
/// Returns a render error if the hash parameter is not found or cannot be deserialized.
pub fn get_hash<T: DeserializeOwned>(h: &Helper, name: &str) -> Result<T, RenderErrorReason> {
    get_hash_opt(h, name)?.ok_or_else(|| {
        RenderErrorReason::Other(format!(
            "helper {} hash with name {} not found",
            h.name(),
            name
        ))
    })
}

/// Gets an optional hash parameter from a helper.
///
/// # Errors
///
/// Returns a render error if the hash parameter cannot be deserialized.
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

/// Gets a parameter value from a helper.
///
/// # Errors
///
/// Returns a render error if the parameter is not found.
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

/// Gets an optional parameter value from a helper.
#[must_use]
pub fn get_param_value_opt<'a>(h: &'a Helper, index: usize) -> Option<&'a Value> {
    h.param(index).map(handlebars::PathAndJson::value)
}
