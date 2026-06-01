use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use php_compiler::{emit_ir_source, run_source};

fn temp_meta_path(label: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is after epoch")
        .as_nanos();
    std::env::temp_dir()
        .join(format!("phpc-get-meta-tags-{label}-{nanos}.html"))
        .display()
        .to_string()
}

#[test]
fn get_meta_tags_matches_core_phpt_rows() {
    let path = temp_meta_path("phpt");
    let source = format!(
        r#"<?php
$filename = "{}";
$array = array(
    "<meta name=\"author\" content=\"name\">\n<meta name=\"keywords\" content=\"php documentation\">\n<meta name=\"DESCRIPTION\" content=\"a php manual\">\n<meta name=\"geo.position\" content=\"49.33;-86.59\">\n</head> <!-- parsing stops here -->",
    "<html>\n    <head>\n        <meta name=\"author\" content=\"name\">\n        <meta name=\"keywords\" content=\"php documentation\">\n        <meta name=\"DESCRIPTION\" content=\"a php manual\">\n        <meta name=\"geo.position\" content=\"49.33;-86.59\">\n    </head>\n    <body>\n        <meta name=\"author\" content=\"name1\">\n        <meta name=\"keywords\" content=\"php documentation1\">\n    </body>\n</html>",
    "<meta name=\"author\" content=\"name\"\n<meta name=\"keywords\" content=\"php documentation\">",
    "<meta <meta name=\"keywords\" content=\"php documentation\">",
    "<meta name=\"author\" content=\"name\"\n<meta name=\"keywords\" content=\"php documentation\"",
    "",
    "<>",
    "<meta<<<<<"
);
foreach ($array as $html) {{
    file_put_contents($filename, $html);
    var_dump(get_meta_tags($filename));
}}
unlink($filename);
echo "Done\n";
"#,
        path
    );

    let execution = run_source(&source).unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(
        execution.stdout,
        "array(4) {\n  [\"author\"]=>\n  string(4) \"name\"\n  [\"keywords\"]=>\n  string(17) \"php documentation\"\n  [\"description\"]=>\n  string(12) \"a php manual\"\n  [\"geo_position\"]=>\n  string(12) \"49.33;-86.59\"\n}\narray(4) {\n  [\"author\"]=>\n  string(4) \"name\"\n  [\"keywords\"]=>\n  string(17) \"php documentation\"\n  [\"description\"]=>\n  string(12) \"a php manual\"\n  [\"geo_position\"]=>\n  string(12) \"49.33;-86.59\"\n}\narray(1) {\n  [\"keywords\"]=>\n  string(17) \"php documentation\"\n}\narray(1) {\n  [\"keywords\"]=>\n  string(17) \"php documentation\"\n}\narray(0) {\n}\narray(0) {\n}\narray(0) {\n}\narray(0) {\n}\nDone\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_meta_tags_supports_dynamic_calls_metadata_and_attribute_edges() {
    let path = temp_meta_path("metadata");
    let source = format!(
        r#"<?php
$file = "{}";
file_put_contents($file, "<META CONTENT='first value' NAME='Mixed.Name'><meta data-x=1 name=second content=two></HEAD><meta name=late content=ignored>");
$call = "get_meta_tags";
var_dump($call($file));
echo function_exists($call) ? "fn" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not";
echo "|";
$reflection = new ReflectionFunction("get_meta_tags");
echo $reflection->getName(), ":", $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters();
unlink($file);
"#,
        path
    );

    let execution = run_source(&source).unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(
        execution.stdout,
        "array(2) {\n  [\"mixed_name\"]=>\n  string(11) \"first value\"\n  [\"second\"]=>\n  string(3) \"two\"\n}\nfn|callable|get_meta_tags:1/2"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_get_meta_tags_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("get_meta_tags") ? "1" : "0";
echo is_callable("get_meta_tags") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("get_meta_tags"), "{ir}");
}
