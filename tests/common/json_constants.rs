use indoc::indoc;

// Contains a match for "name" in both, a key and a value
pub const OR_MATCH: &str  = indoc!(r#"
    {
      "parents": {
        "information": [
          { "type": "name", "value": "Alice" },
          { "type": "name", "value": "Bob" }
        ]
      },
      "name": "Charlie",
      "children": {
        "information": [
          {
            "type": "name",
            "value": "David"
          }
        ]
      }
    }
"#);
