use assert_cmd::Command;
use common::json_constants;
use indoc::indoc;

mod common {
    pub mod json_constants;
}

#[test]
fn query_lang_simple() {
    let out = indoc!(r#"
        .items[1].meta.author.name: "Jane"
    "#);

    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg("Jane");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);
}

#[test]
fn query_lang_keys() {
    let out = indoc!(r#"
        .items[0].meta.author: {"name":"John","verified":false}
        .items[1].meta.author: {"name":"Jane","verified":true}
    "#);

    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg("author:");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);

    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg(".meta.author");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);
}

#[test]
fn query_lang_values() {
    let out = indoc!(r#"
        .items[0].active: true
        .items[1].meta.author.verified: true
    "#);

    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg(": true");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);
}


#[test]
fn query_lang_values_and_keys() {
    let out = indoc!(r#"
        .items[0].meta.rating: 4.7
    "#);

    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg(".rating: 4.7");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);
}

#[test]
fn query_lang_wildcard() {
    let out = indoc!(r#"
        .items[0].meta.author.name: "John"
        .items[1].meta.author.name: "Jane"
    "#);

    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg(".name: J*n*");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);

    let out = indoc!(r#"
        .items[1].meta.author.name: "Jane"
    "#);


    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg(".name: Jan?");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);
}

#[test]
fn query_lang_wildcard_number() {
    let out = indoc!(r#"
        .items[0].meta.rating: 4.7
    "#);

    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg(".rating: 4.*?");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);
}

#[test]
fn flags_json() {
    let out = indoc!(r#"
    {
      "items": [
        {
          "meta": {
            "rating": 4.7
          }
        },
        {
          "meta": {
            "rating": 3.9
          }
        }
      ]
    }
    "#);

    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg(".rating");
    cmd.arg("-j");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);
}

#[test]
fn flags_only() {
    let out = indoc!(r#"
        {"name":"John","verified":false}
        {"name":"Jane","verified":true}
    "#);

    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg(".author");
    cmd.arg("-o");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);
}

#[test]
fn context() {
    let out = ".items[1].meta.author: {\"name\":\"Jane\",\"verified\":true}\n";

    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg("Jane");
    cmd.arg("-C");
    cmd.arg("1");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);
}

#[test]
fn context_json() {
    let out = indoc!(r#"
        {
          "items": [
            {
              "meta": {
                "rating": 3.9,
                "author": {
                  "name": "Jane",
                  "verified": true
                }
              }
            },
          ]
        }
    "#);

    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg("Jane");
    cmd.arg("-C");
    cmd.arg("2");
    cmd.arg("-j");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);
}

#[test]
fn no_context_json() {
    let out = indoc!(r#"
        {
          "items": [
            {
              "meta": {
                "author": {
                  "name": "Jane"
                }
              }
            },
          ]
        }
    "#);

    let mut cmd = Command::cargo_bin("jgrep").unwrap();
    cmd.arg("Jane");
    cmd.arg("-j");
    cmd.write_stdin(json_constants::README_EXAMPLE);

    cmd.assert().code(0).stdout(out);
}
