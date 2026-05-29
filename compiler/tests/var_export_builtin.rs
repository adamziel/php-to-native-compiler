use php_compiler::run_source;

#[test]
fn var_export_formats_scalars_arrays_and_objects() {
    let execution = run_source(
        r#"<?php
class Box {
    public $name;
    protected $hidden;
    function __construct() {
        $this->name = "Ada";
        $this->hidden = array("n" => 1);
    }
}

$std = new stdClass();
$std->a = 1;
$std->b = array("k" => "v");
var_export(array("nul" => "\0", "float" => 1.0, "tiny" => 1e-5));
echo "\n--\n";
echo var_export($std, true);
echo "\n--\n";
var_export(new Box());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "array (\n  'nul' => '' . \"\\0\" . '',\n  'float' => 1.0,\n  'tiny' => 1.0000000000000001E-5,\n)\n--\n(object) array(\n   'a' => 1,\n   'b' => \n  array (\n    'k' => 'v',\n  ),\n)\n--\n\\Box::__set_state(array(\n   'name' => 'Ada',\n   'hidden' => \n  array (\n    'n' => 1,\n  ),\n))"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
