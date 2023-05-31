#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "op")]
enum Operation {
    Delete { count: usize },
    Insert { chars: String },
    Skip { count: usize },
}

pub fn is_valid(stale: &str, latest: &str, otjson: &str) -> bool {
    let operations = parse_operations(otjson);

    let mut draft = stale.to_string();
    let mut position: usize = 0;

    for operation in operations.iter() {
        match operation {
            Operation::Delete { count } => {
                if position + *count > draft.len() {
                    return false;
                }

                draft = draft
                    .chars()
                    .take(position)
                    .chain(draft.chars().skip(position + *count))
                    .collect();
            }
            Operation::Insert { chars } => {
                draft = draft
                    .chars()
                    .take(position)
                    .chain(chars.chars())
                    .chain(draft.chars().skip(position))
                    .collect();

                position += chars.len();
            }
            Operation::Skip { count } => {
                if position + count >= draft.len() {
                    return false;
                }

                position += count;
            }
        }
    }

    draft == latest
}

fn parse_operations(serialized: &str) -> Vec<Operation> {
    serde_json::from_str(&serialized).expect("Valid JSON")
}

#[test]
fn works_with_deletions() {
    let result = is_valid(
        "Repl.it uses operational transformations to keep everyone in a multiplayer repl in sync.",
        "Repl.it uses operational transformations.",
        "[{\"op\": \"skip\", \"count\": 40}, {\"op\": \"delete\", \"count\": 47}]",
    ); // true

    assert!(result == true);
}

#[test]
fn fails_to_delete_excess() {
    let result = is_valid(
        "Repl.it uses operational transformations to keep everyone in a multiplayer repl in sync.",
        "Repl.it uses operational transformations.",
        "[{\"op\": \"skip\", \"count\": 45}, {\"op\": \"delete\", \"count\": 47}]",
    ); // false, delete past end

    assert!(result == false);
}

#[test]
fn fails_to_skip_excess() {
    let result = is_valid(
      "Repl.it uses operational transformations to keep everyone in a multiplayer repl in sync.",
      "Repl.it uses operational transformations.",
      "[{\"op\": \"skip\", \"count\": 40}, {\"op\": \"delete\", \"count\": 47}, {\"op\": \"skip\", \"count\": 2}]"
    ); // false, skip past end

    assert!(result == false);
}

#[test]
fn succeeds_with_multiple_operations() {
    let result = is_valid(
      "Repl.it uses operational transformations to keep everyone in a multiplayer repl in sync.",
      "We use operational transformations to keep everyone in a multiplayer repl in sync.",
      "[{\"op\": \"delete\", \"count\": 7}, {\"op\": \"insert\", \"chars\": \"We\"}, {\"op\": \"skip\", \"count\": 4}, {\"op\": \"delete\", \"count\": 1}]"
    ); // true

    assert!(result == true);
}

#[test]
fn fails_if_desynced() {
    let result = is_valid(
      "Repl.it uses operational transformations to keep everyone in a multiplayer repl in sync.",
      "We can use operational transformations to keep everyone in a multiplayer repl in sync.",
      "[{\"op\": \"delete\", \"count\": 7}, {\"op\": \"insert\", \"chars\": \"We\"}, {\"op\": \"skip\", \"count\": 4}, {\"op\": \"delete\", \"count\": 1}]"
    ); // false

    assert!(result == false);
}

#[test]
fn works_with_no_operations() {
    let result = is_valid(
        "Repl.it uses operational transformations to keep everyone in a multiplayer repl in sync.",
        "Repl.it uses operational transformations to keep everyone in a multiplayer repl in sync.",
        "[]",
    ); // true

    assert!(result == true);
}
