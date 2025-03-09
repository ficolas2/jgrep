use assert_cmd::Command;
use common::json_constants;
use indoc::indoc;

mod common {
    pub mod json_constants;
}

#[test]
fn value_and_key_order_preservation() {
    let out = indoc!(r#"
    {
      "parents": {
        "information": [
          {
            "type": "name"
          },
          {
            "type": "name"
          }
        ]
      },
      "name": "Charlie",
      "children": {
        "information": [
          {
            "type": "name"
          }
        ]
      }
    }
    "#);

    let mut cmd = Command::cargo_bin("jrep").unwrap();
    cmd.arg("name");
    cmd.arg("-j");
    cmd.write_stdin(json_constants::OR_MATCH);

    cmd.assert().code(0).stdout(out);
}
