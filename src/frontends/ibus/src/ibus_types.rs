//! Construction of the IBus serializable wire types used by the
//! `org.freedesktop.IBus.Engine` signals.
//!
//! Every IBus serializable object is marshalled as a D-Bus struct whose first
//! two fields are the GType name (`s`) and an attachments dictionary (`a{sv}`),
//! followed by the type specific fields. We only need two of them and always
//! produce them empty/attribute-less:
//!
//! * `IBusText`     -> `(sa{sv}sv)` = (name, attachments, text, attrs-variant)
//! * `IBusAttrList` -> `(sa{sv}av)` = (name, attachments, attributes)
//!
//! The attribute list is wrapped in a variant (`v`) inside `IBusText`, matching
//! `ibus_text_serialize()` in upstream IBus. We force that nesting with
//! `Value::Value(Box::new(..))`, since a plain `Value::Structure` field would be
//! marshalled inline instead of as a variant.

use std::collections::HashMap;
use zbus::zvariant::Value;

/// Build an empty `IBusAttrList` (no attributes).
fn ibus_attr_list() -> Value<'static> {
    let v: Value = (
        "IBusAttrList".to_string(),
        HashMap::<String, Value>::new(), // a{sv} attachments
        Vec::<Value>::new(),             // av  attributes
    )
        .into();
    v
}

/// Build an `IBusText` wrapping `text` with an empty attribute list.
///
/// The returned [`Value`] is a `Value::Structure` with signature `(sa{sv}sv)`.
/// When passed as a single signal argument it is marshalled as a variant (`v`),
/// which is exactly what `CommitText`/`UpdatePreeditText` expect.
pub fn ibus_text(text: &str) -> Value<'static> {
    // The `attrs` field must be a variant wrapping the IBusAttrList struct.
    let attrs = Value::Value(Box::new(ibus_attr_list()));
    let v: Value = (
        "IBusText".to_string(),
        HashMap::<String, Value>::new(), // a{sv} attachments
        text.to_string(),                // s text
        attrs,                           // v attrs (IBusAttrList)
    )
        .into();
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::{serialized::Context, to_bytes, LE};

    #[test]
    fn attr_list_signature() {
        assert_eq!(ibus_attr_list().value_signature().to_string(), "(sa{sv}av)");
    }

    #[test]
    fn text_signature() {
        // IBusText marshals as the documented (sa{sv}sv) struct.
        assert_eq!(ibus_text("가").value_signature().to_string(), "(sa{sv}sv)");
    }

    #[test]
    fn text_round_trips_as_variant() {
        // Emitting it as a single signal arg must produce a variant (`v`) whose
        // inner value is the IBusText struct, and that must demarshal back.
        let ctxt = Context::new_dbus(LE, 0);
        let encoded = to_bytes(ctxt, &(ibus_text("한글"),)).unwrap();
        let (decoded, _): ((Value,), _) = encoded.deserialize().unwrap();
        assert_eq!(decoded.0.value_signature().to_string(), "(sa{sv}sv)");
    }
}
