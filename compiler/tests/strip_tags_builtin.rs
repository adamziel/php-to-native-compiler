use php_compiler::{emit_ir_source, run_source};

#[test]
fn strip_tags_removes_html_php_comments_and_nul_bytes() {
    let execution = run_source(
        r#"<?php
echo strip_tags("<html><b>hello</b><p>world</p></html>"), "\n";
echo strip_tags("NEAT <? cool > blah ?> STUFF"), "\n";
echo strip_tags("NEAT <!-- cool > blah --> STUFF"), "\n";
echo strip_tags("<html> I am html string </html>" . chr(0) . "<?php I am php string ?>");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "helloworld\nNEAT  STUFF\nNEAT  STUFF\n I am html string "
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strip_tags_preserves_allowed_html_tags_from_string_and_array_forms() {
    let execution = run_source(
        r##"<?php
$value = '<html><p>hello</p><b>world</b><a href="#fragment">Other text</a></html><?php echo hello ?>';
echo strip_tags($value, "<html><p><a><?php"), "\n";
$nested = "<<htmL>>hello<</htmL>>";
echo strip_tags($nested, "<<html>>"), "\n";
$array = '<p>foo <b>bar</b> <a href="#">foobar</a></p>';
echo strip_tags($array, ['p', 'a']), "\n";
echo strip_tags($array, ['p' => 'a']);
"##,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "<html><p>hello</p>world<a href=\"#fragment\">Other text</a></html>\n<htmL>hello</htmL>\n<p>foo bar <a href=\"#\">foobar</a></p>\nfoo bar <a href=\"#\">foobar</a>"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strip_tags_ignores_angle_brackets_inside_quoted_attributes() {
    let execution = run_source(
        r#"<?php
echo strip_tags('hello <img title="<"> world'), "\n";
echo strip_tags('hello <img title=">_<"> world'), "\n";
echo strip_tags("hello <img title='>_<'> world");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "hello  world\nhello  world\nhello  world");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strip_tags_metadata_is_available_to_capability_checks() {
    let execution = run_source(
        r#"<?php
$call = "strip_tags";
echo function_exists($call) ? "fn" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not";
echo "|";
$function = new ReflectionFunction("Strip_Tags");
echo $function->getName(), ":", $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|strip_tags:1/2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strip_tags_folds_function_metadata_for_ir_capability_checks() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("strip_tags") ? "1" : "0";
echo is_callable("strip_tags") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("strip_tags"), "{ir}");
}
