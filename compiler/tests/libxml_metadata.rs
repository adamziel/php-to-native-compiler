use php_compiler::run_source;

#[test]
fn libxml_internal_errors_empty_stack_matches_metadata_phpt_row() {
    let execution = run_source(
        r#"<?php

var_dump(libxml_use_internal_errors(false));
var_dump(libxml_use_internal_errors(true));
var_dump(libxml_use_internal_errors());

var_dump(libxml_get_errors());
var_dump(libxml_get_last_error());

var_dump(libxml_clear_errors());

echo "Done\n";
?>
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(false)\n",
            "bool(false)\n",
            "bool(true)\n",
            "array(0) {\n",
            "}\n",
            "bool(false)\n",
            "NULL\n",
            "Done\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn libxml_extension_version_and_reflection_metadata_are_bounded() {
    let execution = run_source(
        r#"<?php

echo extension_loaded('libxml') ? 'loaded' : 'missing', "\n";
echo function_exists('libxml_use_internal_errors') ? 'fn' : 'no-fn', "\n";
echo LIBXML_VERSION, '|', LIBXML_DOTTED_VERSION, '|', LIBXML_LOADED_VERSION, "\n";
echo LIBXML_ERR_NONE, LIBXML_ERR_WARNING, LIBXML_ERR_ERROR, LIBXML_ERR_FATAL, "\n";
$ext = new ReflectionExtension('libxml');
echo $ext->getName(), '|', $ext->getVersion() !== false ? 'version' : 'no-version', "\n";
$classes = $ext->getClassNames();
echo $classes[0], '|', class_exists('LibXMLError') ? 'class' : 'no-class', "\n";
$functions = $ext->getFunctions();
foreach (['libxml_use_internal_errors', 'libxml_get_errors', 'libxml_get_last_error', 'libxml_clear_errors'] as $name) {
    echo array_key_exists($name, $functions) ? '1' : '0';
}
echo "\n";
$constants = $ext->getConstants();
echo $constants['LIBXML_VERSION'] === LIBXML_VERSION ? '1' : '0';
echo $constants['LIBXML_DOTTED_VERSION'] === LIBXML_DOTTED_VERSION ? '1' : '0';
echo $constants['LIBXML_ERR_FATAL'] === LIBXML_ERR_FATAL ? '1' : '0';
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "loaded\n",
            "fn\n",
            "21308|2.13.8|21308\n",
            "0123\n",
            "libxml|version\n",
            "LibXMLError|class\n",
            "1111\n",
            "111\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
