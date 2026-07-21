use serde_json::Value;

pub fn format_value(value: &Value, raw: bool, raw_multiline: bool) -> String {
    if raw {
        match value {
            Value::String(s) if raw_multiline => return s.clone(),
            Value::String(s) if !s.contains('\n') => return s.clone(),
            Value::String(s) if s.ends_with('\n') && s.matches('\n').count() == 1 => {
                return s.trim_end_matches('\n').to_string()
            }
            Value::Number(n) => return n.to_string(),
            Value::Bool(b) => return b.to_string(),
            _ => {}
        }
    }

    value.to_string()
}
