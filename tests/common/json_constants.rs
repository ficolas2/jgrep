use indoc::indoc;

// Contains a match for "name" in both, a key and a value
#[allow(unused)]
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

// The JSON used in the README.md examples
#[allow(unused)]
pub const README_EXAMPLE: &str = indoc!(r#"
    {
      "items": [
        {
          "id": 1,
          "name": "Lorem",
          "active": true,
          "meta": {
            "rating": 4.7,
            "author": { "name": "John", "verified": false }
          }
        },
        {
          "id": 2,
          "name": "Ipsum",
          "active": false,
          "meta": {
            "rating": 3.9,
            "author": { "name": "Jane", "verified": true }
          }
        }
      ]
    }
"#);
