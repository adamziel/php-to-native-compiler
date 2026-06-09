use std::mem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PayloadId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Value {
    Null,
    Int(i64),
    String(PayloadId),
    Array(PayloadId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RefCount {
    Counted(usize),
    Immortal,
}

#[derive(Clone, Debug)]
struct ArrayEntry {
    key: String,
    value: Value,
}

#[derive(Clone, Debug)]
enum PayloadKind {
    String(String),
    Array(Vec<ArrayEntry>),
}

#[derive(Clone, Debug)]
struct Payload {
    refcount: RefCount,
    freed: bool,
    kind: PayloadKind,
}

#[derive(Default)]
struct ContractStore {
    payloads: Vec<Payload>,
}

impl ContractStore {
    fn new_string(&mut self, data: &str) -> Value {
        self.push_payload(PayloadKind::String(data.to_string()), RefCount::Counted(1))
            .into_string_value()
    }

    fn immortal_string(&mut self, data: &str) -> Value {
        self.push_payload(PayloadKind::String(data.to_string()), RefCount::Immortal)
            .into_string_value()
    }

    fn new_array(&mut self, entries: Vec<(&str, Value)>) -> Value {
        let entries = entries
            .into_iter()
            .map(|(key, value)| ArrayEntry {
                key: key.to_string(),
                value,
            })
            .collect();
        self.push_payload(PayloadKind::Array(entries), RefCount::Counted(1))
            .into_array_value()
    }

    fn retain(&mut self, value: Value) -> Value {
        match value {
            Value::String(id) | Value::Array(id) => {
                let payload = &mut self.payloads[id.0];
                assert!(!payload.freed);
                if let RefCount::Counted(ref mut count) = payload.refcount {
                    *count += 1;
                }
            }
            Value::Null | Value::Int(_) => {}
        }
        value
    }

    fn release_slot(&mut self, slot: &mut Value) {
        let value = mem::replace(slot, Value::Null);
        self.release_owned(value);
    }

    fn write_slot(&mut self, slot: &mut Value, value: Value) {
        self.release_slot(slot);
        *slot = value;
    }

    fn append_string(&mut self, slot: &mut Value, suffix: &str) {
        let id = self.detach_string_for_write(slot);
        let PayloadKind::String(data) = &mut self.payloads[id.0].kind else {
            panic!("string slot should reference a string payload");
        };
        data.push_str(suffix);
    }

    fn array_set(&mut self, slot: &mut Value, key: &str, value: Value) {
        let id = self.detach_array_for_write(slot);
        let old_value = {
            let PayloadKind::Array(entries) = &mut self.payloads[id.0].kind else {
                panic!("array slot should reference an array payload");
            };
            if let Some(entry) = entries.iter_mut().find(|entry| entry.key == key) {
                Some(mem::replace(&mut entry.value, value))
            } else {
                entries.push(ArrayEntry {
                    key: key.to_string(),
                    value,
                });
                None
            }
        };
        if let Some(old_value) = old_value {
            self.release_owned(old_value);
        }
    }

    fn detach_string_for_write(&mut self, slot: &mut Value) -> PayloadId {
        let Value::String(id) = *slot else {
            panic!("slot should hold a string");
        };
        if self.is_unique_mutable(id) {
            return id;
        }
        let copy = match &self.payloads[id.0].kind {
            PayloadKind::String(data) => data.clone(),
            PayloadKind::Array(_) => panic!("string slot should reference a string payload"),
        };
        let new_id = self.push_payload(PayloadKind::String(copy), RefCount::Counted(1));
        self.release_slot(slot);
        *slot = Value::String(new_id);
        new_id
    }

    fn detach_array_for_write(&mut self, slot: &mut Value) -> PayloadId {
        let Value::Array(id) = *slot else {
            panic!("slot should hold an array");
        };
        if self.is_unique_mutable(id) {
            return id;
        }
        let copied_entries = match &self.payloads[id.0].kind {
            PayloadKind::Array(entries) => entries.clone(),
            PayloadKind::String(_) => panic!("array slot should reference an array payload"),
        }
        .into_iter()
        .map(|entry| ArrayEntry {
            key: entry.key,
            value: self.retain(entry.value),
        })
        .collect();
        let new_id = self.push_payload(PayloadKind::Array(copied_entries), RefCount::Counted(1));
        self.release_slot(slot);
        *slot = Value::Array(new_id);
        new_id
    }

    fn refcount(&self, id: PayloadId) -> Option<usize> {
        match self.payloads[id.0].refcount {
            RefCount::Counted(count) => Some(count),
            RefCount::Immortal => None,
        }
    }

    fn string_data(&self, id: PayloadId) -> &str {
        let PayloadKind::String(data) = &self.payloads[id.0].kind else {
            panic!("payload should be a string");
        };
        data
    }

    fn array_entry_value(&self, id: PayloadId, key: &str) -> Value {
        let PayloadKind::Array(entries) = &self.payloads[id.0].kind else {
            panic!("payload should be an array");
        };
        entries
            .iter()
            .find(|entry| entry.key == key)
            .unwrap_or_else(|| panic!("array should contain key {key}"))
            .value
    }

    fn is_freed(&self, id: PayloadId) -> bool {
        self.payloads[id.0].freed
    }

    fn string_id(&self, value: Value) -> PayloadId {
        match value {
            Value::String(id) => id,
            _ => panic!("value should be a string"),
        }
    }

    fn array_id(&self, value: Value) -> PayloadId {
        match value {
            Value::Array(id) => id,
            _ => panic!("value should be an array"),
        }
    }

    fn push_payload(&mut self, kind: PayloadKind, refcount: RefCount) -> PayloadId {
        let id = PayloadId(self.payloads.len());
        self.payloads.push(Payload {
            refcount,
            freed: false,
            kind,
        });
        id
    }

    fn is_unique_mutable(&self, id: PayloadId) -> bool {
        let payload = &self.payloads[id.0];
        assert!(!payload.freed);
        matches!(payload.refcount, RefCount::Counted(1))
    }

    fn release_owned(&mut self, value: Value) {
        let id = match value {
            Value::String(id) | Value::Array(id) => id,
            Value::Null | Value::Int(_) => return,
        };
        let children = {
            let payload = &mut self.payloads[id.0];
            assert!(!payload.freed);
            match &mut payload.refcount {
                RefCount::Immortal => return,
                RefCount::Counted(count) => {
                    assert!(*count > 0);
                    *count -= 1;
                    if *count != 0 {
                        return;
                    }
                }
            }
            payload.freed = true;
            match &payload.kind {
                PayloadKind::String(_) => Vec::new(),
                PayloadKind::Array(entries) => entries.iter().map(|entry| entry.value).collect(),
            }
        };
        for child in children {
            self.release_owned(child);
        }
    }
}

trait PayloadIdExt {
    fn into_string_value(self) -> Value;
    fn into_array_value(self) -> Value;
}

impl PayloadIdExt for PayloadId {
    fn into_string_value(self) -> Value {
        Value::String(self)
    }

    fn into_array_value(self) -> Value {
        Value::Array(self)
    }
}

#[test]
fn assignment_shares_string_payload_until_first_write() {
    let mut store = ContractStore::default();
    let mut original = store.new_string("seed");
    let original_id = store.string_id(original);

    let mut alias = store.retain(original);
    assert_eq!(store.string_id(alias), original_id);
    assert_eq!(store.refcount(original_id), Some(2));

    store.append_string(&mut alias, "-changed");
    let alias_id = store.string_id(alias);
    assert_ne!(alias_id, original_id);
    assert_eq!(store.refcount(original_id), Some(1));
    assert_eq!(store.refcount(alias_id), Some(1));
    assert_eq!(store.string_data(original_id), "seed");
    assert_eq!(store.string_data(alias_id), "seed-changed");

    store.release_slot(&mut alias);
    assert!(store.is_freed(alias_id));
    assert_eq!(store.refcount(original_id), Some(1));
    store.release_slot(&mut original);
    assert!(store.is_freed(original_id));
}

#[test]
fn immutable_literal_detaches_before_write_without_refcounting() {
    let mut store = ContractStore::default();
    let literal = store.immortal_string("static");
    let literal_id = store.string_id(literal);

    let mut alias = store.retain(literal);
    assert_eq!(store.string_id(alias), literal_id);
    assert_eq!(store.refcount(literal_id), None);

    store.append_string(&mut alias, "-copy");
    let detached_id = store.string_id(alias);
    assert_ne!(detached_id, literal_id);
    assert_eq!(store.refcount(literal_id), None);
    assert_eq!(store.refcount(detached_id), Some(1));
    assert_eq!(store.string_data(literal_id), "static");
    assert_eq!(store.string_data(detached_id), "static-copy");

    store.release_slot(&mut alias);
    assert!(store.is_freed(detached_id));
    assert!(!store.is_freed(literal_id));
}

#[test]
fn array_detach_copies_outer_storage_and_keeps_nested_payloads_shared() {
    let mut store = ContractStore::default();
    let mut leaf = store.new_string("leaf");
    let leaf_id = store.string_id(leaf);
    let retained_leaf = store.retain(leaf);
    let mut array = store.new_array(vec![("child", retained_leaf)]);
    let array_id = store.array_id(array);

    let mut alias = store.retain(array);
    assert_eq!(store.array_id(alias), array_id);
    assert_eq!(store.refcount(array_id), Some(2));
    assert_eq!(store.refcount(leaf_id), Some(2));

    store.detach_array_for_write(&mut alias);
    let alias_id = store.array_id(alias);
    assert_ne!(alias_id, array_id);
    assert_eq!(store.refcount(array_id), Some(1));
    assert_eq!(store.refcount(alias_id), Some(1));
    assert_eq!(store.refcount(leaf_id), Some(3));
    assert_eq!(
        store.array_entry_value(array_id, "child"),
        Value::String(leaf_id)
    );
    assert_eq!(
        store.array_entry_value(alias_id, "child"),
        Value::String(leaf_id)
    );

    store.array_set(&mut alias, "child", Value::Int(9));
    assert_eq!(store.refcount(leaf_id), Some(2));
    assert_eq!(store.array_entry_value(alias_id, "child"), Value::Int(9));
    assert_eq!(
        store.array_entry_value(array_id, "child"),
        Value::String(leaf_id)
    );

    store.release_slot(&mut alias);
    assert!(store.is_freed(alias_id));
    assert_eq!(store.refcount(leaf_id), Some(2));
    store.release_slot(&mut array);
    assert!(store.is_freed(array_id));
    assert_eq!(store.refcount(leaf_id), Some(1));
    store.release_slot(&mut leaf);
    assert!(store.is_freed(leaf_id));
}

#[test]
fn slot_replacement_releases_overwritten_payload_once() {
    let mut store = ContractStore::default();
    let mut slot = store.new_string("old");
    let old_id = store.string_id(slot);
    let mut replacement_owner = store.new_string("new");
    let replacement_id = store.string_id(replacement_owner);
    let replacement_for_slot = store.retain(replacement_owner);

    store.write_slot(&mut slot, replacement_for_slot);
    assert!(store.is_freed(old_id));
    assert_eq!(store.refcount(replacement_id), Some(2));

    store.release_slot(&mut slot);
    assert_eq!(store.refcount(replacement_id), Some(1));
    store.release_slot(&mut replacement_owner);
    assert!(store.is_freed(replacement_id));
}

#[test]
fn call_arguments_and_temporaries_balance_owned_references() {
    let mut store = ContractStore::default();
    let mut variable = store.new_string("arg");
    let payload_id = store.string_id(variable);

    let mut argument_slot = store.retain(variable);
    assert_eq!(store.refcount(payload_id), Some(2));

    let mut func_get_arg_temporary = store.retain(argument_slot);
    assert_eq!(store.refcount(payload_id), Some(3));

    store.release_slot(&mut func_get_arg_temporary);
    assert_eq!(store.refcount(payload_id), Some(2));
    store.release_slot(&mut argument_slot);
    assert_eq!(store.refcount(payload_id), Some(1));
    store.release_slot(&mut variable);
    assert!(store.is_freed(payload_id));
}
