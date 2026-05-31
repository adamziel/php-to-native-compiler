use std::fs;

use php_compiler::run_source;

#[test]
fn str_replace_and_str_ireplace_cover_array_and_binary_subjects() {
    let execution = run_source(
        r#"<?php
var_dump(str_ireplace("t", "bz", "Text"));
$search = "qXxx\0xXxXxXxx";
$subject = "qxXx\0xxxxxxxx";
var_dump(str_ireplace($search, "any text", $subject));
$a = [0, 1, 2];
$b = ["Nula", "Jedna", "Dva"];
echo str_replace($a, $b, "1"), "|", implode(",", $a), "\n";
var_dump(str_replace([""], [1000], "foo"));
var_dump(str_replace("2", "3", [array("one" => array("a" => "2222"))]));
"#,
    )
    .unwrap();

    assert!(
        execution.stdout.contains("string(6) \"bzexbz\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("string(8) \"any text\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("Jedna|0,1,2\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("string(3) \"foo\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: Array to string conversion"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("string(5) \"Array\""),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn path_highlight_and_strip_whitespace_residuals_are_available() {
    let root = std::env::temp_dir().join(format!(
        "phpc-batch023-standard-string-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    let data = root.join("strip.php");
    fs::write(&data, "<?php\n/* comment */\necho   098   ;\n").unwrap();

    let source = format!(
        r#"<?php
class temp {{
    function __toString() {{
        return "Object";
    }}
}}
echo bin2hex(basename("\xff")), "|", dirname(new temp), "|";
var_dump(set_time_limit(1));
var_dump(is_string(highlight_string("abc", true)));
var_dump(ob_get_contents());
echo php_strip_whitespace("{}");
"#,
        data.display()
    );
    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        "ff|.|bool(true)\nbool(true)\nbool(false)\n<?php\n echo 098 ;"
    );
    assert_eq!(execution.exit_code, 0);
}
