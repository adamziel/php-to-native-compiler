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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Counters {
    array_detaches: usize,
    string_detaches: usize,
    payload_frees: usize,
}

#[derive(Default)]
struct ContractStore {
    payloads: Vec<Payload>,
    counters: Counters,
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

    fn array_set_path(&mut self, slot: &mut Value, path: &[&str], value: Value) {
        assert!(!path.is_empty());
        if path.len() == 1 {
            self.array_set(slot, path[0], value);
            return;
        }

        let id = self.detach_array_for_write(slot);
        let mut child = self.take_array_entry(id, path[0]);
        self.array_set_path(&mut child, &path[1..], value);
        self.put_array_entry(id, path[0], child);
    }

    fn array_unset(&mut self, slot: &mut Value, key: &str) {
        let id = self.detach_array_for_write(slot);
        let removed = {
            let PayloadKind::Array(entries) = &mut self.payloads[id.0].kind else {
                panic!("array slot should reference an array payload");
            };
            entries
                .iter()
                .position(|entry| entry.key == key)
                .map(|index| entries.remove(index).value)
        };
        if let Some(value) = removed {
            self.release_owned(value);
        }
    }

    fn array_unset_path(&mut self, slot: &mut Value, path: &[&str]) {
        assert!(!path.is_empty());
        if path.len() == 1 {
            self.array_unset(slot, path[0]);
            return;
        }

        let id = self.detach_array_for_write(slot);
        let mut child = self.take_array_entry(id, path[0]);
        self.array_unset_path(&mut child, &path[1..]);
        self.put_array_entry(id, path[0], child);
    }

    fn append_string_path(&mut self, slot: &mut Value, path: &[&str], suffix: &str) {
        if path.is_empty() {
            self.append_string(slot, suffix);
            return;
        }

        let id = self.detach_array_for_write(slot);
        let mut child = self.take_array_entry(id, path[0]);
        self.append_string_path(&mut child, &path[1..], suffix);
        self.put_array_entry(id, path[0], child);
    }

