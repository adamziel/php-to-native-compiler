use php_compiler::{emit_ir_source, run_source};

#[test]
fn http_build_query_encodes_scalar_arrays_with_prefix_and_separator() {
    let execution = run_source(
        r#"<?php
$array = array("foo"=>"bar","baz"=>1,"test"=>"a ' \" ", "abc", "float" => 10.42, "true" => true, "false" => false);
var_dump(http_build_query($array));
var_dump(http_build_query($array, "foo"));
var_dump(http_build_query($array, "foo", ";"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(62) \"foo=bar&baz=1&test=a+%27+%22+&0=abc&float=10.42&true=1&false=0\"\nstring(65) \"foo=bar&baz=1&test=a+%27+%22+&foo0=abc&float=10.42&true=1&false=0\"\nstring(65) \"foo=bar;baz=1;test=a+%27+%22+;foo0=abc;float=10.42;true=1;false=0\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn http_build_query_uses_public_object_properties_only() {
    let execution = run_source(
        r#"<?php
class UrlBuilder
{
  public $name = "homepage";
  public $page = 1;
  protected $sort = "desc,name";
  private $access = "admin";
}

$obj = new stdClass;
$obj->name = "homepage";
$obj->page = 1;
$obj->sort = "desc,name";

echo http_build_query($obj), "\n";
echo http_build_query(new UrlBuilder()), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "name=homepage&page=1&sort=desc%2Cname\nname=homepage&page=1\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn http_build_query_recurses_and_skips_null_or_resource_values() {
    let execution = run_source(
        r#"<?php
$mDimensional = array(
  20,
  5 => 13,
  "9" => array(
    1 => "val1",
    3 => "val2",
    "string" => "string"
  ),
  "name" => "homepage",
  "page" => 10,
  "sort" => array(
    "desc",
    "admin" => array(
      "admin1",
      "admin2" => array(
        "who" => "admin2",
        2 => "test"
      )
    )
  )
);

echo http_build_query($mDimensional), "\n";
echo http_build_query($mDimensional, "prefix_"), "\n";
var_dump(http_build_query(array(null)));
$v = "value";
$ref = &$v;
var_dump(http_build_query(array($ref)));
var_dump(http_build_query(array(STDIN)));
echo http_build_query(array("space" => "a b", "tilde" => "~"), "", "&", PHP_QUERY_RFC3986), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "0=20&5=13&9%5B1%5D=val1&9%5B3%5D=val2&9%5Bstring%5D=string&name=homepage&page=10&sort%5B0%5D=desc&sort%5Badmin%5D%5B0%5D=admin1&sort%5Badmin%5D%5Badmin2%5D%5Bwho%5D=admin2&sort%5Badmin%5D%5Badmin2%5D%5B2%5D=test\nprefix_0=20&prefix_5=13&prefix_9%5B1%5D=val1&prefix_9%5B3%5D=val2&prefix_9%5Bstring%5D=string&name=homepage&page=10&sort%5B0%5D=desc&sort%5Badmin%5D%5B0%5D=admin1&sort%5Badmin%5D%5Badmin2%5D%5Bwho%5D=admin2&sort%5Badmin%5D%5Badmin2%5D%5B2%5D=test\nstring(0) \"\"\nstring(7) \"0=value\"\nstring(0) \"\"\nspace=a%20b&tilde=~\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn http_build_query_metadata_and_query_encoding_constants_are_visible() {
    let execution = run_source(
        r#"<?php
echo PHP_QUERY_RFC1738, "|", PHP_QUERY_RFC3986, "|";
echo function_exists("http_build_query") ? "fn" : "missing";
echo "|", is_callable("http_build_query") ? "callable" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|2|fn|callable");
    assert_eq!(execution.exit_code, 0);

    let ir = emit_ir_source(
        r#"<?php
echo function_exists("http_build_query") ? "1" : "0";
echo defined("PHP_QUERY_RFC1738") ? "1" : "0";
echo defined("PHP_QUERY_RFC3986") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");
    assert!(!ir.contains("http_build_query"), "{ir}");
    assert!(!ir.contains("PHP_QUERY_RFC1738"), "{ir}");
    assert!(!ir.contains("PHP_QUERY_RFC3986"), "{ir}");
}

#[test]
fn http_build_query_accepts_named_optional_arguments() {
    let execution = run_source(
        r#"<?php
$data = ["hello" => "world", "space" => "a b"];
var_dump(http_build_query($data, encoding_type: PHP_QUERY_RFC3986));
class StringableOnly {
    public function __toString(): string {
        return "Stringable";
    }
}
$object = new StringableOnly();
var_dump(http_build_query(["hello", $object], numeric_prefix: "prefix_"));
var_dump(http_build_query($object, numeric_prefix: "prefix_"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(23) \"hello=world&space=a%20b\"\nstring(14) \"prefix_0=hello\"\nstring(0) \"\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn http_build_query_skips_recursive_object_branches() {
    let execution = run_source(
        r#"<?php
class KeyVal {
    public $public = "input";
}
$one = new KeyVal();
$one->public = $one;
var_dump(http_build_query($one));

class SelfNamed {
    public $name = "ok";
    public $self = null;
}
$two = new SelfNamed();
$two->self = $two;
var_dump(http_build_query($two));

$shared = new KeyVal();
$shared->public = "input";
var_dump(http_build_query(["a" => $shared, "b" => $shared]));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(0) \"\"\nstring(7) \"name=ok\"\nstring(39) \"a%5Bpublic%5D=input&b%5Bpublic%5D=input\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}
