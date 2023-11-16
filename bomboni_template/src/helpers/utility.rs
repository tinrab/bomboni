use handlebars::{Helper, RenderError};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub fn get_param<T: DeserializeOwned>(
    h: &Helper,
    index: usize,
    name: &str,
) -> Result<T, RenderError> {
    get_param_opt(h, index, name)?.ok_or_else(|| {
        RenderError::new(format!(
            "`{}` param not provided for helper `{}`",
            name,
            h.name()
        ))
    })
}

pub fn get_param_opt<T: DeserializeOwned>(
    h: &Helper,
    index: usize,
    name: &str,
) -> Result<Option<T>, RenderError> {
    if let Some(param) = h.param(index).map(|param| param.value().clone()) {
        let value = serde_json::from_value(param).map_err(|_| {
            RenderError::new(format!(
                "cannot parse param `{}` for helper `{}`",
                name,
                h.name(),
            ))
        })?;
        Ok(value)
    } else {
        Ok(None)
    }
}

pub fn get_hash<T: DeserializeOwned>(h: &Helper, name: &str) -> Result<T, RenderError> {
    get_hash_opt(h, name)?.ok_or_else(|| {
        RenderError::new(format!(
            "`{}` param not provided for helper `{}`",
            name,
            h.name()
        ))
    })
}

pub fn get_hash_opt<T: DeserializeOwned>(h: &Helper, name: &str) -> Result<Option<T>, RenderError> {
    if let Some(param) = h.hash_get(name).map(|param| param.value().clone()) {
        let value = serde_json::from_value(param).map_err(|_| {
            RenderError::new(format!(
                "`{}` param not provided for helper `{}`",
                name,
                h.name()
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
) -> Result<&'a Value, RenderError> {
    get_param_value_opt(h, index).ok_or_else(|| {
        RenderError::new(format!(
            "`{}` param not provided for helper `{}`",
            name,
            h.name()
        ))
    })
}

#[must_use]
pub fn get_param_value_opt<'a>(h: &'a Helper, index: usize) -> Option<&'a Value> {
    h.param(index).map(handlebars::PathAndJson::value)
}

// pub fn update_block_context(
//     block: &mut BlockContext<'_>,
//     base_path: Option<&Vec<String>>,
//     relative_path: String,
//     is_first: bool,
//     value: &Value,
// ) {
//     if let Some(p) = base_path {
//         if is_first {
//             *block.base_path_mut() = copy_on_push_vec(p, relative_path);
//         } else if let Some(ptr) = block.base_path_mut().last_mut() {
//             *ptr = relative_path;
//         }
//     } else {
//         block.set_base_value(value.clone());
//     }
// }

// pub fn set_block_param<'reg>(
//     block: &mut BlockContext<'reg>,
//     h: &Helper<'reg, '_>,
//     base_path: Option<&Vec<String>>,
//     k: &Value,
//     v: &Value,
// ) -> Result<(), RenderError> {
//     if let Some(bp_val) = h.block_param() {
//         let mut params = BlockParams::new();
//         if base_path.is_some() {
//             params.add_path(bp_val, Vec::with_capacity(0))?;
//         } else {
//             params.add_value(bp_val, v.clone())?;
//         }

//         block.set_block_params(params);
//     } else if let Some((bp_val, bp_key)) = h.block_param_pair() {
//         let mut params = BlockParams::new();
//         if base_path.is_some() {
//             params.add_path(bp_val, Vec::with_capacity(0))?;
//         } else {
//             params.add_value(bp_val, v.clone())?;
//         }
//         params.add_value(bp_key, k.clone())?;

//         block.set_block_params(params);
//     }

//     Ok(())
// }

// pub fn copy_on_push_vec<T>(input: &[T], el: T) -> Vec<T>
// where
//     T: Clone,
// {
//     let mut new_vec = Vec::with_capacity(input.len() + 1);
//     new_vec.extend_from_slice(input);
//     new_vec.push(el);
//     new_vec
// }

// pub fn create_block<'reg>(param: &PathAndJson<'reg, '_>) -> BlockContext<'reg> {
//     let mut block = BlockContext::new();

//     if let Some(new_path) = param.context_path() {
//         *block.base_path_mut() = new_path.clone();
//     } else {
//         // use clone for now
//         block.set_base_value(param.value().clone());
//     }

//     block
// }
