use base64::{prelude::BASE64_STANDARD, Engine};
use lazy_static::lazy_static;
use sea_orm::JsonValue;
use std::collections::HashMap;
use tera::{Filter, Tera, Test, Value};

struct IsNaN;
impl Test for IsNaN {
    fn test(&self, value: Option<&Value>, args: &[Value]) -> tera::Result<bool> {
        if let Some(Value::Number(n)) = value {
            Ok(n.as_f64().map_or(false, |n| n.is_nan()))
        } else {
            Ok(false)
        }
    }
}

struct AsPercent;

impl Filter for AsPercent {
    fn filter(&self, value: &Value, _args: &HashMap<String, JsonValue>) -> tera::Result<Value> {
        if let Value::Number(n) = value {
            let n = n.as_f64().unwrap();
            Ok(format!("{:.2}%", n * 100.0).into())
        } else {
            Ok("-".into())
        }
    }
}

struct AsBlob;

impl Filter for AsBlob {
    fn filter(&self, value: &Value, _args: &HashMap<String, JsonValue>) -> tera::Result<Value> {
        if let Value::Array(arr) = value {
            let mut bytes = vec![];
            for v in arr {
                if let Value::Number(n) = v {
                    bytes.push(n.as_u64().unwrap() as u8);
                }
                else {
                    return Err("Invalid value".into());
                }
            }
            // Convert to base64
            let encoded = BASE64_STANDARD.encode(&bytes);
            let encoded = format!("data:image;base64,{}", encoded);
            Ok(encoded.into())
        } else {
            Ok(Value::Null)
        }
    }
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let source = "templates/**/*.html";
        let mut tera = Tera::new(source).expect("failed to compile template");
        tera.autoescape_on(vec![".html", ".sql"]);
        tera.register_filter("as_percent", AsPercent);
        tera.register_filter("as_blob", AsBlob);
        tera
    };
}