    fn detach_string_for_write(&mut self, slot: &mut Value) -> PayloadId {
        let Value::String(id) = *slot else {
            panic!("slot should hold a string");
        };
        if self.is_unique_mutable(id) {
            return id;
        }
        self.counters.string_detaches += 1;
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
        self.counters.array_detaches += 1;
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

    fn take_array_entry(&mut self, id: PayloadId, key: &str) -> Value {
        let PayloadKind::Array(entries) = &mut self.payloads[id.0].kind else {
            panic!("payload should be an array");
        };
        let entry = entries
            .iter_mut()
            .find(|entry| entry.key == key)
            .unwrap_or_else(|| panic!("array should contain key {key}"));
        mem::replace(&mut entry.value, Value::Null)
    }

    fn put_array_entry(&mut self, id: PayloadId, key: &str, value: Value) {
        let PayloadKind::Array(entries) = &mut self.payloads[id.0].kind else {
            panic!("payload should be an array");
        };
        let entry = entries
            .iter_mut()
            .find(|entry| entry.key == key)
            .unwrap_or_else(|| panic!("array should contain key {key}"));
        assert_eq!(entry.value, Value::Null);
        entry.value = value;
    }

    fn is_freed(&self, id: PayloadId) -> bool {
        self.payloads[id.0].freed
    }

    fn counters(&self) -> Counters {
        self.counters
    }

    fn total_payloads(&self) -> usize {
        self.payloads.len()
    }

    fn live_payloads(&self) -> usize {
        self.payloads
            .iter()
            .filter(|payload| !payload.freed)
            .count()
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
            self.counters.payload_frees += 1;
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

#[test]
fn nested_detach_unset_return_and_temporary_cleanup_balance_counters() {
    let mut store = ContractStore::default();
    let mut leaf = store.new_string("leaf");
    let leaf_id = store.string_id(leaf);
    let leaf_for_branch = store.retain(leaf);
    let drop_leaf = store.new_string("drop");
    let branch = store.new_array(vec![("text", leaf_for_branch), ("drop", drop_leaf)]);
    let mut root = store.new_array(vec![("branch", branch)]);

    let mut alias = store.retain(root);
    let mut returned = store.retain(root);
    let mut temporary = store.retain(root);
    assert_eq!(store.live_payloads(), 4, "initial live payloads");
    assert_eq!(store.counters(), Counters::default(), "initial counters");

    store.append_string_path(&mut alias, &["branch", "text"], "-copy");
    assert_eq!(
        store.counters().array_detaches,
        2,
        "alias path array detaches"
    );
    assert_eq!(
        store.counters().string_detaches,
        1,
        "alias leaf string detach"
    );
    assert_eq!(
        store.counters().payload_frees,
        0,
        "no payload freed during detach"
    );
    assert_eq!(
        store.live_payloads(),
        7,
        "live payloads after nested string detach"
    );
    assert_eq!(
        store.refcount(leaf_id),
        Some(2),
        "original leaf remains shared"
    );

    store.array_unset_path(&mut alias, &["branch", "drop"]);
    assert_eq!(
        store.counters().array_detaches,
        2,
        "unique alias unset avoids array detach"
    );
    assert_eq!(
        store.counters().string_detaches,
        1,
        "unique alias unset avoids string detach"
    );
    assert_eq!(
        store.counters().payload_frees,
        0,
        "shared unset does not free last drop"
    );
    assert_eq!(
        store.live_payloads(),
        7,
        "unset only releases one shared child"
    );

    store.array_set_path(&mut returned, &["branch", "text"], Value::Int(7));
    assert_eq!(
        store.counters().array_detaches,
        4,
        "return path array detaches"
    );
    assert_eq!(
        store.counters().string_detaches,
        1,
        "integer overwrite avoids string detach"
    );
    assert_eq!(
        store.counters().payload_frees,
        0,
        "shared overwrite keeps children alive"
    );
    assert_eq!(
        store.live_payloads(),
        9,
        "return path clones outer and branch arrays"
    );
    assert_eq!(
        store.refcount(leaf_id),
        Some(2),
        "overwrite releases one retained leaf"
    );

    store.release_slot(&mut temporary);
    assert_eq!(
        store.counters().payload_frees,
        0,
        "temporary drop only decrements root"
    );
    assert_eq!(
        store.live_payloads(),
        9,
        "temporary drop does not free shared graph"
    );

    store.release_slot(&mut returned);
    assert_eq!(
        store.counters().payload_frees,
        2,
        "returned graph frees two arrays"
    );
    assert_eq!(
        store.live_payloads(),
        7,
        "returned graph children remain shared"
    );

    store.release_slot(&mut alias);
    assert_eq!(
        store.counters().payload_frees,
        5,
        "alias graph frees arrays and detached string"
    );
    assert_eq!(
        store.live_payloads(),
        4,
        "only original graph and external leaf remain"
    );

    store.release_slot(&mut root);
    assert_eq!(
        store.counters().payload_frees,
        8,
        "root graph frees root branch and drop leaf"
    );
    assert_eq!(
        store.refcount(leaf_id),
        Some(1),
        "external leaf retains final counted owner"
    );
    store.release_slot(&mut leaf);
    assert_eq!(
        store.counters(),
        Counters {
            array_detaches: 4,
            string_detaches: 1,
            payload_frees: 9,
        },
        "documented final detach/free counts"
    );
    assert_eq!(
        store.total_payloads(),
        9,
        "documented total allocated payloads"
    );
    assert_eq!(
        store.live_payloads(),
        0,
        "no leaked payloads after full cleanup"
    );
}

#[test]
fn repeated_nested_value_cycles_leave_no_live_payloads() {
    let mut store = ContractStore::default();

    for cycle in 0..12 {
        let before = store.counters();
        let before_total = store.total_payloads();
        run_nested_value_drop_cycle(&mut store, cycle);

        assert_eq!(
            store.counters().array_detaches - before.array_detaches,
            4,
            "cycle {cycle} array detach count"
        );
        assert_eq!(
            store.counters().string_detaches - before.string_detaches,
            1,
            "cycle {cycle} string detach count"
        );
        assert_eq!(
            store.counters().payload_frees - before.payload_frees,
            9,
            "cycle {cycle} payload free count"
        );
        assert_eq!(
            store.total_payloads() - before_total,
            9,
            "cycle {cycle} allocated payload count"
        );
        assert_eq!(store.live_payloads(), 0, "cycle {cycle} live payload count");
    }

    assert_eq!(
        store.counters(),
        Counters {
            array_detaches: 48,
            string_detaches: 12,
            payload_frees: 108,
        },
        "documented aggregate counts across 12 nested COW cycles"
    );
    assert_eq!(
        store.total_payloads(),
        108,
        "documented aggregate payload count"
    );
    assert_eq!(
        store.live_payloads(),
        0,
        "no leaks after repeated nested COW cycles"
    );
}

fn run_nested_value_drop_cycle(store: &mut ContractStore, cycle: usize) {
    let leaf_text = format!("leaf-{cycle}");
    let drop_text = format!("drop-{cycle}");
    let mut leaf = store.new_string(&leaf_text);
    let leaf_for_branch = store.retain(leaf);
    let drop_leaf = store.new_string(&drop_text);
    let branch = store.new_array(vec![("text", leaf_for_branch), ("drop", drop_leaf)]);
    let mut root = store.new_array(vec![("branch", branch)]);

    let mut alias = store.retain(root);
    let mut returned = store.retain(root);
    let mut temporary = store.retain(root);

    store.append_string_path(&mut alias, &["branch", "text"], "-alias");
    store.array_unset_path(&mut alias, &["branch", "drop"]);
    store.array_set_path(&mut returned, &["branch", "text"], Value::Int(cycle as i64));

    store.release_slot(&mut temporary);
    store.release_slot(&mut returned);
    store.release_slot(&mut alias);
    store.release_slot(&mut root);
    store.release_slot(&mut leaf);
}
