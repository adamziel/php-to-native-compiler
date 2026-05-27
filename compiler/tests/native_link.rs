use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use php_compiler::{codegen::emit_native_executable_c_source, error::Phase, parse};

const ASSEMBLY_CLOSURE_REJECTION: &str = "assembly closure lowering rejects closure shapes outside the bounded generated-C descriptor-backed closure frame subset, including by-reference closure captures that cannot be materialized through root symbol/reference handles or promoted frame locals, by-reference variadic closure parameters, by-reference closure returns, unsupported closure bodies, references/copy-on-write, and exact native callable errors; generated-native C lowers supported descriptor closures, supported static arrow closures, by-value captures, supported by-reference captures, implicit by-value arrow captures, non-static $this closure binding, typed/default/variadic by-value closure parameters, and untyped by-reference closure parameters through dynamic callable dispatch";

const NATIVE_VALUE_TRUTHINESS_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"empty\" => \"\", \"zero\" => \"0\", \"one\" => \"1\"];\n",
    "echo (!($items[\"empty\"]) ? \"T\" : \"F\");\n",
    "echo (!($items[\"zero\"]) ? \"T\" : \"F\");\n",
    "echo (!($items[\"one\"]) ? \"T\" : \"F\");\n",
    "echo (($items[\"zero\"] xor $items[\"one\"]) ? \"T\" : \"F\");\n",
    "echo (($items[\"empty\"] xor array_sum([0])) ? \"T\" : \"F\");\n",
    "echo ((array_sum([2]) xor $items[\"one\"]) ? \"T\" : \"F\");\n",
);

const NATIVE_SHORT_CIRCUIT_LOGICAL_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"empty\" => \"\", \"zero\" => \"0\", \"one\" => \"1\", \"word\" => \"go\"];\n",
    "echo ($items[\"zero\"] && exit(\"bad\")) ? \"T\" : \"F\";\n",
    "echo \"|\";\n",
    "echo ($items[\"one\"] || exit(\"bad\")) ? \"T\" : \"F\";\n",
    "echo \"|\";\n",
    "echo ($items[\"one\"] && strtoupper($items[\"word\"])) ? \"T\" : \"F\";\n",
    "echo \"|\";\n",
    "echo ($items[\"empty\"] || strrev(\"ko\")) ? \"T\" : \"F\";\n",
);

const NATIVE_SCOPED_IF_SOURCE: &str = concat!(
    "<?php\n",
    "$sum = 2 + \"1\";\n",
    "if ($sum) {\n",
    "    echo \"truthy\";\n",
    "} else {\n",
    "    echo \"false\";\n",
    "}\n",
    "echo \"|\";\n",
    "$left = \"10\";\n",
    "$right = 2;\n",
    "if ($left > $right) {\n",
    "    echo \"numeric\";\n",
    "} else {\n",
    "    echo \"lexical\";\n",
    "}\n",
    "echo \"|\";\n",
    "if ($sum == 3) {\n",
    "    print \"equal\";\n",
    "} else {\n",
    "    print \"different\";\n",
    "}\n",
);

const NATIVE_LEADING_NUMERIC_ARITHMETIC_SOURCE: &str = concat!(
    "<?php\n",
    "echo \"8tail\" + 2, \"|\";\n",
    "echo 10 - \"3tail\", \"|\";\n",
    "echo \"2.5tail\" * 4, \"|\";\n",
    "echo \"9tail\" / 3, \"|\";\n",
    "echo \"9tail\" % 4, \"|\";\n",
    "echo -(\"6tail\");\n",
);

const NATIVE_OUTPUT_BUFFER_SOURCE: &str = concat!(
    "<?php\n",
    "ob_start();\n",
    "echo \"A\\0B\";\n",
    "echo 42;\n",
    "$contents = ob_get_contents();\n",
    "$length = ob_get_length();\n",
    "ob_clean();\n",
    "echo strtolower(\"HIDDEN\");\n",
    "$hidden = ob_get_clean();\n",
    "echo $contents;\n",
    "echo \":\";\n",
    "echo $length;\n",
    "echo \":\";\n",
    "echo $hidden;\n",
    "echo \"|\";\n",
    "ob_start(null, strlen(\"aa\"), strlen(\"flags\"));\n",
    "ob_list_handlers();\n",
    "ob_get_status(true);\n",
    "echo \"A\";\n",
    "ob_start();\n",
    "echo \"B\";\n",
    "ob_end_flush();\n",
    "echo ob_get_clean();\n",
    "echo \"|\";\n",
    "echo ob_get_level();\n",
    "echo \"\\n\";\n",
);

const NATIVE_DIAGNOSTIC_RESULT_DISCARDED_EXPR_SOURCE: &str = concat!(
    "<?php\n",
    "1;\n",
    "\"two\";\n",
    "$flag = true;\n",
    "$flag;\n",
    "echo \"ok\";\n",
);

const NATIVE_DIAGNOSTIC_RESULT_OUTPUT_OPERANDS_SOURCE: &str = concat!(
    "<?php\n",
    "$value = \"V\";\n",
    "$ref =& $value;\n",
    "echo \"A\", 2, $value, $ref, \"|\";\n",
    "echo [1];\n",
    "print \"|done\\n\";\n",
);

const NATIVE_RETURN_TERMINAL_KIND_HANDOFF_SOURCE: &str = concat!(
    "<?php\n",
    "function finish($value) {\n",
    "    try { return $value; } finally { echo \"f\"; }\n",
    "}\n",
    "class ReturnTerminalBox {\n",
    "    public function label($value) {\n",
    "        try { return $value; } finally { echo \"m\"; }\n",
    "    }\n",
    "}\n",
    "$box = new ReturnTerminalBox();\n",
    "echo finish(\"GO\"), \"|\", $box->label(\"hi\");\n",
);

const NATIVE_TRY_FINALLY_RETURN_CLEANUP_DIAGNOSTIC_SOURCE: &str = concat!(
    "<?php\n",
    "function cleanup_finish($value) {\n",
    "    try { return $value; } finally { echo \"f\", [1]; }\n",
    "}\n",
    "class CleanupReturnBox {\n",
    "    public function label($value) {\n",
    "        try { return $value; } finally { echo \"m\", [2]; }\n",
    "    }\n",
    "}\n",
    "$box = new CleanupReturnBox();\n",
    "echo cleanup_finish(\"GO\"), \"|\", $box->label(\"hi\");\n",
);

const NATIVE_DECLARED_CLASS_OBJECT_SOURCE: &str = concat!(
    "<?php\n",
    "class Box { public $name; private $secret; }\n",
    "class Packet {}\n",
    "$box = new Box();\n",
    "echo class_exists(\"Box\") ? \"Y\" : \"N\";\n",
    "echo class_exists(\"packet\") ? \"Y\" : \"N\";\n",
    "echo class_exists(\"Missing\", false) ? \"Y\" : \"N\";\n",
    "echo \"|\";\n",
    "echo is_object($box) ? \"object\" : \"not\";\n",
    "echo \":\";\n",
    "echo gettype($box);\n",
    "echo \":\";\n",
    "echo get_debug_type($box);\n",
    "echo \"|\";\n",
    "echo is_object(new Packet()) ? get_debug_type(new Packet()) : \"bad\";\n",
    "echo \"\\n\";\n",
);

const NATIVE_DECLARED_CLASS_DYNAMIC_NEW_SOURCE: &str = concat!(
    "<?php\n",
    "class Alpha {}\n",
    "class Beta {}\n",
    "class Base { public $name; }\n",
    "class Child extends Base {}\n",
    "function mark($value) { echo $value; return $value; }\n",
    "$class = \"beta\";\n",
    "echo is_object(new Alpha(mark(\"N\"))) ? \":named:\" : \":bad:\";\n",
    "$object = new $class(mark(\"D1\"), mark(\"D2\"));\n",
    "echo gettype($object);\n",
    "echo \":\";\n",
    "echo get_debug_type($object);\n",
    "echo \":\";\n",
    "$class = \"Child\";\n",
    "$child = new $class(mark(\"D3\"));\n",
    "$child->name = \"kid\";\n",
    "echo ($child instanceof Base) ? \"base\" : \"no\";\n",
    "echo \":\";\n",
    "echo $child->name;\n",
    "echo \":\";\n",
    "$class = \"Alpha\";\n",
    "echo is_object(new $class()) ? get_debug_type(new $class(mark(\"D4\"))) : \"bad\";\n",
    "echo \"\\n\";\n",
);

const NATIVE_DECLARED_CLASS_PROPERTY_SOURCE: &str = concat!(
    "<?php\n",
    "class Box { public $name; public $count; private $secret; }\n",
    "$box = new Box();\n",
    "echo empty($box->name) ? \"E\" : \"N\";\n",
    "echo isset($box->name) ? \"S\" : \"M\";\n",
    "echo \"|\";\n",
    "echo ($box->name = \"Ada\");\n",
    "echo \":\";\n",
    "echo $box->name;\n",
    "echo \":\";\n",
    "echo isset($box->name) ? \"S\" : \"M\";\n",
    "echo \":\";\n",
    "echo empty($box->name) ? \"E\" : \"N\";\n",
    "echo \"|\";\n",
    "$other = new Box();\n",
    "$other->count = 7;\n",
    "echo $other->count;\n",
    "echo \"\\n\";\n",
);

const NATIVE_DECLARED_CLASS_STATIC_PROPERTY_SOURCE: &str = concat!(
    "<?php\n",
    "class Base { public static int $count = 2; }\n",
    "class Counter extends Base { public static $label = \"ready\"; }\n",
    "echo Counter::$count;\n",
    "echo \":\";\n",
    "echo (Counter::$count = 5);\n",
    "echo \":\";\n",
    "echo Base::$count;\n",
    "echo \":\";\n",
    "echo Counter::$label;\n",
    "echo \"\\n\";\n",
);

const NATIVE_SELF_PARENT_STATIC_PROPERTY_SOURCE: &str = concat!(
    "<?php\n",
    "class StaticPropertyRoot {\n",
    "    protected static int $count = 2;\n",
    "    public static $label = \"root\";\n",
    "    public static function rootRead() { return self::$label; }\n",
    "}\n",
    "class StaticPropertyMid extends StaticPropertyRoot {\n",
    "    private static $local = \"mid\";\n",
    "    public static function bump($value) {\n",
    "        self::$local = $value;\n",
    "        parent::$count = parent::$count + 3;\n",
    "        return self::$local . \":\" . parent::$count . \":\" . self::rootRead();\n",
    "    }\n",
    "    public static function read() { return self::$local . \":\" . parent::$count; }\n",
    "}\n",
    "class StaticPropertyLeaf extends StaticPropertyMid {}\n",
    "echo StaticPropertyMid::bump(\"go\"), \"|\", StaticPropertyLeaf::read(), \"\\n\";\n",
);

const NATIVE_LATE_STATIC_PROPERTY_SOURCE: &str = concat!(
    "<?php\n",
    "class LateStaticPropertyRoot {\n",
    "    public static int $count = 1;\n",
    "    public static $label = \"root\";\n",
    "    public static function bump() {\n",
    "        static::$count = static::$count + 4;\n",
    "        return static::$label . \":\" . static::$count;\n",
    "    }\n",
    "    public static function rootCount() { return self::$count; }\n",
    "}\n",
    "class LateStaticPropertyLeaf extends LateStaticPropertyRoot {\n",
    "    public static int $count = 10;\n",
    "    public static $label = \"leaf\";\n",
    "}\n",
    "echo LateStaticPropertyRoot::bump(), \"|\", LateStaticPropertyLeaf::bump(), \"|\";\n",
    "echo LateStaticPropertyRoot::rootCount(), \":\", LateStaticPropertyLeaf::bump(), \"\\n\";\n",
);

const NATIVE_DECLARED_CLASS_PROPERTY_UNSET_SOURCE: &str = concat!(
    "<?php\n",
    "class Box { public $name; public $peer; private $secret; }\n",
    "$box = new Box();\n",
    "$box->name = \"Ada\";\n",
    "$box->peer = new Box();\n",
    "$box->peer->name = \"Bee\";\n",
    "unset($box->name);\n",
    "echo isset($box->name) ? \"1\" : \"0\";\n",
    "echo empty($box->name) ? \"1\" : \"0\";\n",
    "echo \"|\";\n",
    "unset($box->peer->name, $box->missing);\n",
    "echo isset($box->peer->name) ? \"1\" : \"0\";\n",
    "echo empty($box->missing) ? \"1\" : \"0\";\n",
    "echo \"|\";\n",
    "$box->name = \"Z\";\n",
    "echo $box->name;\n",
    "echo \"\\n\";\n",
);

const NATIVE_DECLARED_CLASS_INSTANCEOF_SOURCE: &str = concat!(
    "<?php\n",
    "class Box {}\n",
    "class Packet {}\n",
    "$box = new Box();\n",
    "echo $box instanceof Box ? \"Y\" : \"N\";\n",
    "echo $box instanceof box ? \"Y\" : \"N\";\n",
    "echo $box instanceof Packet ? \"Y\" : \"N\";\n",
    "echo (new Packet()) instanceof Packet ? \"Y\" : \"N\";\n",
    "echo 7 instanceof Box ? \"Y\" : \"N\";\n",
    "echo \"\\n\";\n",
);

const NATIVE_DECLARED_CLASS_METHOD_SOURCE: &str = concat!(
    "<?php\n",
    "class Box {\n",
    "    public $name;\n",
    "    public function store($value = \"Ada\") {\n",
    "        $this->name = $value;\n",
    "        return $this->name;\n",
    "    }\n",
    "    public function label($prefix) {\n",
    "        return strtoupper($prefix);\n",
    "    }\n",
    "}\n",
    "class Packet {\n",
    "    public $code;\n",
    "    public function store($value) {\n",
    "        $this->code = $value;\n",
    "        return $this->code;\n",
    "    }\n",
    "}\n",
    "$box = new Box();\n",
    "echo $box->store(), \":\", $box->name, \"|\";\n",
    "echo $box->store(\"Grace\"), \":\", $box->name, \"|\";\n",
    "$packet = new Packet();\n",
    "echo $packet->store(7), \":\", $packet->code, \"|\";\n",
    "echo $box->label(\"go\"), \"|\";\n",
    "echo (new Box())->store(\"Temp\"), \"|\";\n",
    "echo $box->store(\"Tail\"), \"\\n\";\n",
);

const NATIVE_DECLARED_CLASS_DYNAMIC_METHOD_SOURCE: &str = concat!(
    "<?php\n",
    "class Box {\n",
    "    public $name;\n",
    "    public function store($value = \"Ada\") {\n",
    "        $this->name = $value;\n",
    "        return $this->name;\n",
    "    }\n",
    "    public function label($prefix) {\n",
    "        return strtoupper($prefix);\n",
    "    }\n",
    "}\n",
    "class Packet {\n",
    "    public $code;\n",
    "    public function store($value) {\n",
    "        $this->code = $value;\n",
    "        return $this->code;\n",
    "    }\n",
    "    public function label($prefix = \"P\") {\n",
    "        return strtolower($prefix);\n",
    "    }\n",
    "}\n",
    "$box = new Box();\n",
    "$packet = new Packet();\n",
    "$store = \"store\";\n",
    "$label = \"label\";\n",
    "echo $box->$store(), \":\", $box->name, \"|\";\n",
    "echo $packet->{$store}(7), \":\", $packet->code, \"|\";\n",
    "echo $box->{$label}(\"go\"), \"|\";\n",
    "echo $packet->{($packet instanceof Packet ? \"label\" : \"store\")}(\"LOUD\"), \"|\";\n",
    "$box->{$store}(\"Tail\");\n",
    "echo $box->name, \"\\n\";\n",
);

const NATIVE_DECLARED_CLASS_DYNAMIC_METHOD_LOOKUP_SOURCE: &str = concat!(
    "<?php\n",
    "class Box {\n",
    "    public function store($value = \"ok\") { return $value; }\n",
    "}\n",
    "$box = new Box();\n",
    "$method = 0;\n",
    "echo $box->$method(), \"\\n\";\n",
);

const NATIVE_DECLARED_DYNAMIC_METHOD_SOURCE_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "class DynamicSourceCallBox {\n",
    "    public function left(&$slot, $suffix) { $slot = \"left-\" . $suffix; return \"L:\" . $slot; }\n",
    "    public function right(&$slot, $suffix) { $slot = \"right-\" . $suffix; return \"R:\" . $slot; }\n",
    "}\n",
    "$box = new DynamicSourceCallBox();\n",
    "$slot = \"seed\";\n",
    "echo $box->{(true ? \"left\" : \"right\")}($slot, \"a\"), \"|\", $slot, \"|\";\n",
    "echo $box->{(false ? \"left\" : \"right\")}($slot, \"b\"), \"|\", $slot, \"\\n\";\n",
);

const NATIVE_DECLARED_DYNAMIC_METHOD_SOURCE_CALL_FAILURE_SOURCE: &str = concat!(
    "<?php\n",
    "class DynamicSourceCallFailureBox {\n",
    "    public function ok($value) { return \"ok:\" . $value; }\n",
    "}\n",
    "$box = new DynamicSourceCallFailureBox();\n",
    "echo $box->{(false ? \"ok\" : \"missing\")}(\"value\");\n",
    "echo \"after\\n\";\n",
);

const NATIVE_DECLARED_DYNAMIC_METHOD_CLASS_CONTEXT_SOURCE: &str = concat!(
    "<?php\n",
    "class DynamicSourceCallVisibilityBox {\n",
    "    public function reveal($value) {\n",
    "        $method = \"secret\";\n",
    "        return $this->{$method}($value);\n",
    "    }\n",
    "    private function secret($value) { return \"p\" . strtoupper($value); }\n",
    "}\n",
    "$box = new DynamicSourceCallVisibilityBox();\n",
    "echo $box->reveal(\"go\"), \"\\n\";\n",
);

const NATIVE_DECLARED_RUNTIME_DYNAMIC_METHOD_SOURCE_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "class RuntimeDynamicSourceCallBox {\n",
    "    public function left(&$slot, $suffix) { $slot = \"left-\" . $suffix; return \"L:\" . $slot; }\n",
    "    public function right(&$slot, $suffix) { $slot = \"right-\" . $suffix; return \"R:\" . $slot; }\n",
    "}\n",
    "$box = new RuntimeDynamicSourceCallBox();\n",
    "$slot = \"seed\";\n",
    "$method = strtolower(\"LEFT\");\n",
    "echo $box->{$method}($slot, \"a\"), \"|\", $slot, \"|\";\n",
    "$method = strtolower(\"RIGHT\");\n",
    "echo $box->{$method}($slot, \"b\"), \"|\", $slot, \"\\n\";\n",
);

const NATIVE_DECLARED_DYNAMIC_METHOD_MAGIC_BLOCKED_SOURCE: &str = concat!(
    "<?php\n",
    "class DynamicSourceCallMagicBox {\n",
    "    public function __call($name, $args) { return \"magic:\" . $name . \":\" . $args[0]; }\n",
    "    public function known($value = \"ok\") { return \"known:\" . $value; }\n",
    "    public function reveal() { $method = \"hidden\"; return $this->{$method}(\"in\"); }\n",
    "    private function hidden($value) { return \"hidden:\" . $value; }\n",
    "}\n",
    "$box = new DynamicSourceCallMagicBox();\n",
    "$method = \"known\";\n",
    "echo $box->{$method}(\"A\"), \"|\";\n",
    "$method = strtolower(\"MISSING\");\n",
    "echo $box->{$method}(\"B\"), \"|\";\n",
    "echo $box->reveal(), \"\\n\";\n",
);

const NATIVE_DECLARED_CLASS_STATIC_METHOD_SOURCE: &str = concat!(
    "<?php\n",
    "class Label {\n",
    "    public static function text($value = \"Ada\") { return strtoupper($value); }\n",
    "    public static function note($value) { echo $value; return $value; }\n",
    "}\n",
    "class Counter {\n",
    "    public static function add($left, $right = 1) { return $left + $right; }\n",
    "}\n",
    "echo Label::text(), \"|\";\n",
    "echo Label::text(\"grace\"), \"|\";\n",
    "echo Counter::add(6), \"|\";\n",
    "$stored = Label::text(Label::text(\"go\"));\n",
    "echo $stored, \"|\";\n",
    "Label::note(\"drop\");\n",
    "echo \"\\n\";\n",
);

const NATIVE_DECLARED_OBJECT_STATIC_METHOD_SOURCE: &str = concat!(
    "<?php\n",
    "class Label {\n",
    "    public static function text($value = \"Ada\") { return strtoupper($value); }\n",
    "    public static function note($value) { echo $value; return $value; }\n",
    "}\n",
    "class Lower {\n",
    "    public static function text($value = \"LOUD\") { return strtolower($value); }\n",
    "}\n",
    "class Counter {\n",
    "    public static function add($left, $right = 1) { return $left + $right; }\n",
    "}\n",
    "$label = new Label();\n",
    "$lower = new Lower();\n",
    "$counter = new Counter();\n",
    "echo $label::text(), \"|\";\n",
    "echo $lower::text(\"SHOUT\"), \"|\";\n",
    "echo $counter::add(6), \"|\";\n",
    "$stored = $label::text($lower::text(\"GO\"));\n",
    "echo $stored, \"|\";\n",
    "$target = $counter;\n",
    "echo $target::add(2, 5), \"|\";\n",
    "$label::note(\"drop\");\n",
    "echo \"\\n\";\n",
);

const NATIVE_DECLARED_OBJECT_STATIC_DEFAULT_VARIADIC_SOURCE_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "class ObjectStaticWide {\n",
    "    public static function stat($head, $suffix = \"S\", ...$tail) {\n",
    "        return $head . \":\" . $suffix . \":\" . ($tail[0] ?? \"empty\");\n",
    "    }\n",
    "}\n",
    "$wide = new ObjectStaticWide();\n",
    "echo $wide::stat(\"O\"), \"|\";\n",
    "echo $wide::stat(\"O\", \"x\", \"tail\"), \"\\n\";\n",
);

const NATIVE_DECLARED_METHOD_STATIC_SOURCE_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "class SourceCallAlpha {\n",
    "    public function inst($value) { return \"A\" . strtoupper($value); }\n",
    "    public static function stat($value) { return \"S\" . strtolower($value); }\n",
    "}\n",
    "class SourceCallBeta {\n",
    "    public function inst($value) { return \"B\" . strtolower($value); }\n",
    "    public static function stat($value) { return \"T\" . strtoupper($value); }\n",
    "}\n",
    "$alpha = new SourceCallAlpha();\n",
    "$beta = new SourceCallBeta();\n",
    "echo $alpha->inst(\"go\"), \"|\", $beta->inst(\"LOUD\"), \"|\";\n",
    "echo SourceCallAlpha::stat(\"LOUD\"), \"|\", SourceCallBeta::stat(\"go\"), \"\\n\";\n",
);

const NATIVE_DECLARED_METHOD_STATIC_DEFAULT_VARIADIC_SOURCE_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "class SourceCallWide {\n",
    "    public function inst($head, $suffix = \"D\", ...$tail) {\n",
    "        return $head . \":\" . $suffix . \":\" . ($tail[0] ?? \"empty\");\n",
    "    }\n",
    "    public static function stat($head, $suffix = \"S\", ...$tail) {\n",
    "        return $head . \":\" . $suffix . \":\" . ($tail[0] ?? \"empty\");\n",
    "    }\n",
    "}\n",
    "$wide = new SourceCallWide();\n",
    "echo $wide->inst(\"R\"), \"|\";\n",
    "echo $wide->inst(\"R\", \"x\", \"tail\"), \"|\";\n",
    "echo SourceCallWide::stat(\"S\"), \"|\";\n",
    "echo SourceCallWide::stat(\"S\", \"y\", \"pack\"), \"\\n\";\n",
);

const NATIVE_SELF_PARENT_STATIC_SOURCE_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "class StaticBoundaryRoot {\n",
    "    protected static function hidden(&$slot, $value) { $slot = strtolower($value); return \"R\" . $slot; }\n",
    "    public static function inherited($value) { return \"I\" . strtoupper($value); }\n",
    "}\n",
    "class StaticBoundaryMid extends StaticBoundaryRoot {\n",
    "    private static function local(&$slot, $value) { $slot = $value; return \"M\" . $value; }\n",
    "    public static function relay($value) {\n",
    "        $slot = \"seed\";\n",
    "        $left = parent::hidden($slot, $value);\n",
    "        $middle = self::local($slot, strtoupper($value));\n",
    "        return $left . \":\" . $middle . \":\" . parent::inherited($value);\n",
    "    }\n",
    "}\n",
    "class StaticBoundaryLeaf extends StaticBoundaryMid {}\n",
    "echo StaticBoundaryMid::relay(\"Go\"), \"|\", StaticBoundaryLeaf::inherited(\"ok\"), \"\\n\";\n",
);

const NATIVE_LATE_STATIC_SOURCE_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "class LateStaticRoot {\n",
    "    protected static function hidden($value) { return \"R\" . strtoupper($value); }\n",
    "    public static function name($value = \"seed\", ...$tail) { return \"root:\" . $value . \":\" . ($tail[0] ?? \"empty\"); }\n",
    "    public static function relay($value) {\n",
    "        return static::name($value, \"tail\") . \":\" . static::hidden($value);\n",
    "    }\n",
    "}\n",
    "class LateStaticLeaf extends LateStaticRoot {}\n",
    "echo LateStaticRoot::relay(\"go\"), \"|\", LateStaticLeaf::relay(\"up\"), \"\\n\";\n",
);

const NATIVE_DECLARED_CLASS_CONSTRUCTOR_SOURCE: &str = concat!(
    "<?php\n",
    "class Box {\n",
    "    public $name;\n",
    "    public function __construct($value = \"Ada\") {\n",
    "        $this->name = strtoupper($value);\n",
    "    }\n",
    "    public function label() { return $this->name; }\n",
    "}\n",
    "class Packet {\n",
    "    public $code;\n",
    "    public function __construct($code) { $this->code = $code; }\n",
    "}\n",
    "class Guarded {\n",
    "    public $name;\n",
    "    public $tag;\n",
    "    public function __construct($skip = true, $value = \"run\") {\n",
    "        $this->name = \"start\";\n",
    "        if ($skip) {\n",
    "            $this->name = \"SKIP\";\n",
    "            return;\n",
    "        }\n",
    "        $this->name = strtoupper($value);\n",
    "        $this->tag = $value;\n",
    "    }\n",
    "}\n",
    "$box = new Box();\n",
    "echo $box->name, \"|\";\n",
    "$named = new Box(\"Grace\");\n",
    "echo $named->label(), \"|\";\n",
    "$packet = new Packet(7);\n",
    "echo $packet->code, \"|\";\n",
    "echo (new Box(\"Temp\"))->label(), \"|\";\n",
    "$guarded = new Guarded();\n",
    "echo $guarded->name, \":\", empty($guarded->tag) ? \"none\" : $guarded->tag, \"|\";\n",
    "$run = new Guarded(false, \"loop\");\n",
    "echo $run->name, \":\", $run->tag, \"\\n\";\n",
);

const NATIVE_DECLARED_CLASS_DYNAMIC_CONSTRUCTOR_NEW_SOURCE: &str = concat!(
    "<?php\n",
    "class DefaultBox {\n",
    "    public $name;\n",
    "    public function __construct($value = \"Ada\") { $this->name = strtoupper($value); }\n",
    "}\n",
    "class Packet {\n",
    "    public $code;\n",
    "    public function __construct($code) { $this->code = $code; }\n",
    "}\n",
    "class BaseCtor {\n",
    "    public $value;\n",
    "    public function __construct($value = \"base\") { $this->value = $value; }\n",
    "}\n",
    "class ChildCtor extends BaseCtor { public $own; }\n",
    "function mark($value) { echo $value; return $value; }\n",
    "$class = \"defaultbox\";\n",
    "$box = new $class();\n",
    "echo $box->name, \":\", get_debug_type($box), \"|\";\n",
    "$class = \"Packet\";\n",
    "$packet = new $class(mark(\"side\"));\n",
    "echo \":\", $packet->code, \":\", get_debug_type($packet), \"|\";\n",
    "$class = \"ChildCtor\";\n",
    "$child = new $class(\"kid\");\n",
    "echo ($child instanceof BaseCtor) ? \"base\" : \"no\";\n",
    "echo \":\", $child->value, \":\", get_debug_type($child), \"|\";\n",
    "$class = \"DefaultBox\";\n",
    "echo (new $class(\"Grace\"))->name, \"\\n\";\n",
);

const NATIVE_DECLARED_CLASS_CONSTRUCTOR_VALUE_RETURN_SOURCE: &str = concat!(
    "<?php\n",
    "class ReturnedValue {}\n",
    "class Box {\n",
    "    public $name;\n",
    "    public function __construct($value = \"bad\") {\n",
    "        $this->name = \"before\";\n",
    "        return new ReturnedValue();\n",
    "    }\n",
    "}\n",
    "new Box(\"x\");\n",
    "echo \"after\\n\";\n",
);

const NATIVE_DECLARED_CLASS_INHERITANCE_SOURCE: &str = concat!(
    "<?php\n",
    "class Other {}\n",
    "class Base {\n",
    "    public $baseName;\n",
    "    public function init($value = \"Ada\") {\n",
    "        $this->baseName = $value;\n",
    "        return $this->baseName;\n",
    "    }\n",
    "    public static function tag($value = \"BASE\") { return strtolower($value); }\n",
    "}\n",
    "class Mid extends Base { public $midName; }\n",
    "class Child extends Mid {\n",
    "    public $childName;\n",
    "    public function child($value) {\n",
    "        $this->childName = $value;\n",
    "        return $this->baseName . \":\" . $this->childName;\n",
    "    }\n",
    "}\n",
    "class CtorBase {\n",
    "    public $value;\n",
    "    public function __construct($value = \"made\") { $this->value = $value; }\n",
    "}\n",
    "class CtorChild extends CtorBase { public $own; }\n",
    "$child = new Child();\n",
    "echo $child instanceof Base ? \"B\" : \"-\";\n",
    "echo $child instanceof Mid ? \"M\" : \"-\";\n",
    "echo $child instanceof Child ? \"C\" : \"-\";\n",
    "echo $child instanceof Other ? \"O\" : \"-\";\n",
    "echo \"|\";\n",
    "echo empty($child->baseName) ? \"E\" : \"N\";\n",
    "echo \":\";\n",
    "$child->baseName = \"root\";\n",
    "echo $child->baseName;\n",
    "echo \"|\";\n",
    "echo $child->init(\"next\"), \":\", $child->baseName;\n",
    "echo \"|\";\n",
    "echo $child->child(\"leaf\");\n",
    "echo \"|\";\n",
    "$method = \"init\";\n",
    "echo $child->$method(\"dyn\"), \":\", $child->baseName;\n",
    "echo \"|\";\n",
    "echo Child::tag(\"LOUD\");\n",
    "echo \"|\";\n",
    "echo $child::tag(\"SHOUT\");\n",
    "echo \"|\";\n",
    "$ctor = new CtorChild(\"ctor\");\n",
    "echo $ctor->value, \"\\n\";\n",
);

const NATIVE_BRANCH_STATE_MERGE_SOURCE: &str = concat!(
    "<?php\n",
    "$flags = [\"go\" => \"1\", \"stop\" => \"0\"];\n",
    "$label = \"base\";\n",
    "if ($flags[\"go\"]) {\n",
    "    $label = \"then\";\n",
    "} else {\n",
    "    $label = \"else\";\n",
    "}\n",
    "echo $label, \"|\";\n",
    "$score = 10;\n",
    "if ($flags[\"stop\"]) {\n",
    "    $score = 1;\n",
    "}\n",
    "echo $score, \"|\";\n",
    "if ($score > 5) {\n",
    "    $word = \"high\";\n",
    "    echo \"H\";\n",
    "} else {\n",
    "    $word = \"low\";\n",
    "    echo \"L\";\n",
    "}\n",
    "echo $word;\n",
);

const NATIVE_BRANCH_NATIVE_VALUE_OWNER_SOURCE: &str = concat!(
    "<?php\n",
    "$flags = [\"take\" => \"1\", \"skip\" => \"\"];\n",
    "if ($flags[\"take\"]) {\n",
    "    $value = strtoupper(\"go\");\n",
    "} else {\n",
    "    $value = strrev(\"dab\");\n",
    "}\n",
    "echo $value, \"|\";\n",
    "$carry = strtolower(\"KEEP\");\n",
    "if ($flags[\"skip\"]) {\n",
    "    $carry = strtoupper(\"bad\");\n",
    "}\n",
    "echo $carry, \"|\";\n",
    "if ($flags[\"take\"]) {\n",
    "    $picked = array_sum([2, 3]);\n",
    "} else {\n",
    "    $picked = array_product([2, 3]);\n",
    "}\n",
    "echo $picked;\n",
);

const NATIVE_BRANCH_LOCAL_VALUE_CLEANUP_SOURCE: &str = concat!(
    "<?php\n",
    "$flags = [\"take\" => \"1\", \"skip\" => \"0\"];\n",
    "if ($flags[\"take\"]) {\n",
    "    array_sum([2, 3]);\n",
    "    echo \"T\";\n",
    "} else {\n",
    "    array_product([4, 5]);\n",
    "    echo \"E\";\n",
    "}\n",
    "echo \"|\";\n",
    "if ($flags[\"skip\"]) {\n",
    "    array_sum([9]);\n",
    "    echo \"bad\";\n",
    "}\n",
    "echo \"done\";\n",
);

const NATIVE_BRANCH_LOCAL_NON_VALUE_OWNER_CLEANUP_SOURCE: &str = concat!(
    "<?php\n",
    "$flags = [\"take\" => \"1\", \"skip\" => \"0\"];\n",
    "$mark = \"base\";\n",
    "if ($flags[\"take\"]) {\n",
    "    [\"discarded\"];\n",
    "    echo \"abc\"[1];\n",
    "    $mark = \"T\";\n",
    "} else {\n",
    "    [\"else-discarded\"];\n",
    "    echo \"xyz\"[2];\n",
    "    $mark = \"E\";\n",
    "}\n",
    "echo \"|\", $mark, \"|\";\n",
    "if ($flags[\"skip\"]) {\n",
    "    [\"unused\"];\n",
    "    echo \"bad\"[0];\n",
    "}\n",
    "echo \"done\";\n",
);

const NATIVE_STATE_STABLE_WHILE_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"ab\", \"cd\"];\n",
    "while (current($items) !== false) {\n",
    "    array_sum([1, 2]);\n",
    "    echo strtoupper(current($items));\n",
    "    next($items);\n",
    "}\n",
    "echo \"|\";\n",
    "$box = [\"letters\" => [\"x\" => \"ef\", \"y\" => \"gh\"]];\n",
    "while (current($box[\"letters\"]) !== false) {\n",
    "    echo key($box[\"letters\"]), \"=\", current($box[\"letters\"]), \";\";\n",
    "    next($box[\"letters\"]);\n",
    "}\n",
    "echo \"|\";\n",
    "$empty = [];\n",
    "while (current($empty) !== false) { echo \"bad\"; }\n",
    "echo \"done\";\n",
);

const NATIVE_WHILE_LOOP_TRANSFER_SOURCE: &str = concat!(
    "<?php\n",
    "$flags = [\"skip\" => \"1\", \"stop\" => \"1\"];\n",
    "$items = [\"a\", \"b\"];\n",
    "while (current($items) !== false) {\n",
    "    echo strtoupper(current($items));\n",
    "    next($items);\n",
    "    if ($flags[\"skip\"]) {\n",
    "        array_sum([1, 2]);\n",
    "        continue;\n",
    "    }\n",
    "    echo \"bad\";\n",
    "}\n",
    "echo \"|\";\n",
    "$more = [\"x\", \"y\"];\n",
    "while (current($more) !== false) {\n",
    "    echo strtoupper(current($more));\n",
    "    if ($flags[\"stop\"]) {\n",
    "        array_sum([3, 4]);\n",
    "        break;\n",
    "    }\n",
    "    next($more);\n",
    "}\n",
    "echo \"|done\";\n",
);

const NATIVE_STATE_STABLE_FOR_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"ab\", \"cd\"];\n",
    "for (; current($items) !== false; next($items)) {\n",
    "    array_sum([1, 2]);\n",
    "    echo strtoupper(current($items));\n",
    "}\n",
    "echo \"|\";\n",
    "$more = [\"e\", \"f\", \"g\", \"h\"];\n",
    "for (; current($more) !== false; next($more)) {\n",
    "    echo strtoupper(current($more));\n",
    "    next($more);\n",
    "    continue;\n",
    "    echo \"bad\";\n",
    "}\n",
    "echo \"|\";\n",
    "$stop = [\"x\", \"y\"];\n",
    "for (; current($stop) !== false; next($stop)) {\n",
    "    echo strtoupper(current($stop));\n",
    "    break;\n",
    "}\n",
    "echo \"|done\";\n",
);

const NATIVE_STATE_STABLE_DO_WHILE_SOURCE: &str = concat!(
    "<?php\n",
    "$keep = true;\n",
    "do {\n",
    "    if ($keep) { echo \"K\"; }\n",
    "    $keep = false;\n",
    "} while ($keep);\n",
    "echo \"|\";\n",
    "$i = 0;\n",
    "do {\n",
    "    array_sum([1, 2]);\n",
    "    echo $i;\n",
    "    $i = ($i << 1) | 1;\n",
    "    if ($i == 1) {\n",
    "        continue;\n",
    "    }\n",
    "    echo \"!\";\n",
    "} while ($i < 3);\n",
    "echo \"|\";\n",
    "$f = 1.5;\n",
    "do {\n",
    "    echo $f;\n",
    "    $f = $f + 1.0;\n",
    "} while ($f < 3.0);\n",
    "echo \"|\";\n",
    "$flags = [\"go\" => \"1\"];\n",
    "do {\n",
    "    echo \"B\";\n",
    "    break;\n",
    "} while ($flags[\"go\"]);\n",
    "echo \"|done\";\n",
);

const NATIVE_LOOP_CARRIED_SCALAR_SOURCE: &str = concat!(
    "<?php\n",
    "$i = 0;\n",
    "while ($i < 3) {\n",
    "    echo $i;\n",
    "    $i = ($i << 1) | 1;\n",
    "}\n",
    "echo \"|\";\n",
    "$keep = true;\n",
    "$seen = 0;\n",
    "while ($keep) {\n",
    "    echo $seen;\n",
    "    $seen = ($seen << 1) | 1;\n",
    "    if ($seen >= 1) {\n",
    "        $keep = false;\n",
    "    }\n",
    "}\n",
    "echo \"|\";\n",
    "for ($j = 1; $j < 5; $j = $j << 1) {\n",
    "    echo $j;\n",
    "}\n",
);

const NATIVE_LOOP_CARRIED_FLOAT_SOURCE: &str = concat!(
    "<?php\n",
    "$f = 1.5;\n",
    "while ($f < 4.0) {\n",
    "    $f = $f + 0.5;\n",
    "}\n",
    "echo $f;\n",
);

const NATIVE_MULTI_LEVEL_LOOP_TRANSFER_SOURCE: &str = concat!(
    "<?php\n",
    "$outer = 0;\n",
    "$inner = 0;\n",
    "while ($outer < 2) {\n",
    "    echo $outer;\n",
    "    $inner = 0;\n",
    "    while ($inner < 1) {\n",
    "        if ($outer == 0) {\n",
    "            $outer = 1;\n",
    "            continue 2;\n",
    "        }\n",
    "        if ($outer == 1) {\n",
    "            break 2;\n",
    "        }\n",
    "        $inner = 1;\n",
    "        echo \"bad\";\n",
    "    }\n",
    "    echo \"after\";\n",
    "    $outer = 2;\n",
    "}\n",
    "echo \"|\";\n",
    "$item = 0;\n",
    "$again = 0;\n",
    "for (; $item < 2; ) {\n",
    "    echo $item;\n",
    "    $again = 0;\n",
    "    for (; $again < 1; ) {\n",
    "        if ($item == 0) {\n",
    "            $item = 1;\n",
    "            continue 2;\n",
    "        }\n",
    "        if ($item == 1) {\n",
    "            break 2;\n",
    "        }\n",
    "        $again = 1;\n",
    "        echo \"bad\";\n",
    "    }\n",
    "    echo \"after\";\n",
    "    $item = 2;\n",
    "}\n",
    "echo \"|done\";\n",
);

const NATIVE_SWITCH_DISPATCH_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"kind\" => \"b\", \"fall\" => 1];\n",
    "switch ($items[\"kind\"]) {\n",
    "    case \"a\":\n",
    "        echo \"A\";\n",
    "        break;\n",
    "    case strtolower(\"B\"):\n",
    "        array_sum([1, 2]);\n",
    "        echo \"B\";\n",
    "        break;\n",
    "    default:\n",
    "        echo \"D\";\n",
    "}\n",
    "echo \"|\";\n",
    "switch ($items[\"fall\"]) {\n",
    "    case 1:\n",
    "        echo \"one\";\n",
    "    default:\n",
    "        echo \"default\";\n",
    "    case 2:\n",
    "        echo \"two\";\n",
    "        break;\n",
    "}\n",
    "echo \"|\";\n",
    "switch (\"later\") {\n",
    "    default:\n",
    "        echo \"default\";\n",
    "        break;\n",
    "    case strtolower(\"LATER\"):\n",
    "        echo \"later\";\n",
    "        break;\n",
    "}\n",
    "echo \"|done\";\n",
);

const NATIVE_STATE_STABLE_GOTO_SOURCE: &str = concat!(
    "<?php\n",
    "echo \"A\";\n",
    "goto second;\n",
    "echo \"bad\";\n",
    "array_sum([99]);\n",
    "first:\n",
    "echo \"C\";\n",
    "goto done;\n",
    "second:\n",
    "echo \"B\";\n",
    "goto first;\n",
    "done:\n",
    "echo \"D\";\n",
);

const NATIVE_TRY_FINALLY_NORMAL_FLOW_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"flag\" => \"1\"];\n",
    "try {\n",
    "    echo \"try|\";\n",
    "    if ($items[\"flag\"]) {\n",
    "        echo strtoupper(\"body\");\n",
    "    }\n",
    "    array_sum([1, 2]);\n",
    "} catch (Exception $e) {\n",
    "    echo \"catch\";\n",
    "} finally {\n",
    "    echo \"|finally|\";\n",
    "}\n",
    "echo \"after\";\n",
);

const NATIVE_TRY_FINALLY_RETURN_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"flag\" => \"old\"];\n",
    "try {\n",
    "    echo \"try|\";\n",
    "    array_sum([1, 2]);\n",
    "    return $items[\"flag\"] = array_sum([3, 4]);\n",
    "} finally {\n",
    "    echo \"finally:\", $items[\"flag\"], \"|\";\n",
    "}\n",
    "echo \"after\";\n",
);

const NATIVE_FUNCTION_TRY_FINALLY_FRAME_SOURCE: &str = concat!(
    "<?php\n",
    "function finish($value) {\n",
    "    try {\n",
    "        echo \"try:\";\n",
    "        return $value;\n",
    "    } finally {\n",
    "        echo \"finally:\";\n",
    "    }\n",
    "}\n",
    "function fallthrough($value) {\n",
    "    try {\n",
    "        echo \"body:\";\n",
    "        $value = strrev($value);\n",
    "    } finally {\n",
    "        echo \"cleanup:\";\n",
    "    }\n",
    "    return $value;\n",
    "}\n",
    "function nested_finally($value) {\n",
    "    try {\n",
    "        try {\n",
    "            return $value;\n",
    "        } finally {\n",
    "            echo \"inner:\";\n",
    "        }\n",
    "    } finally {\n",
    "        echo \"outer:\";\n",
    "    }\n",
    "}\n",
    "echo finish(\"go\"), \"|\", fallthrough(\"abc\"), \"|\", nested_finally(\"done\"), \"|after\";\n",
);

const NATIVE_TRY_FINALLY_LOOP_TRANSFER_SOURCE: &str = concat!(
    "<?php\n",
    "$i = 0;\n",
    "while ($i < 2) {\n",
    "    try {\n",
    "        echo \"try\", $i, \"|\";\n",
    "        if ($i === 0) {\n",
    "            $i = 1;\n",
    "            continue;\n",
    "        }\n",
    "        break;\n",
    "    } finally {\n",
    "        echo \"finally\", $i, \"|\";\n",
    "    }\n",
    "    echo \"bad\";\n",
    "}\n",
    "echo \"after\", $i;\n",
);

const NATIVE_TRY_FINALLY_NESTED_LOOP_TRANSFER_SOURCE: &str = concat!(
    "<?php\n",
    "$i = 0;\n",
    "while ($i < 1) {\n",
    "    $i = 1;\n",
    "    try {\n",
    "        try {\n",
    "            break;\n",
    "        } finally {\n",
    "            echo \"inner|\";\n",
    "        }\n",
    "    } finally {\n",
    "        echo \"outer|\";\n",
    "    }\n",
    "}\n",
    "echo \"after\";\n",
);

const NATIVE_TRY_FINALLY_INNER_LOOP_TRANSFER_SOURCE: &str = concat!(
    "<?php\n",
    "$i = 0;\n",
    "try {\n",
    "    while ($i < 1) {\n",
    "        $i = 1;\n",
    "        break;\n",
    "    }\n",
    "    echo \"inside|\";\n",
    "} finally {\n",
    "    echo \"finally\";\n",
    "}\n",
);

const NATIVE_TOP_LEVEL_RETURN_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"flag\" => \"1\"];\n",
    "$held = strtoupper(\"held\");\n",
    "echo \"before|\";\n",
    "if ($items[\"flag\"]) {\n",
    "    array_sum([1, 2]);\n",
    "    return strtoupper(\"ignored\");\n",
    "}\n",
    "echo $held;\n",
);

const NATIVE_DISCARDED_VALUE_STATEMENT_CLEANUP_SOURCE: &str = concat!(
    "<?php\n",
    "array_sum([2, 3]);\n",
    "echo \"A\";\n",
    "array_product([4, 5]);\n",
    "echo \"B\";\n",
);

const NATIVE_FOREACH_PRIOR_CURSOR_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"a\" => \"A\", \"b\" => \"B\"];\n",
    "$k = \"old-key\";\n",
    "$v = \"old-value\";\n",
    "foreach ($items as $k => $v) { echo $k, \"=\", $v, \";\"; }\n",
    "echo \"|\", $k, \"=\", $v;\n",
    "$literalKey = \"before-key\";\n",
    "$literalValue = \"before-value\";\n",
    "foreach ([\"x\" => \"R\"] as $literalKey => $literalValue) { echo \"|\", strtoupper($literalValue); }\n",
    "echo \"|\", $literalKey, \"=\", $literalValue;\n",
    "$empty = [];\n",
    "$carry = \"keep\";\n",
    "foreach ($empty as $carry) { echo \"bad\"; }\n",
    "echo \"|\", $carry;\n",
);

const NATIVE_BY_REFERENCE_FOREACH_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"a\" => \"ab\", \"b\" => \"cd\"];\n",
    "foreach ($items as $k => &$value) { echo $k, \"=\", $value, \";\"; $value = strtoupper($value); }\n",
    "echo \"|\", $items[\"a\"], \":\", $items[\"b\"];\n",
    "$nested = [];\n",
    "$nested[\"outer\"][\"x\"] = \"hi\";\n",
    "foreach ($nested[\"outer\"] as &$leaf) { $leaf = $leaf . \"!\"; }\n",
    "echo \"|\", $nested[\"outer\"][\"x\"];\n",
);

const NATIVE_VALUE_TERNARY_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"flag\" => \"1\", \"word\" => \"go\"];\n",
    "$choice = $items[\"flag\"] ? strtoupper($items[\"word\"]) : escapeshellarg(\"A\0B\");\n",
    "echo $choice, \"|\";\n",
    "$items[\"flag\"] = \"0\";\n",
    "echo $items[\"flag\"] ? escapeshellarg(\"A\0B\") : strrev(\"ko\");\n",
);

const NATIVE_VALUE_SHORT_TERNARY_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"word\" => \"go\", \"empty\" => \"\"];\n",
    "echo $items[\"word\"] ?: exit(\"bad\");\n",
    "echo \"|\";\n",
    "echo $items[\"empty\"] ?: strrev(\"ko\");\n",
    "echo \"|\";\n",
    "echo array_sum([0]) ?: strtoupper(\"fallback\");\n",
    "echo \"|\";\n",
    "echo array_sum([2]) ?: exit(\"bad\");\n",
);

const NATIVE_EXIT_STRING_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"message\" => \"bye\"];\n",
    "echo \"before|\";\n",
    "exit($items[\"message\"]);\n",
    "echo \"after\";\n",
);

const NATIVE_EXIT_NO_ARG_SOURCE: &str = concat!(
    "<?php\n",
    "echo \"before|\";\n",
    "exit();\n",
    "echo \"after\";\n",
);

const NATIVE_EXIT_NULL_SOURCE: &str = concat!(
    "<?php\n",
    "$status = null;\n",
    "echo \"before|\";\n",
    "exit($status);\n",
    "echo \"after\";\n",
);

const NATIVE_EXIT_STATUS_SOURCE: &str = concat!(
    "<?php\n",
    "$status = [\"code\" => 5];\n",
    "echo \"before|\";\n",
    "die($status[\"code\"]);\n",
    "echo \"after\";\n",
);

const NATIVE_EXIT_UNSUPPORTED_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"flag\" => true];\n",
    "echo \"before|\";\n",
    "exit($items[\"flag\"]);\n",
    "echo \"after\";\n",
);

const NATIVE_EXIT_SYMBOL_CLEANUP_SOURCE: &str = concat!(
    "<?php\n",
    "$root = [\"message\" => \"bye\"];\n",
    "$alias =& $root;\n",
    "exit($alias[\"message\"]);\n",
    "echo \"after\";\n",
);

const NATIVE_EXIT_REQUEST_CLEANUP_SOURCE: &str = concat!(
    "<?php\n",
    "exit($_GET[\"message\"]);\n",
    "echo \"after\";\n",
);

#[test]
fn native_executable_c_source_routes_direct_strings_and_scalars_through_runtime_helpers() {
    let program = parse(
        "<?php\necho \"native link\\n\";\nprint \"runtime string\";\necho 42;\nprint true;\necho 1.25;\necho false;\necho null;\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(source.contains("phpc_native_string_from_bytes"), "{source}");
    assert!(source.contains("phpc_native_value_from_scalar"), "{source}");
    assert!(
        source.contains("phpc_native_value_from_string_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_value_echo_stdout"),
        "{source}"
    );
    assert_eq!(
        source
            .matches("phpc_native_value_from_scalar(scalar_")
            .count(),
        5,
        "{source}"
    );
    assert_eq!(
        source
            .matches("phpc_native_value_format_stdout_with_diagnostic(value_")
            .count(),
        7,
        "{source}"
    );
    assert!(source.contains("PHPC_NATIVE_VALUE_FORMAT_ECHO"), "{source}");
    assert!(
        source.contains("phpc_NativeDiagnosticHandle stdout_diagnostic_")
            && source.contains("phpc_native_diagnostic_report(stdout_diagnostic_"),
        "{source}"
    );
    assert!(!source.contains("printf(\"%s\", \"native link"), "{source}");
    assert!(!source.contains("printf(\"%lld\""), "{source}");
    assert!(!source.contains("printf(\"%g\""), "{source}");
    assert!(!source.contains("printf(\"%s\", \"1\")"), "{source}");
}

#[test]
fn native_executable_c_source_materializes_binary_string_values_with_explicit_lengths() {
    let program = parse(concat!(
        "<?php\n$payload = \"A",
        "\0",
        "B\";\necho strlen($payload), \":\", $payload[1], \":\", $payload;\n"
    ))
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("extern phpc_NativeValueHandle phpc_native_value_from_string_bytes_with_diagnostic(const uint8_t *ptr, size_t len, phpc_NativeDiagnosticHandle *diagnostic);"),
        "{source}"
    );
    assert!(
        source.contains("static const uint8_t phpc_native_value_bytes_")
            && source.contains("{65, 0, 66}"),
        "{source}"
    );
    assert!(
        source.contains(
            "phpc_native_value_from_string_bytes_with_diagnostic(phpc_native_value_bytes_"
        ) && source.contains(", 3, &native_value_diagnostic_"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_offset_read_source")
            && source.contains("phpc_native_conversion_source_value(native_value_handle_"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_array_family_through_runtime_contract() {
    let program = parse(concat!(
        "<?php\n$payload = \"A",
        "\0",
        "B|\u{00ff}\";\n$parts = explode(\"|\", $payload, 2);\n$chunks = str_split($parts[1], 1);\necho strlen($parts[0]), \":\", bin2hex($parts[0]), \":\", bin2hex($chunks[0]), \":\", bin2hex($chunks[1]), \"\\n\";\n",
    ))
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("extern phpc_NativeValueHandle phpc_native_string_array_operation_with_reference_slots_with_diagnostic(phpc_NativeValueHandle subject, phpc_NativeReferenceHandle subject_reference, phpc_NativeValueHandle operand, phpc_NativeReferenceHandle operand_reference, int64_t limit_or_length, uint8_t flags, uint8_t operation, phpc_NativeDiagnosticHandle *diagnostic);"),
        "{source}"
    );
    assert!(
        source
            .matches("phpc_native_string_array_operation_with_reference_slots_with_diagnostic(")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_offset_operation_with_diagnostic")
            && source.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_to_int64_with_diagnostic")
            || source.contains("phpc_native_value_to_int_with_reference_slot_with_diagnostic"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_reports_owned_diagnostics_through_shared_consumer() {
    let program = parse("<?php\necho \"left\";\n$s = \"AB\";\n$s[0] = \"Z\";\necho $s;\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("extern size_t phpc_native_diagnostic_report"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_diagnostic_report(diagnostic_"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_diagnostic_report(string_offset_write_diagnostic_"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_diagnostic_message_stderr(diagnostic_"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_diagnostic_free(diagnostic_"),
        "{source}"
    );
    assert!(
        !source.contains("phpc_native_diagnostic_message_stderr(string_offset_write_diagnostic_"),
        "{source}"
    );
    assert_no_diagnostic_report_double_free(&source);
}

#[test]
fn native_executable_c_source_routes_exit_through_runtime_result_and_cleanup() {
    for (source_text, expected_cleanup) in [
        (
            NATIVE_EXIT_SYMBOL_CLEANUP_SOURCE,
            "phpc_native_symbol_table_free(",
        ),
        (
            NATIVE_EXIT_REQUEST_CLEANUP_SOURCE,
            "phpc_native_request_state_free(",
        ),
    ] {
        let program = parse(source_text).unwrap();
        let source = emit_native_executable_c_source(&program).unwrap();
        let body = main_body(&source);

        assert!(source.contains("phpc_NativeExitResult"), "{source}");
        assert!(
            source.contains("phpc_native_exit_value_result_and_free"),
            "{source}"
        );
        assert!(source.contains("PHPC_NATIVE_EXIT_OK"), "{source}");
        let return_pos = body.find("return native_exit_result_").unwrap_or_else(|| {
            panic!("exit should terminate main after the runtime result is consumed:\n{source}")
        });
        assert!(
            body[..return_pos].contains(expected_cleanup),
            "early exit should run owned cleanup before returning:\n{source}"
        );
        assert!(
            !source.contains("termination lowering rejects exit()/die()"),
            "{source}"
        );
    }
}

#[test]
fn emit_exe_links_and_runs_scalar_runtime_value_echo_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("scalar_runtime_value_echo.php");
    let output_path = native_link_output_path("scalar_runtime_value_echo");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        "<?php\necho 42;\nprint true;\necho 1.5;\necho false;\necho null;\n",
    )
    .expect("write scalar native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native scalar source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"4211.5");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_native_byte_string_value_boundary_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "native_byte_string_value_boundary",
        concat!(
            "<?php\n$payload = \"A",
            "\0",
            "B\";\necho strlen($payload), \":\", $payload[1], \":\", $payload, \"\\n\";\n"
        ),
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native byte-string executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"3:\0:A\0B\n");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_native_string_array_operation_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "native_string_array_operation",
        concat!(
            "<?php\n$payload = \"A",
            "\0",
            "B|\u{00ff}\";\n$parts = explode(\"|\", $payload, 2);\n$chunks = str_split($parts[1], 1);\necho strlen($parts[0]), \":\", bin2hex($parts[0]), \":\", bin2hex($chunks[0]), \":\", bin2hex($chunks[1]), \"\\n\";\n"
        ),
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native string-array executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"3:410042:c3:bf\n");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_native_output_buffer_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("native_output_buffer", NATIVE_OUTPUT_BUFFER_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native output-buffer executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"A0B42:5:hidden|AB|0\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_discarded_expression_diagnostic_result_operands() {
    if !has_cc() {
        return;
    }

    let program = parse(NATIVE_DIAGNOSTIC_RESULT_DISCARDED_EXPR_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    assert!(
        source.contains("phpc_native_diagnostic_result_report_stderr_list_and_free"),
        "discarded expression statements should report/free through diagnostic-result sinks:\n{source}"
    );
    assert!(
        source.matches("phpc_native_diagnostic_result_from_value")
            .count()
            >= 3,
        "discarded expression statements should materialize owned diagnostic-result value operands:\n{source}"
    );

    let (source_path, output_path) = compile_native_link_fixture(
        "diagnostic_result_discarded_expr",
        NATIVE_DIAGNOSTIC_RESULT_DISCARDED_EXPR_SOURCE,
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run diagnostic-result executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"ok");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_output_diagnostic_result_operands() {
    if !has_cc() {
        return;
    }

    let program = parse(NATIVE_DIAGNOSTIC_RESULT_OUTPUT_OPERANDS_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    assert!(
        source.contains("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free"),
        "echo/print operands should report/free through diagnostic-result echo sinks:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_diagnostic_result_from_value")
            .count()
            >= 6,
        "echo/print operands should materialize owned diagnostic-result value operands:\n{source}"
    );

    let (source_path, output_path) = compile_native_link_fixture(
        "diagnostic_result_output_operands",
        NATIVE_DIAGNOSTIC_RESULT_OUTPUT_OPERANDS_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run diagnostic-result output executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"A2VV|Array|done\n");
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("Warning: Array to string conversion"),
        "stderr:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_class_object_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("declared_class_object", NATIVE_DECLARED_CLASS_OBJECT_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run declared-class executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"YYN|object:object:Box|Packet\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_class_dynamic_new_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_class_dynamic_new",
        NATIVE_DECLARED_CLASS_DYNAMIC_NEW_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-class dynamic-new executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"N:named:D1D2object:Beta:D3base:kid:D4Alpha\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_object_property_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_object_property",
        NATIVE_DECLARED_CLASS_PROPERTY_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-object-property executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"EM|Ada:Ada:S:N|7\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_typed_declared_instance_property_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class TypedBox {\n",
        "    public int $count = 5;\n",
        "    public string $name = \"Ada\";\n",
        "    public function label() { return $this->name . \":\" . $this->count; }\n",
        "    public function bump($value) { $this->count = $value; }\n",
        "}\n",
        "$first = new TypedBox();\n",
        "$second = new TypedBox();\n",
        "$first->bump(\"7\");\n",
        "$second->count = \"8\";\n",
        "echo $first->label(), \"|\", $second->label(), \"\\n\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("typed_declared_instance_property", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run typed declared instance-property executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"Ada:7|Ada:8\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_typed_declared_instance_property_reports_type_failure() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class TypedFailure { public int $count; }\n",
        "$box = new TypedFailure();\n",
        "$box->count = [];\n",
        "echo \"after\\n\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("typed_declared_instance_property_failure", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run typed declared instance-property failure executable: {error}")
    });

    assert!(
        !run.status.success(),
        "typed property failure should exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("typed property TypedFailure::$count expects int, got array"),
        "stderr:\n{stderr}"
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_static_property_storage_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_static_property_storage",
        NATIVE_DECLARED_CLASS_STATIC_PROPERTY_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-static-property executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"2:5:5:ready\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_self_parent_static_property_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "self_parent_static_property",
        NATIVE_SELF_PARENT_STATIC_PROPERTY_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run self/parent static-property executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"go:5:root|go:5\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_late_static_property_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("late_static_property", NATIVE_LATE_STATIC_PROPERTY_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run late-static property executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"root:5|leaf:14|5:leaf:18\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_object_property_unset_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_object_property_unset",
        NATIVE_DECLARED_CLASS_PROPERTY_UNSET_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-object-property-unset executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"01|01|Z\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_class_instanceof_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_class_instanceof",
        NATIVE_DECLARED_CLASS_INSTANCEOF_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-class-instanceof executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"YYNYN\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_class_method_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("declared_class_method", NATIVE_DECLARED_CLASS_METHOD_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run declared-class-method executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"Ada:Ada|Grace:Grace|7:7|GO|Temp|Tail\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_this_property_assignment_in_generated_method_frames() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class Box {\n",
        "    public $name;\n",
        "    public $alt;\n",
        "    public function literal($value) {\n",
        "        $this->name = strtoupper($value);\n",
        "        return $this->name;\n",
        "    }\n",
        "    public function dynamic($property, $value) {\n",
        "        $this->$property = $value;\n",
        "        return $value;\n",
        "    }\n",
        "}\n",
        "$box = new Box();\n",
        "echo $box->literal(\"ada\"), \":\", $box->name, \"|\";\n",
        "echo $box->dynamic(\"alt\", 7), \":\", $box->alt, \"\\n\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("this_property_method_frame_assignment", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run method-frame $this property assignment executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"ADA:ADA|7:7\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_this_property_assignment_reports_mutation_failure_from_method_frame() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class Box {\n",
        "    private $secret;\n",
        "    public function store($value) {\n",
        "        $this->secret = $value;\n",
        "        echo \"after\";\n",
        "    }\n",
        "}\n",
        "$box = new Box();\n",
        "$box->store(\"bad\");\n",
        "echo \"tail\\n\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("this_property_method_frame_assignment_failure", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run method-frame $this property assignment failure executable: {error}")
    });

    assert!(
        !run.status.success(),
        "non-public property mutation failure should terminate the native frame"
    );
    assert_eq!(run.stdout, b"");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("non-public property") && stderr.contains("Box::$secret"),
        "stderr:\n{stderr}"
    );
    assert!(!stderr.contains("after") && !stderr.contains("tail"));

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_class_dynamic_method_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_class_dynamic_method",
        NATIVE_DECLARED_CLASS_DYNAMIC_METHOD_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-class-dynamic-method executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"Ada:Ada|7:7|GO|loud|Tail\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_class_static_method_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_class_static_method",
        NATIVE_DECLARED_CLASS_STATIC_METHOD_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-class-static-method executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"ADA|GRACE|7|GO|drop\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_object_static_method_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_object_static_method",
        NATIVE_DECLARED_OBJECT_STATIC_METHOD_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-object-static-method executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"ADA|shout|7|GO|7|drop\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_object_static_default_variadic_source_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_object_static_default_variadic_source_call",
        NATIVE_DECLARED_OBJECT_STATIC_DEFAULT_VARIADIC_SOURCE_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run declared object-static default/variadic source-call executable: {error}"
        )
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"O:S:empty|O:x:tail\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_method_static_source_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_method_static_source_call",
        NATIVE_DECLARED_METHOD_STATIC_SOURCE_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared method/static source-call executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"AGO|Bloud|Sloud|TGO\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_method_static_default_variadic_source_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_method_static_default_variadic_source_call",
        NATIVE_DECLARED_METHOD_STATIC_DEFAULT_VARIADIC_SOURCE_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run declared method/static default/variadic source-call executable: {error}"
        )
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"R:D:empty|R:x:tail|S:S:empty|S:y:pack\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_dynamic_method_source_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_dynamic_method_source_call",
        NATIVE_DECLARED_DYNAMIC_METHOD_SOURCE_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run dynamic method source-call executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"L:left-a|left-a|R:right-b|right-b\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_reports_dynamic_method_source_call_lookup_failure() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_dynamic_method_source_call_failure",
        NATIVE_DECLARED_DYNAMIC_METHOD_SOURCE_CALL_FAILURE_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run dynamic method source-call failure executable: {error}")
    });

    assert!(
        !run.status.success(),
        "dynamic method source-call lookup miss should fail through the shared runtime diagnostic"
    );
    assert_eq!(run.stdout, b"");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("native method invocation failed")
            && stderr.contains("Method missing is not registered")
            && !stderr.contains("after"),
        "stderr:\n{stderr}"
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_dynamic_method_source_call_class_context_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_dynamic_method_source_call_class_context",
        NATIVE_DECLARED_DYNAMIC_METHOD_CLASS_CONTEXT_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run dynamic method class-context executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"pGO\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_declared_class_method_dispatch_reports_runtime_misses() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class Box { public function open() { return \"open\"; } }\n",
        "class Packet {}\n",
        "$packet = new Packet();\n",
        "echo $packet->open();\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("declared_class_method_miss", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-class-method miss executable: {error}")
    });

    assert!(
        !run.status.success(),
        "method miss should fail through the shared runtime diagnostic"
    );
    assert!(
        String::from_utf8_lossy(&run.stderr)
            .contains("native method dispatch for Packet::open is not supported"),
        "stderr:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_class_constructor_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_class_constructor",
        NATIVE_DECLARED_CLASS_CONSTRUCTOR_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-class-constructor executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"ADA|GRACE|7|TEMP|SKIP:none|LOOP:loop\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_class_dynamic_constructor_new_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_class_dynamic_constructor_new",
        NATIVE_DECLARED_CLASS_DYNAMIC_CONSTRUCTOR_NEW_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-class dynamic-constructor-new executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        run.stdout,
        b"ADA:DefaultBox|side:side:Packet|base:kid:ChildCtor|GRACE\n"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_runtime_dynamic_method_source_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_runtime_dynamic_method_source_call",
        NATIVE_DECLARED_RUNTIME_DYNAMIC_METHOD_SOURCE_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run runtime dynamic method source-call executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"L:left-a|left-a|R:right-b|right-b\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_magic_dynamic_method_source_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_magic_dynamic_method_source_call",
        NATIVE_DECLARED_DYNAMIC_METHOD_MAGIC_BLOCKED_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run magic dynamic method source-call executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"known:A|magic:missing:B|hidden:in\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_self_parent_static_source_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "self_parent_static_source_call",
        NATIVE_SELF_PARENT_STATIC_SOURCE_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run self/parent static source-call executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"Rgo:MGO:IGO|IOK\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_late_static_source_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "late_static_source_call",
        NATIVE_LATE_STATIC_SOURCE_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run late-static source-call executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"root:go:tail:RGO|root:up:tail:RUP\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_declared_class_inheritance_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_class_inheritance",
        NATIVE_DECLARED_CLASS_INHERITANCE_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-class-inheritance executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        run.stdout,
        b"BMC-|E:root|next:next|next:leaf|dyn:dyn|loud|shout|ctor\n"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_declared_class_constructor_reports_runtime_arity_misses() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class Box { public function __construct($value) { $this->value = $value; } }\n",
        "new Box();\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("declared_class_constructor_miss", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-class-constructor miss executable: {error}")
    });

    assert!(
        !run.status.success(),
        "constructor arity miss should fail through the shared runtime diagnostic"
    );
    assert!(
        String::from_utf8_lossy(&run.stderr)
            .contains("native method dispatch for Box::__construct is not supported"),
        "stderr:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_declared_class_constructor_value_return_reports_diagnostic() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "declared_class_constructor_value_return",
        NATIVE_DECLARED_CLASS_CONSTRUCTOR_VALUE_RETURN_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run declared-class-constructor value-return executable: {error}")
    });

    assert!(
        !run.status.success(),
        "constructor value return should fail through the shared runtime diagnostic"
    );
    assert_eq!(run.stdout, b"");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("native method dispatch for Box::__construct is not supported")
            && stderr.contains("constructor value returns are not implemented"),
        "stderr:\n{stderr}"
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_runs_leading_numeric_arithmetic_value_results_with_diagnostics() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("leading_numeric_arithmetic.php");
    let output_path = native_link_output_path("leading_numeric_arithmetic");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(&source_path, NATIVE_LEADING_NUMERIC_ARITHMETIC_SOURCE)
        .expect("write leading-numeric arithmetic native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native leading-numeric arithmetic source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(
        run.status.success(),
        "native executable stdout:\n{}\nnative executable stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"10|7|10|3|1|-6");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.matches("leading-numeric string operand").count() >= 6,
        "each arithmetic value operation should report the PHP leading-numeric warning:\n{stderr}"
    );

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_native_exit_string_value_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("native_exit_string", NATIVE_EXIT_STRING_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native exit executable: {error}"));

    assert!(run.status.success(), "native exit executable failed");
    assert_eq!(run.stdout, b"before|bye");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_native_exit_without_argument_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("native_exit_no_arg", NATIVE_EXIT_NO_ARG_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native no-arg exit executable: {error}"));

    assert!(run.status.success(), "native no-arg exit executable failed");
    assert_eq!(run.stdout, b"before|");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_native_exit_null_value_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("native_exit_null", NATIVE_EXIT_NULL_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native null exit executable: {error}"));

    assert!(run.status.success(), "native null exit executable failed");
    assert_eq!(run.stdout, b"before|");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_native_exit_integer_status_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("native_exit_status", NATIVE_EXIT_STATUS_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native exit status executable: {error}"));

    assert_eq!(
        run.status.code(),
        Some(5),
        "native exit status should be the PHP integer operand"
    );
    assert_eq!(run.stdout, b"before|");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_reports_native_exit_unsupported_value_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("native_exit_unsupported", NATIVE_EXIT_UNSUPPORTED_SOURCE);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native exit unsupported executable: {error}")
    });

    assert_eq!(run.status.code(), Some(1));
    assert_eq!(run.stdout, b"before|");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains(
            "native exit failed: argument must be null, int, or string in the current subset, got bool"
        ),
        "{stderr}"
    );
    assert!(!stderr.contains("after"), "{stderr}");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn native_executable_c_source_routes_by_value_foreach_through_array_lvalue_owner() {
    let program = parse(
        "<?php\n$a = [\"x\" => \"ab\", \"y\" => \"cd\"];\nforeach ($a as $k => $v) { echo $k, \"=\", strtoupper($v), \";\"; }\n$b = [];\n$b[\"nested\"][\"n\"] = \"ef\";\nforeach ($b[\"nested\"] as $nk => $nv) { print $nk; print \"=\"; print strtoupper($nv); print \";\"; }\nforeach ([\"lit\" => \"gh\"] as $lk => $lv) { echo $lk, \"=\", strtoupper($lv), \";\"; }\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_foreach_iterable_result"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_lvalue_owner_array"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_array_foreach_iterable_key_result")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        body.matches("phpc_native_array_foreach_iterable_value_result")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        body.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        !source.contains("assembly array lowering rejects arrays"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_preserves_prior_foreach_cursor_storage() {
    let program = parse(NATIVE_FOREACH_PRIOR_CURSOR_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches("phpc_NativeValueHandle array_foreach_cursor_storage_")
            .count()
            >= 5,
        "prior key/value targets and empty-loop fallback should keep post-loop cursor storage:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_clone(array_foreach_key_value_")
            && body.contains("phpc_native_value_clone(array_foreach_value_value_"),
        "foreach cursor storage should clone key and value handles through the shared native-value owner ABI:\n{source}"
    );
    assert!(
        body.contains("if (array_foreach_cursor_storage_")
            && body.contains("phpc_native_value_free(array_foreach_cursor_storage_"),
        "cursor storage updates should release the previous owned value before each replacement:\n{source}"
    );
    assert!(
        !source.contains("assembly array lowering rejects arrays"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_foreach_body_array_offset_unsets_through_lvalue_owner() {
    let program = parse(
        "<?php\n$items = [\"x\" => \"A\", \"y\" => \"B\", \"z\" => \"C\"];\n$other = [\"z\" => \"Z\", \"keep\" => \"K\"];\n$nested = [\"outer\" => [\"drop\" => \"D\", \"keep\" => \"N\"]];\nforeach ($items as $key => $value) { unset($items[$key], $other[\"z\"], $nested[\"outer\"][\"drop\"]); echo $key, \":\", $value, \";\"; }\necho \"|\", isset($items[\"x\"]) ? 1 : 0, isset($items[\"z\"]) ? 1 : 0, isset($other[\"z\"]) ? 1 : 0, isset($other[\"keep\"]) ? 1 : 0, isset($nested[\"outer\"][\"drop\"]) ? 1 : 0, isset($nested[\"outer\"][\"keep\"]) ? 1 : 0;\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_foreach_iterable_result"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_array_lvalue_owner_value_operation_result")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UNSET")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        !source.contains("assembly mutation lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_pointer_builtins_through_lvalue_owner_results() {
    let program = parse(
        "<?php\n$items = [10 => \"first\", 20 => \"second\", 30 => \"third\"];\n$box = [\"nested\" => [\"n1\", \"n2\", \"n3\"]];\necho current($items), \"|\", key($items), \"|\", next($items), \"|\", key($items), \"|\", prev($items), \"|\", end($items), \"|\", reset($items), \"|\", next($box[\"nested\"]), \"|\", end($box[\"nested\"]), \"|\", reset($box[\"nested\"]);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_pointer_result"),
        "{source}"
    );
    for tag in [
        "PHPC_NATIVE_ARRAY_LVALUE_POINTER_CURRENT",
        "PHPC_NATIVE_ARRAY_LVALUE_POINTER_KEY",
        "PHPC_NATIVE_ARRAY_LVALUE_POINTER_NEXT",
        "PHPC_NATIVE_ARRAY_LVALUE_POINTER_PREV",
        "PHPC_NATIVE_ARRAY_LVALUE_POINTER_RESET",
        "PHPC_NATIVE_ARRAY_LVALUE_POINTER_END",
    ] {
        assert!(source.contains(tag), "{source}");
    }
    assert!(
        body.matches("phpc_native_array_lvalue_owner_pointer_result")
            .count()
            >= 10,
        "{source}"
    );
    assert!(
        !source.contains("assembly array lowering rejects arrays"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_sort_builtins_through_lvalue_owner_results() {
    let program = parse(
        "<?php\n$values = [3, 1, 2];\n$mode = 1;\nsort($values, $mode);\n$box = [\"items\" => [\"a\" => \"b2\", \"b\" => \"b10\"], \"keys\" => [\"a\" => 1, \"b\" => 2], \"natural\" => [\"z\" => \"img10\", \"y\" => \"img2\", \"x\" => \"img01\"], \"case\" => [\"up\" => \"Img12\", \"low\" => \"img2\", \"first\" => \"img1\"]];\nasort($box[\"items\"], 2);\nkrsort($box[\"keys\"]);\nnatsort($box[\"natural\"]);\nnatcasesort($box[\"case\"]);\nrsort($values);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_sort_result"),
        "{source}"
    );
    for tag in [
        "PHPC_NATIVE_ARRAY_LVALUE_SORT_SORT",
        "PHPC_NATIVE_ARRAY_LVALUE_SORT_ASORT",
        "PHPC_NATIVE_ARRAY_LVALUE_SORT_KRSORT",
        "PHPC_NATIVE_ARRAY_LVALUE_SORT_NATSORT",
        "PHPC_NATIVE_ARRAY_LVALUE_SORT_NATCASESORT",
        "PHPC_NATIVE_ARRAY_LVALUE_SORT_RSORT",
    ] {
        assert!(source.contains(tag), "{tag}\n\n{source}");
    }
    assert!(
        body.matches("phpc_native_array_lvalue_owner_sort_result")
            .count()
            >= 6,
        "{source}"
    );
    assert!(
        body.contains("phpc_NativeValueHandle array_sort_operands_"),
        "sort flags should be materialized as PHP value operands:\n{source}"
    );
    assert!(
        !source.contains("assembly array lowering rejects arrays"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_mutation_builtins_through_lvalue_owner_results() {
    let program = parse(
        "<?php\n$items = [1, 2];\n$box = [\"items\" => [\"a\", \"b\"], \"head\" => 9];\narray_push($items, $box[\"value\"] = 3, $box[\"fallback\"] ??= 4);\narray_pop($box[\"items\"]);\narray_shift($box[\"items\"]);\narray_unshift($box[\"items\"], $box[\"head\"] += 1);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_array_mutation_result"),
        "{source}"
    );
    for tag in [
        "PHPC_NATIVE_ARRAY_LVALUE_MUTATION_PUSH",
        "PHPC_NATIVE_ARRAY_LVALUE_MUTATION_POP",
        "PHPC_NATIVE_ARRAY_LVALUE_MUTATION_SHIFT",
        "PHPC_NATIVE_ARRAY_LVALUE_MUTATION_UNSHIFT",
    ] {
        assert!(source.contains(tag), "{tag}\n\n{source}");
    }
    assert!(
        body.matches("phpc_native_array_lvalue_owner_array_mutation_result")
            .count()
            >= 4,
        "{source}"
    );
    assert!(
        body.contains("phpc_NativeValueHandle array_mutation_operands_"),
        "mutation operands should be materialized as PHP value handles:\n{source}"
    );
    assert!(
        !source.contains("assembly array lowering rejects arrays"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_callback_builtins_through_shared_result() {
    let program = parse(
        "<?php\n$box = [];\n$filtered = array_filter([0, \"Ada\", \"\", \"Bea\"], $box[\"callback\"] ??= null, $box[\"mode\"] = \"1\");\n$mapped = array_map(null, [\"name\" => \"Ada\", 5 => \"five\"]);\narray_reduce([1, 2], \"sum\", 0);\necho $filtered[1], $mapped[\"name\"];\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_array_callback_result"),
        "{source}"
    );
    for tag in [
        "PHPC_NATIVE_VALUE_ARRAY_CALLBACK_FILTER",
        "PHPC_NATIVE_VALUE_ARRAY_CALLBACK_MAP",
        "PHPC_NATIVE_VALUE_ARRAY_CALLBACK_REDUCE",
    ] {
        assert!(source.contains(tag), "{tag}\n\n{source}");
    }
    assert!(
        body.matches("phpc_native_value_array_callback_result")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        body.contains("phpc_NativeValueHandle native_value_array_callback_args_"),
        "callback operands should be materialized as PHP value handles:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_free(native_value_array_callback_"),
        "owned callback results must be cleaned:\n{source}"
    );
    assert!(
        !source.contains("assembly array lowering rejects arrays"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_query_family_through_shared_value_operation() {
    let program = parse(
        "<?php\n$items = [\"zero\" => 0, \"string_zero\" => \"0\", \"name\" => \"Ada\", \"empty\" => \"\"];\n$labels = [\"a\", \"b\"];\n$counted = [\"a\", \"a\", 2];\n$nums = [1, \"2\", 3];\n$fillKeys = [\"x\", 7];\n$combineKeys = [\"left\", \"right\"];\n$combineValues = [1, 2];\necho array_keys($items, \"0\", false)[0], \",\", array_keys($items, \"0\", false)[1], \"|\", in_array(\"Ada\", $items, true), \"|\", array_search(\"\", $items), \"|\", array_flip($labels)[\"a\"], array_flip($labels)[\"b\"], \"|\", array_count_values($counted)[\"a\"], \",\", array_count_values($counted)[2], \"|\", array_sum($nums), \"|\", array_product($nums), \"|\", array_fill_keys($fillKeys, \"v\")[\"x\"], \",\", array_fill_keys($fillKeys, \"v\")[7], \"|\", array_combine($combineKeys, $combineValues)[\"left\"], \",\", array_combine($combineKeys, $combineValues)[\"right\"];\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_array_query_operation_with_diagnostic"
        ),
        "{source}"
    );
    for operation_tag in [
        "PHPC_NATIVE_VALUE_ARRAY_QUERY_KEYS_MATCHING",
        "PHPC_NATIVE_VALUE_ARRAY_QUERY_CONTAINS",
        "PHPC_NATIVE_VALUE_ARRAY_QUERY_SEARCH",
        "PHPC_NATIVE_VALUE_ARRAY_QUERY_FLIP",
        "PHPC_NATIVE_VALUE_ARRAY_QUERY_COUNT_VALUES",
        "PHPC_NATIVE_VALUE_ARRAY_QUERY_SUM",
        "PHPC_NATIVE_VALUE_ARRAY_QUERY_PRODUCT",
        "PHPC_NATIVE_VALUE_ARRAY_QUERY_FILL_KEYS",
        "PHPC_NATIVE_VALUE_ARRAY_QUERY_COMBINE",
    ] {
        assert!(body.contains(operation_tag), "{operation_tag}\n\n{source}");
    }
    assert!(body.contains("PHPC_NATIVE_ARRAY_QUERY_STRICT"), "{source}");
    assert!(
        body.matches(" = phpc_native_value_array_query_operation_with_diagnostic(")
            .count()
            >= 9,
        "{source}"
    );
    assert!(
        body.contains("phpc_native_value_free(native_value_array_query"),
        "owned query results must be cleaned:\n{source}"
    );
    assert!(
        body.contains("phpc_native_diagnostic_report(array_query_diagnostic_"),
        "query diagnostics must use the shared diagnostic consumer:\n{source}"
    );
    assert!(
        !source.contains("assembly array lowering rejects arrays"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_change_key_case_through_array_query_operation() {
    let program = parse(
        "<?php\n$items = [\"Name\" => \"Ada\", \"MiXeD\" => \"mixed\", 7 => \"seven\"];\necho array_change_key_case($items)[\"name\"], \"|\", array_change_key_case($items, 1)[\"MIXED\"], \"|\", array_change_key_case($items, -1)[7];\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_array_query_operation_with_diagnostic"
        ),
        "{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_VALUE_ARRAY_QUERY_CHANGE_KEY_CASE"),
        "{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_array_query_operation_with_diagnostic(")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        body.contains("phpc_native_value_free(native_value_array_query"),
        "owned key-case query results must be cleaned:\n{source}"
    );
    assert!(
        !source.contains("assembly array lowering rejects arrays"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_column_through_operand_list_query_operation() {
    let program = parse(
        "<?php\n$rows = [];\n$rows[] = [\"id\" => \"a\", \"name\" => \"Ada\"];\n$rows[] = [\"id\" => \"b\", \"name\" => \"Bee\"];\n$rows[] = [\"name\" => \"NoId\"];\n$names = array_column($rows, \"name\", \"id\");\n$whole = array_column($rows, null);\necho $names[\"a\"], \"|\", $names[\"b\"], \"|\", $names[0], \"|\", $whole[0][\"name\"], \"|\", $whole[2][\"name\"];\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_array_query_operation_with_operands_and_diagnostic"
        ),
        "{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_VALUE_ARRAY_QUERY_COLUMN"),
        "{source}"
    );
    assert!(
        body.contains("phpc_NativeValueHandle array_query_operands_"),
        "array_column operands should be materialized as a reusable operand list:\n{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_array_query_operation_with_operands_and_diagnostic(")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        body.contains("phpc_native_value_free(native_value_array_query"),
        "owned array_column query results must be cleaned:\n{source}"
    );
    assert!(
        !source.contains("assembly array lowering rejects arrays"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_value_result_offset_reads_through_shared_boundary() {
    let program = parse(
        "<?php\necho array_map(null, [\"L\", \"M\"])[1];\necho \"|\";\necho ((array) \"Q\")[0];\necho \"|\";\necho ((array) \"NO\")[0][1];\necho \"|\";\necho (\"A\" . \"B\")[1];\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_offset_operation_with_diagnostic"
        ),
        "{source}"
    );
    assert!(
        body.contains("phpc_native_value_array_callback_result")
            && body.contains("phpc_native_value_cast_result")
            && body.contains("phpc_native_value_binary_result"),
        "{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_offset_operation_with_diagnostic(")
            .count()
            >= 5,
        "value-result offset reads should share the value-offset ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_format_stdout_with_diagnostic(value_offset_read"),
        "offset-read values should feed the existing value formatter:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_free(value_offset_read"),
        "owned offset-read results must be cleaned up:\n{source}"
    );
    assert!(
        !source.contains("assembly array lowering rejects arrays"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_native_value_truthiness_through_runtime_abi() {
    let program = parse(NATIVE_VALUE_TRUTHINESS_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("extern _Bool phpc_native_value_is_truthy(phpc_NativeValueHandle value);"),
        "{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_is_truthy(").count() >= 6,
        "unary and XOR native-value operands should share the truthiness ABI:\n{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_offset_operation_with_diagnostic(")
            .count()
            >= 4,
        "array offset values should remain value-offset producers:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_array_query_operation_with_diagnostic"),
        "array query values should feed the same truthiness consumer:\n{source}"
    );
    assert!(
        body.contains("!="),
        "logical XOR should combine converted truthiness operands:\n{source}"
    );
    assert!(
        !source.contains("assembly logical lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("assembly unary lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_truthiness_consumers_through_value_boundary() {
    let program = parse(NATIVE_VALUE_TRUTHINESS_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source
            .contains("extern _Bool phpc_native_value_truthy_with_reference_slot_with_diagnostic("),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_truthy_with_reference_slot_with_diagnostic(")
            .count()
            >= 6
            && !source.contains(" = phpc_native_value_is_truthy(")
            && !source.contains(" = phpc_native_value_truthy_with_diagnostic("),
        "{source}"
    );
    assert!(
        source.contains(" = phpc_native_offset_read_source(")
            && source.contains(" = phpc_native_value_array_query_operation_with_diagnostic("),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_empty_static_values_through_truthiness_boundary() {
    let program = parse(concat!(
        "<?php\n",
        "$zero = \"0\";\n",
        "$payload = \"A\\0B\";\n",
        "$intZero = 0;\n",
        "$intOne = 1;\n",
        "echo empty($zero);\n",
        "echo empty($payload);\n",
        "echo empty($intZero);\n",
        "echo empty($intOne);\n",
    ))
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source
            .contains("extern _Bool phpc_native_value_truthy_with_reference_slot_with_diagnostic(")
            && source.contains("extern phpc_NativeValueHandle phpc_native_value_from_scalar("),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_truthy_with_reference_slot_with_diagnostic(")
            .count()
            >= 4
            && !source.contains(" = phpc_native_value_is_truthy(")
            && !source.contains(" = phpc_native_value_truthy_with_diagnostic("),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_native_value_unary_not_through_truthiness_boundary() {
    let program = parse(concat!(
        "<?php\n",
        "$payload = \"A\\0B|0|\";\n",
        "echo !$payload[0];\n",
        "echo !$payload[1];\n",
        "echo !$payload[4];\n",
        "$refPayload = \"0\";\n",
        "$ref =& $refPayload;\n",
        "echo !$ref;\n",
    ))
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source
            .contains("extern _Bool phpc_native_value_truthy_with_reference_slot_with_diagnostic(")
            && source
                .contains("extern phpc_NativeConversionResult phpc_native_offset_read_source("),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_truthy_with_reference_slot_with_diagnostic(")
            .count()
            >= 4
            && source.matches("bool_value = ((!(").count() >= 4
            && !source.contains(" = phpc_native_value_is_truthy(")
            && !source.contains(" = phpc_native_value_truthy_with_diagnostic("),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_reference_truthiness_operands_without_value_clone_detour() {
    let program = parse(concat!(
        "<?php\n",
        "$payload = \"0\";\n",
        "$ref =& $payload;\n",
        "echo !$ref;\n",
        "echo empty($ref);\n",
        "$payload2 = \"A\\0B\";\n",
        "$ref2 =& $payload2;\n",
        "echo !$ref2;\n",
        "echo empty($ref2);\n",
    ))
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source
            .contains("extern _Bool phpc_native_value_truthy_with_reference_slot_with_diagnostic(")
            && source.contains("typedef struct { void *ptr; } phpc_NativeReferenceHandle;"),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_truthy_with_reference_slot_with_diagnostic(")
            .count()
            >= 2
            && !source.contains(" = phpc_native_reference_value_clone(")
            && !source.contains(" = phpc_native_value_is_truthy(")
            && !source.contains(" = phpc_native_value_truthy_with_diagnostic("),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_dynamic_logical_and_or_through_short_circuit_branches() {
    let program = parse(NATIVE_SHORT_CIRCUIT_LOGICAL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("_Bool native_logical_result_"),
        "dynamic logical &&/|| should materialize a boolean result variable:\n{source}"
    );
    assert!(
        body.contains("if (native_value_truthy_") && body.contains("if (!(native_value_truthy_"),
        "logical && and || should guard selected RHS evaluation with C branches:\n{source}"
    );
    assert!(
        body.contains("native_exit_result_")
            && body.contains("strtoupper_result_")
            && body.contains("strrev_result_"),
        "RHS producers should live in selected short-circuit branch bodies:\n{source}"
    );
    assert!(
        body.contains("native_logical_result_") && body.contains("? (\"T\") : (\"F\")"),
        "downstream ternary consumers should read the logical result after short-circuiting:\n{source}"
    );
    assert!(
        !source.contains("assembly logical lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_rejects_short_circuit_logical_rhs_state_merges() {
    let program = parse(
        "<?php\n$items = [\"one\" => \"1\"];\necho ($items[\"one\"] && ($value = strtoupper(\"x\"))) ? \"T\" : \"F\";\necho $value;\n",
    )
    .unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert!(
        error.message.contains("logical lowering rejects"),
        "{error:?}"
    );
}

#[test]
fn native_executable_c_source_routes_scoped_if_branches_through_truthiness_boundary() {
    let program = parse(NATIVE_SCOPED_IF_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains(" = phpc_native_value_is_truthy(")
            && body.contains("if (native_value_truthy_"),
        "native value conditions should route through the shared truthiness ABI:\n{source}"
    );
    assert!(
        body.contains(" = phpc_native_value_compare_result("),
        "comparison conditions should feed the shared native value comparison result ABI:\n{source}"
    );
    assert!(
        body.contains(" = phpc_native_value_binary_result(")
            && body.contains(" = phpc_native_value_compare_result("),
        "if conditions should compose existing native value and comparison producers:\n{source}"
    );
    assert!(
        body.contains("} else {"),
        "generated C should preserve both scoped branch bodies:\n{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_leading_numeric_arithmetic_through_value_results() {
    let program = parse(NATIVE_LEADING_NUMERIC_ARITHMETIC_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches(" = phpc_native_value_binary_result(").count() >= 5,
        "binary leading-numeric arithmetic should use the shared native value operation ABI:\n{source}"
    );
    assert!(
        body.contains(" = phpc_native_value_unary_result("),
        "unary leading-numeric arithmetic should use the shared native value operation ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_diagnostic_report("),
        "warning-bearing successful value operations should flow through the shared diagnostic reporter:\n{source}"
    );
    assert!(
        !source.contains("assembly arithmetic lowering rejects")
            && !source.contains("assembly scalar arithmetic coercion rejects"),
        "leading-numeric arithmetic should not fall through arithmetic blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_output_buffers_through_shared_runtime_abi() {
    let program = parse(NATIVE_OUTPUT_BUFFER_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_output_buffer_operation_with_diagnostic"
        ),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_output_buffer_operation_with_diagnostic(")
            .count()
            >= 12,
        "all lowerable output-buffer operations should route through the shared runtime ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "captured output should continue through the diagnostic-aware stdout formatter:\n{source}"
    );
    assert!(
        !source.contains("output-buffer lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_declared_class_objects_through_runtime_abi() {
    let program = parse(NATIVE_DECLARED_CLASS_OBJECT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_new_declared_class_with_diagnostic"),
        "{source}"
    );
    assert!(
        source
            .matches("phpc_native_value_new_declared_class_with_diagnostic")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        source.contains("declared_class_property_name_ptrs")
            && source.contains("declared_class_property_visibilities"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_type_predicate"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_type_name_result"),
        "{source}"
    );
    assert!(
        !source.contains("object-instantiation lowering rejects")
            && !source.contains("object/class lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_declares_user_class_metadata_for_shared_metadata_surfaces() {
    let program = parse(concat!(
        "<?php\n",
        "class UserBase { public $baseSlot; public static function BaseStatic() { } }\n",
        "class UserChild extends UserBase { public $childSlot; public function run() { } }\n",
        "echo class_exists(\"UserChild\") ? \"1\" : \"0\";\n",
        "echo method_exists(\"UserChild\", \"basestatic\") ? \"1\" : \"0\";\n",
        "echo property_exists(\"UserChild\", \"baseSlot\") ? \"1\" : \"0\";\n",
        "echo property_exists(\"UserChild\", \"missing\") ? \"1\" : \"0\";\n",
    ))
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_declare_user_class_bytes")
            && source.contains("phpc_native_declare_user_class_parent_bytes")
            && source.contains("phpc_native_declare_user_class_method_bytes")
            && source.contains("phpc_native_declare_user_class_property_bytes"),
        "class declarations should populate the shared user-class metadata registry:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_class_metadata_exists_with_diagnostic"),
        "class/member metadata predicates should consume the shared runtime metadata boundary:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_value_metadata_consumers_through_runtime_registry() {
    let program = parse(concat!(
        "<?php\n",
        "class UserBase { public $baseSlot; public function baseRun() { } }\n",
        "class UserChild extends UserBase { public $childSlot; public function run() { } }\n",
        "echo get_parent_class(\"UserChild\");\n",
        "$parents = class_parents(\"UserChild\", false);\n",
        "$methods = get_class_methods(\"UserChild\");\n",
        "$vars = get_class_vars(\"UserChild\");\n",
        "$declared = get_declared_classes();\n",
        "echo in_array(\"run\", $methods) ? \"1\" : \"0\";\n",
    ))
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_declare_user_class_bytes")
            && source.contains("phpc_native_value_class_metadata_value_with_diagnostic"),
        "value-returning class metadata consumers should use the shared runtime registry:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_user_class_metadata_registry_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class UserBase { public $baseSlot; public static function BaseStatic() { } }\n",
        "class UserChild extends UserBase { public $childSlot; public function run() { } }\n",
        "echo class_exists(\"UserChild\") ? \"1\" : \"0\";\n",
        "echo \"|\";\n",
        "echo method_exists(\"UserChild\", \"basestatic\") ? \"1\" : \"0\";\n",
        "echo \"|\";\n",
        "echo property_exists(\"UserChild\", \"baseSlot\") ? \"1\" : \"0\";\n",
        "echo \"|\";\n",
        "echo property_exists(\"UserChild\", \"missing\") ? \"1\" : \"0\";\n",
        "echo \"\\n\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("user_class_metadata_registry", source);

    let run = Command::new(&output_path)
        .output()
        .expect("run user class metadata registry executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1|1|1|0\n");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn native_executable_c_source_lowers_class_alias_metadata_boundary() {
    let program = parse(concat!(
        "<?php\n",
        "class AliasSourceBase { public $baseSlot; public function baseRun() { } }\n",
        "class AliasSourceChild extends AliasSourceBase { public $childSlot; public function run() { } }\n",
        "echo class_alias(\"AliasSourceChild\", \"AliasRuntime\", false) ? \"1\" : \"0\";\n",
        "echo class_exists(\"aliasruntime\", false) ? \"1\" : \"0\";\n",
        "echo method_exists(\"AliasRuntime\", \"baserun\") ? \"1\" : \"0\";\n",
        "echo property_exists(\"AliasRuntime\", \"baseSlot\") ? \"1\" : \"0\";\n",
    ))
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_declare_user_class_alias_bytes_with_diagnostic")
            && source.contains("phpc_native_value_class_metadata_exists_with_diagnostic"),
        "generated C should register class aliases through the shared metadata boundary:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_class_alias_metadata_boundary_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class AliasSourceBase { public $baseSlot; public function baseRun() { } }\n",
        "class AliasSourceChild extends AliasSourceBase { public $childSlot; public function run() { } }\n",
        "echo class_alias(\"AliasSourceChild\", \"AliasRuntime\", false) ? \"alias\" : \"alias-fail\";\n",
        "echo \"|\";\n",
        "echo class_exists(\"aliasruntime\", false) ? \"exists\" : \"missing\";\n",
        "echo \"|\";\n",
        "echo method_exists(\"AliasRuntime\", \"baserun\") ? \"method\" : \"missing-method\";\n",
        "echo \"|\";\n",
        "echo property_exists(\"AliasRuntime\", \"baseSlot\") ? \"property\" : \"missing-property\";\n",
        "echo \"\\n\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("class_alias_metadata_boundary", source);

    let run = Command::new(&output_path)
        .output()
        .expect("run class alias metadata boundary executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "alias|exists|method|property\n"
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_class_alias_missing_default_reports_autoload_boundary() {
    if !has_cc() {
        return;
    }

    let source = "<?php\nclass AliasLoaded {}\necho class_alias(\"MissingAliasSource\", \"AliasMissing\");\n";
    let (source_path, output_path) =
        compile_native_link_fixture("class_alias_missing_autoload_boundary", source);

    let run = Command::new(&output_path)
        .output()
        .expect("run class_alias autoload-boundary executable");
    assert!(
        !run.status.success(),
        "class_alias missing source should report autoload boundary"
    );
    assert!(
        String::from_utf8_lossy(&run.stderr).contains(
            "class_alias(): generated-native autoload for missing source classes is not implemented"
        ),
        "stderr:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_user_class_value_metadata_registry_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class UserBase { public $baseSlot; public function baseRun() { } }\n",
        "class UserChild extends UserBase { public $childSlot; public static function ChildStatic() { } public function run() { } }\n",
        "echo get_parent_class(\"UserChild\");\n",
        "echo \"|\";\n",
        "$parents = class_parents(\"UserChild\", false);\n",
        "echo array_key_exists(\"UserBase\", $parents) ? \"parent\" : \"missing-parent\";\n",
        "echo \"|\";\n",
        "$methods = get_class_methods(\"UserChild\");\n",
        "echo in_array(\"run\", $methods) ? \"run\" : \"missing-run\";\n",
        "echo \"|\";\n",
        "echo in_array(\"baseRun\", $methods) ? \"base\" : \"missing-base\";\n",
        "echo \"|\";\n",
        "echo in_array(\"missingRun\", $methods) ? \"unexpected\" : \"missing-filtered\";\n",
        "echo \"|\";\n",
        "$vars = get_class_vars(\"UserChild\");\n",
        "echo array_key_exists(\"childSlot\", $vars) ? \"childSlot\" : \"missing-childSlot\";\n",
        "echo \"|\";\n",
        "echo array_key_exists(\"baseSlot\", $vars) ? \"baseSlot\" : \"missing-baseSlot\";\n",
        "echo \"|\";\n",
        "$declared = get_declared_classes();\n",
        "echo in_array(\"UserChild\", $declared) ? \"declared\" : \"missing-declared\";\n",
        "echo \"|\";\n",
        "echo in_array(\"stdClass\", $declared) ? \"core-declared\" : \"missing-core\";\n",
        "echo \"|\";\n",
        "$coreMethods = get_class_methods(\"ReflectionClass\");\n",
        "echo in_array(\"getName\", $coreMethods) ? \"core-method\" : \"missing-core-method\";\n",
        "echo \"\\n\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("user_class_value_metadata_registry", source);

    let run = Command::new(&output_path)
        .output()
        .expect("run user class value metadata registry executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "UserBase|parent|run|base|missing-filtered|childSlot|baseSlot|declared|core-declared|core-method\n"
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn native_executable_c_source_lowers_namespace_alias_class_policy_boundary() {
    let program = parse(concat!(
        "<?php\n",
        "namespace App\\Core;\n",
        "use App\\Core\\Service as ImportedService;\n",
        "class Base { public $baseSlot; }\n",
        "class Service extends Base { public $name; public static function label($value) { return $value . \"!\"; } }\n",
        "$class = \"\\\\App\\\\Core\\\\Service\";\n",
        "$service = new ImportedService();\n",
        "$dynamic = new $class();\n",
        "echo ImportedService::class;\n",
        "echo $service instanceof Base ? \"B\" : \"-\";\n",
        "echo $dynamic instanceof ImportedService ? \"S\" : \"-\";\n",
        "echo ImportedService::label(\"go\");\n",
        "echo class_exists(ImportedService::class) ? \"Y\" : \"N\";\n",
        "echo class_exists(\"Missing\", false) ? \"Y\" : \"N\";\n",
    ))
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_declare_user_class_bytes")
            && source.contains("phpc_native_value_new_declared_class_with_relationships_and_diagnostic")
            && source.contains("phpc_native_value_dynamic_class_name_matches")
            && source.contains("phpc_native_value_class_metadata_exists_with_autoload_policy_and_diagnostic"),
        "namespace/import class policy should lower through generated-C class metadata and class-name helpers:\n{source}"
    );
    assert!(
        !source.contains("namespace lowering rejects")
            && !source.contains("class-name constant lowering rejects")
            && !source.contains("object-instantiation lowering rejects"),
        "{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_namespace_alias_class_policy_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "namespace App\\Core;\n",
        "use App\\Core\\Service as ImportedService;\n",
        "class Base { public $baseSlot; }\n",
        "class Service extends Base { public $name; public static function label($value) { return $value . \"!\"; } }\n",
        "$class = \"\\\\App\\\\Core\\\\Service\";\n",
        "$service = new ImportedService();\n",
        "$dynamic = new $class();\n",
        "echo ImportedService::class, \"|\";\n",
        "echo $service instanceof Base ? \"B\" : \"-\";\n",
        "echo $dynamic instanceof ImportedService ? \"S\" : \"-\";\n",
        "echo \"|\";\n",
        "echo ImportedService::label(\"go\"), \"|\";\n",
        "echo class_exists(ImportedService::class) ? \"Y\" : \"N\";\n",
        "echo class_exists(\"Missing\", false) ? \"Y\" : \"N\";\n",
        "echo \"\\n\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("namespace_alias_class_policy", source);

    let run = Command::new(&output_path)
        .output()
        .expect("run namespace alias class-policy executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "App\\Core\\Service|BS|go!|YN\n"
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_links_and_runs_exact_imported_const_alias_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "namespace App\\Values;\n",
        "const ANSWER = 42;\n",
        "const LABEL = \"answer\";\n",
        "use const App\\Values\\ANSWER as picked_number, App\\Values\\LABEL as picked_label;\n",
        "use const PHP_VERSION_ID as runtime_version, PHP_VERSION as runtime_label;\n",
        "echo picked_label, \"=\", picked_number, \"|\";\n",
        "echo runtime_version, \"|\", runtime_label, \"\\n\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("exact_imported_const_alias", source);

    let run = Command::new(&output_path)
        .output()
        .expect("run exact imported const alias executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "answer=42|80300|8.3.0\n"
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_namespaced_class_exists_user_function_takes_exact_precedence() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "namespace App\\Core;\n",
        "function class_exists($name) { return \"local:\" . $name; }\n",
        "echo class_exists(\"Widget\"), \"\\n\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("namespaced_class_exists_exact_user_function", source);

    let run = Command::new(&output_path)
        .output()
        .expect("run namespaced class_exists exact user function executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "local:Widget\n");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn emit_exe_class_exists_missing_default_reports_autoload_boundary() {
    if !has_cc() {
        return;
    }

    let source = "<?php\nclass Loaded {}\necho class_exists(\"Missing\");\n";
    let (source_path, output_path) =
        compile_native_link_fixture("class_exists_missing_autoload_boundary", source);

    let run = Command::new(&output_path)
        .output()
        .expect("run class_exists autoload-boundary executable");
    assert!(
        !run.status.success(),
        "class_exists missing should report autoload boundary"
    );
    assert!(
        String::from_utf8_lossy(&run.stderr).contains(
            "class_exists(): generated-native autoload for missing classes is not implemented"
        ),
        "stderr:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn native_executable_c_source_routes_dynamic_declared_class_new_through_declared_allocation_helpers(
) {
    let program = parse(NATIVE_DECLARED_CLASS_DYNAMIC_NEW_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_new_declared_class_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_new_declared_class_with_ancestors_and_diagnostic"),
        "dynamic new should keep declared-child allocation on the ancestor-aware allocation helper:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_dynamic_class_name_matches"),
        "dynamic new should materialize class-name values and match generated declared-class candidates through the shared class-name helper:\n{source}"
    );
    assert!(
        body.matches("phpc_NativeValueHandle constructor_arg_values_")
            .count()
            >= 4,
        "constructorless named and dynamic new argument lists should be evaluated into reusable native-value arrays before allocation:\n{source}"
    );
    assert!(
        body.matches("phpc_native_value_new_declared_class_with_diagnostic")
            .count()
            >= 5,
        "named and dynamic class-name allocation candidates should share declared-class allocation helpers:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_type_predicate")
            && source.contains("phpc_native_value_type_name_result")
            && source.contains("phpc_native_value_instanceof_class_with_diagnostic")
            && source.contains("phpc_native_value_object_public_property_operation_with_diagnostic"),
        "new-expression results should compose with native value, class-relation, and property consumers:\n{source}"
    );
    assert!(
        !source.contains("object-instantiation lowering rejects")
            && !source.contains("object/class lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_declared_object_properties_through_runtime_abi() {
    let program = parse(NATIVE_DECLARED_CLASS_PROPERTY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_object_public_property_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_value_object_public_property_operation_with_diagnostic")
            .count()
            >= 8,
        "reads, writes, isset, and empty should share the public property ABI:\n{source}"
    );
    for tag in [
        "PHPC_NATIVE_OBJECT_PUBLIC_PROPERTY_READ",
        "PHPC_NATIVE_OBJECT_PUBLIC_PROPERTY_WRITE",
        "PHPC_NATIVE_OBJECT_PUBLIC_PROPERTY_ISSET",
        "PHPC_NATIVE_OBJECT_PUBLIC_PROPERTY_EMPTY",
    ] {
        assert!(source.contains(tag), "{source}");
    }
    assert!(
        !source.contains("object-property lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_typed_declared_instance_properties_through_allocation_contract(
) {
    let program = parse(
        "<?php\nclass TypedBox { public int $count = 5; public ?string $label = null; public array $items = array(\"seed\" => \"base\"); }\n$box = new TypedBox();\n$box->count = \"6\";\necho $box->count;\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains(
            "phpc_native_value_new_declared_class_with_relationships_and_property_metadata_and_diagnostic"
        ),
        "typed/default properties should route through the declared-class property metadata allocation ABI:\n{source}"
    );
    assert!(
        source.contains("declared_class_property_type_decl_ptrs_")
            && source.contains("declared_class_property_type_decl_lens_")
            && source.contains("declared_class_property_default_values_")
            && source.contains("declared_class_property_default_flags_"),
        "type declarations and defaults should be passed as metadata arrays, not source-shape special cases:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_object_property_mutation_operation_with_diagnostic"),
        "known typed property assignments should use the mutation ABI so uninitialized typed slots can be initialized with diagnostics:\n{source}"
    );
    assert!(
        !source.contains("object/class lowering rejects")
            && !source.contains("object-instantiation lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_declared_static_property_reads_writes_through_runtime_storage()
{
    let program = parse(NATIVE_DECLARED_CLASS_STATIC_PROPERTY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    for required in [
        "phpc_NativeStaticPropertyStorageHandle",
        "phpc_native_static_property_storage_new",
        "phpc_native_static_property_storage_declare_class_bytes",
        "phpc_native_static_property_storage_declare_class_parent_bytes",
        "phpc_native_static_property_storage_declare_property_bytes",
        "phpc_native_static_property_storage_register_default_value_and_free",
        "phpc_native_static_property_storage_reset_with_diagnostic",
        "phpc_native_static_property_read_class_with_diagnostic",
        "phpc_native_static_property_write_class_with_diagnostic_and_free",
        "phpc_native_static_property_storage_free",
    ] {
        assert!(source.contains(required), "missing {required}:\n{source}");
    }
    assert!(
        body.matches("phpc_native_static_property_read_class_with_diagnostic")
            .count()
            >= 3,
        "literal static-property reads should use runtime storage:\n{source}"
    );
    assert!(
        body.matches("phpc_native_static_property_write_class_with_diagnostic_and_free")
            .count()
            >= 1,
        "literal static-property writes should use runtime storage:\n{source}"
    );
    assert!(
        !body.contains("static-member lowering rejects"),
        "supported literal declared static-property access should not fall back to static-member rejection:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_self_parent_static_properties_through_relative_runtime_storage(
) {
    let program = parse(NATIVE_SELF_PARENT_STATIC_PROPERTY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    for required in [
        "PHPC_NATIVE_STATIC_PROPERTY_RECEIVER_SELF",
        "PHPC_NATIVE_STATIC_PROPERTY_RECEIVER_PARENT",
        "phpc_native_static_property_read_relative_with_diagnostic",
        "phpc_native_static_property_write_relative_with_diagnostic_and_free",
        "static_property_current_class_name_bytes",
    ] {
        assert!(source.contains(required), "missing {required}:\n{source}");
    }
    assert!(
        source
            .matches("phpc_native_static_property_read_relative_with_diagnostic")
            .count()
            >= 4,
        "self::/parent:: static-property reads should use relative runtime storage:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_static_property_write_relative_with_diagnostic_and_free")
            .count()
            >= 2,
        "self::/parent:: static-property writes should use relative runtime storage:\n{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_CALLABLE_ACCESS_CLASS_CONTEXT")
            && source.contains("method_call_caller_scope_"),
        "self::/parent:: property proof should preserve class-context source-call carriers for method entry:\n{source}"
    );
    assert!(
        !source.contains("static-member lowering rejects"),
        "supported self::/parent:: static-property access should not fall back to static-member rejection:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_late_static_properties_through_called_scope_storage() {
    let program = parse(NATIVE_LATE_STATIC_PROPERTY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    for required in [
        "PHPC_NATIVE_STATIC_PROPERTY_RECEIVER_LATE_STATIC",
        "phpc_NativeStringHandle phpc_called_scope",
        "phpc_native_string_bytes(phpc_called_scope)",
        "phpc_native_string_len(phpc_called_scope)",
        "phpc_native_static_property_read_relative_with_diagnostic",
        "phpc_native_static_property_write_relative_with_diagnostic_and_free",
    ] {
        assert!(source.contains(required), "missing {required}:\n{source}");
    }
    assert!(
        source
            .matches("phpc_native_static_property_read_relative_with_diagnostic")
            .count()
            >= 3,
        "static::$prop reads should route through relative runtime storage with called scope:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_static_property_write_relative_with_diagnostic_and_free")
            .count()
            >= 1,
        "static::$prop writes should route through relative runtime storage with called scope:\n{source}"
    );
    assert!(
        !source.contains("static-member lowering rejects"),
        "supported static::$prop access should not fall back to static-member rejection:\n{source}"
    );
}

#[test]
fn native_executable_c_source_keeps_unsupported_static_property_dynamic_shapes_blocked() {
    for (label, source) in [
        (
            "dynamic class name",
            "<?php\nclass Counter { public static $count = 1; }\n$class = \"Counter\";\necho $class::$count;\n",
        ),
        (
            "self static property outside class context",
            "<?php\nclass Counter { public static $count = 1; }\necho self::$count;\n",
        ),
        (
            "late-static property outside class context",
            "<?php\nclass Counter { public static $count = 1; }\necho static::$count;\n",
        ),
        (
            "object static property receiver",
            "<?php\nclass Counter { public static $count = 1; }\n$object = new Counter();\necho $object::$count;\n",
        ),
    ] {
        let program = parse(source).unwrap();
        let error = match emit_native_executable_c_source(&program) {
            Ok(generated) => panic!("{label} unexpectedly emitted C:\n{generated}"),
            Err(error) => error,
        };

        assert_eq!(error.phase, Phase::Codegen, "{label}: {error:?}");
        assert!(
            error.message.contains("static member")
                || error.message.contains("static property")
                || error.message.contains("object/class lowering rejects"),
            "{label} should remain behind an explicit static-property boundary, got {error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_declared_object_property_unsets_through_runtime_abi() {
    let program = parse(NATIVE_DECLARED_CLASS_PROPERTY_UNSET_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_object_public_property_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_OBJECT_PUBLIC_PROPERTY_UNSET"),
        "generated C should declare the shared object property unset operation tag:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_OBJECT_PUBLIC_PROPERTY_UNSET")
            .count()
            >= 3,
        "direct, chained, and missing public property unsets should share the public property ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_value_object_public_property_operation_with_diagnostic")
            .count()
            >= 10,
        "property read/write/isset/empty/unset should compose through one ABI:\n{source}"
    );
    assert!(
        !source.contains("mutation lowering rejects")
            && !source.contains("object-property lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nonlocal_object_property_assignments_through_reference_owner_commit(
) {
    let program = parse(
        "<?php\nclass Box { public $first; public $second; }\n$box = new Box();\n$box->first = \"literal\";\n$slot = \"second\";\n$box->$slot = \"dynamic\";\necho $box->first, \"|\", $box->second;\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_public_property_reference_with_diagnostic_and_free"),
        "literal and single-known dynamic property assignments should acquire the public-property reference owner:\n{source}"
    );
    assert!(
        body.matches("phpc_native_value_public_property_reference_with_diagnostic_and_free")
            .count()
            >= 2,
        "literal and single-known dynamic writes should both route through property reference owners:\n{source}"
    );
    assert!(
        body.matches("phpc_native_reference_set_value").count() >= 2,
        "property assignment owner commits should write back through the native reference commit path:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_value_object_property_mutation_operation_with_diagnostic")
            && !body.contains(
                "phpc_native_object_property_mutation_operation_with_reference_slots_with_diagnostic"
            )
            && !source.contains("non-local assignment lowering rejects"),
        "external property assignment must not use the temporary mutation shortcut or boundary rejection:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_nonlocal_object_property_assignment_owner_commit_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class LeftValue { public function mark() { return \"A\"; } }\n",
        "class RightValue { public function mark() { return \"B\"; } }\n",
        "class Holder { public $first; public $second; public $label; }\n",
        "$holder = new Holder();\n",
        "echo ($holder->label = \"L\"), \"|\";\n",
        "$holder->first = new LeftValue();\n",
        "echo $holder->first->mark(), \"|\";\n",
        "$holder->first = new RightValue();\n",
        "echo $holder->first->mark(), \"|\";\n",
        "$slot = \"second\";\n",
        "$holder->$slot = new RightValue();\n",
        "echo $holder->second->mark(), \"\\n\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("nonlocal_object_property_owner_commit", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run nonlocal object-property owner-commit executable: {error}")
    });

    assert!(
        run.status.success(),
        "native executable failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"L|A|B|B\n");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn native_executable_c_source_keeps_unsupported_nonlocal_property_assignment_shapes_blocked() {
    for (label, source) in [
        ("unknown dynamic property", "<?php\n$box->$slot = 1;\n"),
        ("nested property", "<?php\n$box->child->name = 1;\n"),
        ("static property", "<?php\nRoot::$name = 1;\n"),
    ] {
        let program = parse(source).unwrap();
        let error = match emit_native_executable_c_source(&program) {
            Ok(generated) => panic!("{label} unexpectedly emitted C:\n{generated}"),
            Err(error) => error,
        };

        assert_eq!(error.phase, Phase::Codegen, "{label}: {error:?}");
        assert!(
            error.message.contains("non-local assignment lowering rejects")
                || error.message.contains("static member"),
            "{label} should remain behind an explicit non-local/static-property assignment boundary, got {error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_reference_backed_dynamic_property_assignment_through_owner_commit(
) {
    let program = parse(
        "<?php\nclass Box { public $payload; }\n$obj = new Box();\n$key = \"payload\";\n$keyRef =& $key;\n$value = \"R\\0Y\";\n$valueRef =& $value;\n$obj->$key = $value;\necho $obj->payload;\n$value = \"Z\";\n$obj->$key = $value;\necho $obj->payload;\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_public_property_reference_with_diagnostic_and_free"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_value_public_property_reference_with_diagnostic_and_free")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        body.matches("phpc_native_reference_set_value").count() >= 2
            && body.contains("phpc_native_value_object_public_property_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        !body.contains("phpc_native_value_object_property_mutation_operation_with_diagnostic")
            && !body.contains(
                "phpc_native_object_property_mutation_operation_with_reference_slots_with_diagnostic"
            )
            && !source.contains("object-property lowering rejects")
            && !source.contains("mutation lowering rejects")
            && !source.contains("reference assignment lowering rejects"),
        "{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_reference_backed_dynamic_property_assignment_owner_commit_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "native_object_property_assignment_reference_slots",
        "<?php\nclass Box { public $payload; }\n$obj = new Box();\n$key = \"payload\";\n$keyRef =& $key;\n$value = \"R\\0Y\";\n$valueRef =& $value;\n$obj->$key = $value;\necho $obj->payload, \"|\";\n$value = \"Z\";\n$obj->$key = $value;\necho $obj->payload, \"\\n\";\n",
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run object-property assignment owner-commit executable: {error}")
    });

    assert!(
        run.status.success(),
        "native executable failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"R0Y|Z\n");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        !stderr.contains("object-property lowering rejects")
            && !stderr.contains("mutation lowering rejects")
            && !stderr.contains("reference assignment lowering rejects"),
        "stderr:\n{stderr}"
    );

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn native_executable_c_source_routes_reference_keys_and_text_membership_through_reference_boundaries(
) {
    let program = parse(
        "<?php\n$name = \"MYSQLI_QUERY\";\n$nameRef =& $name;\n$extension = \"Json\";\n$extensionRef =& $extension;\n$key = \"A\\0B\";\n$keyRef =& $key;\n$numeric = \"42\";\n$numericRef =& $numeric;\n$items = [$keyRef => \"value\", $numericRef => \"number\"];\necho function_exists($nameRef);\necho extension_loaded($extensionRef);\necho $items[$key];\necho \"|\";\necho $items[42];\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeArrayKeyMaterializationResult phpc_native_value_to_array_key_with_reference_slot("
        ) && source.contains(
            "extern _Bool phpc_native_text_membership_with_reference_slot_with_diagnostic("
        ),
        "{source}"
    );
    assert!(
        body.matches(" = phpc_native_text_membership_with_reference_slot_with_diagnostic(")
            .count()
            >= 2,
        "{source}"
    );
    for native_known_name in [
        "(const uint8_t *)\"mysqli_query\"",
        "(const uint8_t *)\"stream_get_contents\"",
        "(const uint8_t *)\"is_uploaded_file\"",
        "(const uint8_t *)\"spl_autoload_register\"",
    ] {
        assert!(
            source.contains(native_known_name),
            "function_exists text-membership table should use the full native-known semantic family: {native_known_name}\n{source}"
        );
    }
    assert!(
        body.matches(
            " = phpc_native_value_to_array_key_with_reference_slot((phpc_NativeValueHandle){0}, "
        )
        .count()
            >= 2,
        "{source}"
    );
    assert!(
        body.contains("phpc_native_array_insert_key_value_with_diagnostic("),
        "{source}"
    );
    assert!(
        !source.contains("assembly native-array value lowering rejects")
            && !source.contains("reference assignment lowering rejects")
            && !source.contains("assembly function-call lowering rejects")
            && !body.contains(" = phpc_native_reference_value_clone("),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_reference_held_native_value_comparisons_through_slot_boundary()
{
    let program = parse(
        "<?php\n$left = \"A\\0B\\xFF\";\n$leftRef =& $left;\n$right = \"A\\0B\\xFF\";\n$rightRef =& $right;\necho $leftRef == $rightRef;\necho $leftRef < \"A\\0C\";\necho $leftRef === $rightRef;\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern _Bool phpc_native_value_comparison_with_reference_slots_with_diagnostic(phpc_NativeValueHandle left, phpc_NativeReferenceHandle left_reference, phpc_NativeValueHandle right, phpc_NativeReferenceHandle right_reference, uint8_t operation, phpc_NativeDiagnosticHandle *diagnostic);"
        ),
        "{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_comparison_with_reference_slots_with_diagnostic(")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        body.contains(", PHPC_NATIVE_VALUE_COMPARISON_EQ, &value_comparison_diagnostic_")
            && body.contains(", PHPC_NATIVE_VALUE_COMPARISON_LT, &value_comparison_diagnostic_")
            && body.contains(
                ", PHPC_NATIVE_VALUE_COMPARISON_STRICT_EQ, &value_comparison_diagnostic_"
            ),
        "{source}"
    );
    assert!(
        !body.contains(" = phpc_native_reference_value_clone(")
            && !source.contains("assembly comparison lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_reference_type_introspection_without_value_clone_detour() {
    let program = parse("<?php\n$value = 7;\n$alias =& $value;\necho gettype($alias);\necho is_int($alias);\n$value = \"text\";\necho get_debug_type($alias);\necho is_string($alias);\necho is_scalar($alias);\n").unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_type_name_with_reference_slot_with_diagnostic("
        ) && source.contains("extern bool phpc_native_value_type_predicate_with_reference_slot_with_diagnostic("),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_type_name_with_reference_slot_with_diagnostic(")
            .count(),
        2,
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_type_predicate_with_reference_slot_with_diagnostic(")
            .count(),
        3,
        "{source}"
    );
    assert!(
        body.contains("phpc_native_symbol_table_reference_for_path(")
            && body.contains("phpc_native_symbol_table_bind_reference_path(")
            && body.contains("phpc_native_symbol_table_set_value_by_path_with_diagnostic(")
            && body.contains("phpc_native_reference_free("),
        "{source}"
    );
    assert!(
        !body.contains(" = phpc_native_reference_value_clone(")
            && !body.contains(" = phpc_native_value_type_name_result(")
            && !body.contains(" = phpc_native_value_type_predicate(")
            && !source.contains("reference assignment lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_reference_int_operands_through_reference_boundary() {
    let program = parse(
        "<?php\n$length = 2;\n$lengthRef =& $length;\n$offset = 1;\n$offsetRef =& $offset;\necho strncmp(\"abcdef\", \"abcxyz\", $lengthRef);\necho strncasecmp(\"ABCDEF\", \"abcxyz\", $lengthRef);\n$length = 3;\necho substr_count(\"abcabc\", \"a\", $offsetRef, $lengthRef);\n$offset = 2;\necho strpos(\"abcabc\", \"c\", $offsetRef);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern int64_t phpc_native_value_to_int_with_reference_slot_with_diagnostic("
        ),
        "{source}"
    );
    assert!(
        body.matches(" = (long long)phpc_native_value_to_int_with_reference_slot_with_diagnostic(")
            .count()
            >= 5,
        "{source}"
    );
    assert!(
        body.contains("phpc_native_value_string_int_operation_with_diagnostic(")
            && body.contains("phpc_native_value_string_search_result_with_diagnostic(")
            && body.contains("phpc_native_symbol_table_bind_reference_path(")
            && body.contains("phpc_native_symbol_table_set_value_by_path_with_diagnostic("),
        "{source}"
    );
    assert!(
        !body.contains(" = phpc_native_reference_value_clone(")
            && !source.contains("assembly string-int builtin lowering rejects")
            && !source.contains("reference assignment lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_extended_reference_string_result_slots_through_shared_boundary(
) {
    let program = parse(
        "<?php\n$offset = 1;\n$length = 2;\n$offsetRef =& $offset;\n$lengthRef =& $length;\necho strncmp(\"abcdef\", \"abcxyz\", $lengthRef);\necho substr_count(\"abcabc\", \"a\", $offsetRef, $lengthRef);\n$length = 4;\necho strncasecmp(\"ABCDEF\", \"abcdxy\", $lengthRef);\n$offset = 3;\necho strpos(\"abcabc\", \"a\", $offsetRef);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains(
            " = (long long)phpc_native_value_to_int_with_reference_slot_with_diagnostic("
        ) && body.contains("phpc_native_value_string_int_operation_with_diagnostic(")
            && body.contains("phpc_native_value_string_search_result_with_diagnostic("),
        "{source}"
    );
    assert!(
        body.matches(" = (long long)phpc_native_value_to_int_with_reference_slot_with_diagnostic(")
            .count()
            >= 5,
        "{source}"
    );
    assert!(
        !body.contains(" = phpc_native_reference_value_clone(")
            && !source.contains("assembly string-result builtin lowering rejects")
            && !source.contains("assembly string-int builtin lowering rejects"),
        "{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_native_reference_key_text_membership_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "native_reference_key_text_membership",
        "<?php\n$name = \"MYSQLI_QUERY\";\n$nameRef =& $name;\n$extension = \"Json\";\n$extensionRef =& $extension;\n$key = \"A\\0B\";\n$keyRef =& $key;\n$numeric = \"42\";\n$numericRef =& $numeric;\n$items = [$keyRef => \"value\", $numericRef => \"number\"];\necho function_exists($nameRef), extension_loaded($extensionRef), \"|\", $items[$key], \"|\", $items[42], \"\\n\";\n",
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native reference-key executable: {error}"));

    assert!(
        run.status.success(),
        "native executable failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"11|value|number\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn native_executable_c_source_routes_declared_instanceof_through_runtime_abi() {
    let program = parse(NATIVE_DECLARED_CLASS_INSTANCEOF_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_instanceof_class_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_value_instanceof_class_with_diagnostic")
            .count()
            >= 5,
        "named instanceof expressions should share the object-class relation ABI:\n{source}"
    );
    assert!(!source.contains("instanceof lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_routes_declared_methods_through_frame_dispatch() {
    let program = parse(NATIVE_DECLARED_CLASS_METHOD_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_declared_method_")
            && source.contains("phpc_NativeValueHandle phpc_this"),
        "{source}"
    );
    assert!(
        body.contains("phpc_native_value_instanceof_class_with_diagnostic"),
        "receiver class checks should use the shared object/class ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_object_method_failure_with_diagnostic"),
        "method misses should use the shared runtime failure ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_object_property_mutation_operation_with_diagnostic")
            && source
                .contains("phpc_native_value_object_public_property_operation_with_diagnostic"),
        "$this property writes and reads should stay on the shared property ABIs:\n{source}"
    );
    assert!(!source.contains("method-call lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_routes_this_property_assignments_through_method_frame_contract() {
    let program = parse(
        "<?php\nclass Box { public $name; public $alt; public function literal($value) { $this->name = strtoupper($value); return $this->name; } public function dynamic($property, $value) { $this->$property = $value; return $value; } }\n$box = new Box();\necho $box->literal(\"ada\"), $box->dynamic(\"alt\", 7), $box->alt;\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_NativeValueHandle phpc_this")
            && source.contains("phpc_native_value_clone(phpc_this)"),
        "instance method frames must own a cloned $this receiver:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_object_property_mutation_operation_with_diagnostic"),
        "$this writes should use the shared object-property mutation ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_NativeDiagnosticHandle object_property_mutation_diagnostic_")
            && source.contains("phpc_native_diagnostic_message_stderr(object_property_mutation_diagnostic_")
            && source.contains("phpc_native_diagnostic_free(object_property_mutation_diagnostic_"),
        "mutation diagnostics and failure cleanup must be emitted at the shared boundary:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_free(frame_this_")
            && source.contains("phpc_native_value_free(object_property_write_value_"),
        "receiver and replacement/result handles must be released by generated method frames:\n{source}"
    );
    assert!(
        source.contains("phpc_declared_method_")
            && body.contains("receiver_method_source_call_args_")
            && body.contains(
                "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
            ),
        "caller-side method calls should use source-call carriers while generated method frames still own $this assignment:\n{source}"
    );
    assert!(
        !source.contains("non-local assignment lowering rejects")
            && !source.contains("object-property lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_declared_static_methods_through_frame_dispatch() {
    let program = parse(NATIVE_DECLARED_CLASS_STATIC_METHOD_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    let static_text_frame = source
        .lines()
        .find(|line| line.contains("phpc_declared_method_") && line.contains("_label_text("))
        .unwrap_or_else(|| panic!("missing generated static method frame:\n{source}"));
    assert!(
        !static_text_frame.contains("phpc_this"),
        "static method frames must not bind $this:\n{static_text_frame}\n{source}"
    );
    assert!(
        body.contains("static_method_status")
            && body.contains("phpc_declared_method_")
            && !body.contains("phpc_native_value_instanceof_class_with_diagnostic"),
        "named static calls should dispatch directly through declared static frames without receiver class checks:\n{source}"
    );
    assert!(!source.contains("method-call lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_invokes_dynamic_instance_methods_through_runtime_name_dispatch() {
    let program = parse(NATIVE_DECLARED_CLASS_DYNAMIC_METHOD_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("dynamic_method_dispatch_status")
            && body.contains("phpc_native_value_dynamic_method_name_matches")
            && body.contains("phpc_native_value_instanceof_class_with_diagnostic")
            && body.contains("phpc_declared_method_")
            && source.contains("phpc_NativeValueHandle phpc_this"),
        "dynamic instance calls should compare runtime method names, verify receiver classes, and call declared instance frames:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_object_dynamic_method_failure_with_diagnostic"),
        "dynamic method misses should use the shared object-method diagnostic ABI:\n{source}"
    );
    assert!(!source.contains("method-call lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_routes_scalar_dynamic_method_names_through_runtime_lookup_carrier() {
    let program = parse(NATIVE_DECLARED_CLASS_DYNAMIC_METHOD_LOOKUP_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("dynamic_receiver_method_source_call_args_")
            && body.contains(
                "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
            )
            && !body.contains("dynamic_method_dispatch_status")
            && !body.contains("phpc_native_value_dynamic_method_name_matches")
            && !body.contains("phpc_native_value_dynamic_call_name_matches"),
        "scalar dynamic instance method names should enter the shared runtime lookup carrier instead of generated-name comparison ladders:\n{source}"
    );
    assert!(!source.contains("method-call lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_routes_known_dynamic_method_names_through_source_call_carriers() {
    let program = parse(NATIVE_DECLARED_DYNAMIC_METHOD_SOURCE_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("dynamic_receiver_method_source_call_args_")
            && body.contains(
                "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
            )
            && body.contains("PHPC_NATIVE_CALLABLE_ACCESS_OBJECT_RECEIVER"),
        "known dynamic receiver-method names should use shared source-call target/carrier helpers:\n{source}"
    );
    assert!(
        body.matches("phpc_native_call_arguments_push_reference_and_free")
            .count()
            >= 2,
        "dynamic receiver-method source calls should preserve known by-reference binding through the shared argument handle:\n{source}"
    );
    assert!(
        !body.contains("dynamic_method_dispatch_status")
            && !body.contains("phpc_native_value_dynamic_method_name_matches"),
        "known dynamic source-call route must not use generated-name comparison ladders:\n{source}"
    );
    assert!(!source.contains("method-call lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_routes_dynamic_method_name_misses_through_source_call_diagnostics() {
    let program = parse(NATIVE_DECLARED_DYNAMIC_METHOD_SOURCE_CALL_FAILURE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("dynamic_receiver_method_source_call_args_")
            && body.contains(
                "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
            )
            && body.contains("phpc_native_diagnostic_report"),
        "dynamic receiver-method source-call misses should stay on lookup-plus-invoke diagnostics:\n{source}"
    );
    assert!(
        !body.contains("dynamic_method_dispatch_status")
            && !body.contains("phpc_native_value_dynamic_method_name_matches"),
        "dynamic source-call miss proof must not use generated-name comparison ladders:\n{source}"
    );
    assert!(!source.contains("method-call lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_routes_dynamic_method_class_context_calls_through_source_call_carriers(
) {
    let program = parse(NATIVE_DECLARED_DYNAMIC_METHOD_CLASS_CONTEXT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("PHPC_NATIVE_CALLABLE_VISIBILITY_PRIVATE")
            && source.contains("dynamic_receiver_method_source_call_args_")
            && source.contains(
                "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
            )
            && source.contains("PHPC_NATIVE_CALLABLE_ACCESS_CLASS_CONTEXT")
            && source.contains("method_call_caller_scope_"),
        "dynamic receiver-method source calls inside declared methods should preserve class-context access:\n{source}"
    );
    assert!(
        !source.contains("dynamic_method_dispatch_status")
            && !source.contains("phpc_native_value_dynamic_method_name_matches")
            && !source.contains("method-call lowering rejects"),
        "class-context dynamic source calls should not fall back to generated-name comparison ladders:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_unknown_runtime_dynamic_method_names_through_source_call_carriers(
) {
    let program = parse(NATIVE_DECLARED_RUNTIME_DYNAMIC_METHOD_SOURCE_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("dynamic_receiver_method_source_call_args_")
            && body.contains(
                "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
            )
            && body.contains("PHPC_NATIVE_CALLABLE_ACCESS_OBJECT_RECEIVER"),
        "runtime dynamic receiver-method names should use shared source-call target/carrier helpers:\n{source}"
    );
    assert!(
        body.matches("phpc_native_call_arguments_push_reference_and_free")
            .count()
            >= 2,
        "runtime dynamic receiver-method source calls should preserve supported by-reference arguments through the shared handle:\n{source}"
    );
    assert!(
        !body.contains("dynamic_method_dispatch_status")
            && !body.contains("phpc_native_value_dynamic_method_name_matches"),
        "runtime dynamic source-call route must not use generated-name comparison ladders:\n{source}"
    );
    assert!(!source.contains("method-call lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_routes_magic_dynamic_methods_through_runtime_dispatch_boundary() {
    let program = parse(NATIVE_DECLARED_DYNAMIC_METHOD_MAGIC_BLOCKED_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("dynamic_receiver_method_source_call_args_")
            && body.contains(
                "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
            )
            && body.contains("PHPC_NATIVE_CALLABLE_ACCESS_OBJECT_RECEIVER")
            && !body.contains("dynamic_method_dispatch_status")
            && !body.contains("phpc_native_value_dynamic_method_name_matches"),
        "declared __call receiver calls should use the shared runtime method dispatch boundary instead of generated-name comparison ladders:\n{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_CALLABLE_ACCESS_CLASS_CONTEXT")
            && source.contains("method_call_caller_scope_"),
        "declared __call classes should preserve class-context visibility for dynamic receiver calls inside methods:\n{source}"
    );
    assert!(!source.contains("method-call lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_invokes_object_static_methods_through_source_call_carrier() {
    let program = parse(NATIVE_DECLARED_OBJECT_STATIC_METHOD_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("phpc_native_static_method_scope_from_receiver_with_diagnostic_and_free")
            && body.contains(
                "phpc_native_static_method_invoke_value_with_access_context_diagnostic_and_free_scope_method_arguments"
            )
            && body.contains("object_static_method_source_call_args_")
            && body.contains("PHPC_NATIVE_CALLABLE_ACCESS_STATIC"),
        "object static-receiver calls should derive runtime receiver scope and invoke through shared static source-call carriers:\n{source}"
    );
    assert!(
        !body.contains("object_static_method_status")
            && !body.contains("phpc_native_value_instanceof_class_with_diagnostic"),
        "object static source-call production should not fall back to the generated class-check frame ladder:\n{source}"
    );
    assert!(!source.contains("method-call lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_routes_declared_object_static_default_variadic_calls_through_source_call_carriers(
) {
    let program = parse(NATIVE_DECLARED_OBJECT_STATIC_DEFAULT_VARIADIC_SOURCE_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("phpc_native_static_method_scope_from_receiver_with_diagnostic_and_free")
            && body.contains(
                "phpc_native_static_method_invoke_value_with_access_context_diagnostic_and_free_scope_method_arguments"
            )
            && body.matches("object_static_method_source_call_args_").count() >= 2
            && body.matches("phpc_native_call_arguments_push_value_and_free").count() >= 5,
        "object static default/variadic calls should build shared owned argument handles before carrier invocation:\n{source}"
    );
    assert!(
        body.contains("phpc_native_array_empty")
            && body.contains("phpc_native_array_append_value_with_diagnostic")
            && body.contains("phpc_native_value_from_array"),
        "object static variadic source calls should pack surplus call-site arguments before carrier invocation:\n{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_CALLABLE_ACCESS_STATIC")
            && !body.contains("object_static_method_status"),
        "object static default/variadic source-call production should not fall back to generated frame-dispatch ladders:\n{source}"
    );
    assert!(!source.contains("method-call lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_routes_declared_method_static_calls_through_source_call_carriers() {
    let program = parse(NATIVE_DECLARED_METHOD_STATIC_SOURCE_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains(
            "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
        ) && body.contains(
            "phpc_native_static_method_invoke_value_with_access_context_diagnostic_and_free_scope_method_arguments"
        ),
        "declared exact receiver/static calls should use shared method/static source-call carriers:\n{source}"
    );
    assert!(
        body.matches("phpc_native_call_arguments_push_value_and_free")
            .count()
            >= 4,
        "source-call method/static production should build arguments through the shared owned argument handle:\n{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_CALLABLE_ACCESS_OBJECT_RECEIVER")
            && body.contains("PHPC_NATIVE_CALLABLE_ACCESS_STATIC")
            && !body.contains("method_dispatch_status")
            && !body.contains("static_method_status"),
        "the exact source-call fixture should not fall back to generated frame-dispatch ladders:\n{source}"
    );
    assert!(!source.contains("method-call lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_routes_declared_method_static_default_variadic_calls_through_source_call_carriers(
) {
    let program = parse(NATIVE_DECLARED_METHOD_STATIC_DEFAULT_VARIADIC_SOURCE_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains(
            "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
        ) && body.contains(
            "phpc_native_static_method_invoke_value_with_access_context_diagnostic_and_free_scope_method_arguments"
        ),
        "default/variadic receiver and static calls should still use shared source-call carriers:\n{source}"
    );
    assert!(
        body.matches("receiver_method_source_call_args_").count() >= 2
            && body.matches("static_method_source_call_args_").count() >= 2
            && body.matches("phpc_native_call_arguments_push_value_and_free").count() >= 12,
        "default/variadic method/static production should build every frame slot through the shared owned argument handle:\n{source}"
    );
    assert!(
        body.contains("phpc_native_array_empty")
            && body.contains("phpc_native_array_append_value_with_diagnostic")
            && body.contains("phpc_native_value_from_array"),
        "variadic receiver/static source calls should pack surplus call-site arguments before carrier invocation:\n{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_CALLABLE_ACCESS_OBJECT_RECEIVER")
            && body.contains("PHPC_NATIVE_CALLABLE_ACCESS_STATIC")
            && !body.contains("method_dispatch_status")
            && !body.contains("static_method_status"),
        "default/variadic source-call production should not fall back to generated frame-dispatch ladders:\n{source}"
    );
    assert!(!source.contains("method-call lowering rejects"), "{source}");
}

#[test]
fn native_executable_c_source_routes_self_parent_static_calls_through_class_context_source_call_carriers(
) {
    let program = parse(NATIVE_SELF_PARENT_STATIC_SOURCE_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains(
            "phpc_native_static_method_invoke_value_with_access_context_diagnostic_and_free_scope_method_arguments"
        ) && source.contains("PHPC_NATIVE_CALLABLE_ACCESS_CLASS_CONTEXT")
            && source.contains("method_call_caller_scope_"),
        "self::/parent:: static calls should route through class-context static source-call carriers:\n{source}"
    );
    assert!(
        source.contains("phpc_native_callable_table_register_class_parent_and_free")
            && source.contains("PHPC_NATIVE_CALLABLE_VISIBILITY_PROTECTED")
            && source.contains("PHPC_NATIVE_CALLABLE_VISIBILITY_PRIVATE"),
        "inherited/protected/private static metadata should remain in the callable table for runtime access checks:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_call_arguments_push_reference_and_free")
            .count()
            >= 2
            && source
                .matches("phpc_native_call_arguments_push_value_and_free")
                .count()
                >= 5,
        "self::/parent:: source calls should preserve shared call-argument handle binding, including by-reference args:\n{source}"
    );
    assert!(
        !source.contains("static_method_status") && !source.contains("method-call lowering rejects"),
        "the source-call fixture should not fall back to generated direct static dispatch ladders:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_late_static_calls_through_called_scope_source_call_carriers() {
    let program = parse(NATIVE_LATE_STATIC_SOURCE_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_call_frame_called_scope")
            && source.contains("phpc_NativeStringHandle phpc_called_scope")
            && source.contains("late_static_method_source_call_args_"),
        "generated method frames should receive runtime called scope and use it for static:: source-call arguments:\n{source}"
    );
    assert!(
        source.contains(
            "phpc_native_static_method_invoke_value_with_access_context_diagnostic_and_free_scope_method_arguments"
        ) && source.contains("PHPC_NATIVE_CALLABLE_ACCESS_CLASS_CONTEXT")
            && source.contains("method_call_caller_scope_"),
        "static:: calls should invoke through class-context static source-call carriers instead of lexical direct dispatch:\n{source}"
    );
    assert!(
        source.contains("phpc_native_callable_table_register_class_parent_and_free")
            && source.contains("PHPC_NATIVE_CALLABLE_VISIBILITY_PROTECTED"),
        "late-static source calls should keep inherited protected metadata in the callable table for runtime access checks:\n{source}"
    );
    assert!(
        !source.contains("static_method_status") && !source.contains("method-call lowering rejects"),
        "late-static calls in the supported subset must not fall back to generated direct static dispatch ladders:\n{source}"
    );
}

#[test]
fn native_executable_c_source_keeps_descendant_only_late_static_targets_blocked() {
    let program = parse(concat!(
        "<?php\n",
        "class LateStaticDescendantOnlyBase {\n",
        "    public static function relay($value) { return static::target($value); }\n",
        "}\n",
        "class LateStaticDescendantOnlyChild extends LateStaticDescendantOnlyBase {\n",
        "    public static function target($value) { return $value; }\n",
        "}\n",
        "echo LateStaticDescendantOnlyChild::relay(\"x\");\n",
    ))
    .unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("method-call lowering rejects"),
        "descendant-only static:: targets should stay on the explicit method-call blocker, got {error:?}"
    );
}

#[test]
fn native_executable_c_source_routes_non_public_method_calls_through_class_context() {
    let program = parse(concat!(
        "<?php\n",
        "class NativeVisibilityBox {\n",
        "    public function reveal($value) {\n",
        "        return $this->secret($value) . \":\" . NativeVisibilityBox::guard($value);\n",
        "    }\n",
        "    private function secret($value) { return \"p\" . strtoupper($value); }\n",
        "    protected static function guard($value) { return \"s\" . strtolower($value); }\n",
        "}\n",
        "$box = new NativeVisibilityBox();\n",
        "echo $box->reveal(\"Go\"), \"\\n\";\n",
    ))
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("PHPC_NATIVE_CALLABLE_VISIBILITY_PRIVATE")
            && source.contains("PHPC_NATIVE_CALLABLE_VISIBILITY_PROTECTED")
            && source.contains("phpc_native_callable_table_register_visibility_staticness_frame_callback_and_free"),
        "non-public generated method frames must be registered with runtime visibility metadata:\n{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_CALLABLE_ACCESS_OBJECT_RECEIVER")
            && source.contains("PHPC_NATIVE_CALLABLE_ACCESS_CLASS_CONTEXT")
            && source.contains("phpc_native_string_free(method_call_caller_scope_"),
        "caller-side source calls should distinguish external object access from method-frame class context and clean the caller-scope carrier:\n{source}"
    );
    let method_invoke_count = source
        .matches(
            "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments",
        )
        .count();
    assert!(
        method_invoke_count >= 2
            && source.contains(
                "phpc_native_static_method_invoke_value_with_access_context_diagnostic_and_free_scope_method_arguments"
            ),
        "instance and static non-public calls should share lookup-plus-invoke carriers with owned arguments:\n{source}"
    );
    assert!(
        !body.contains("method_dispatch_status")
            && !body.contains("static_method_status")
            && !source.contains("method-call lowering rejects"),
        "non-public visibility must not bypass the runtime lookup diagnostic boundary via direct frame ladders:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_non_public_method_class_context_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "non_public_method_class_context",
        concat!(
            "<?php\n",
            "class NativeVisibilityBoxRun {\n",
            "    public function reveal($value) {\n",
            "        return $this->secret($value) . \":\" . NativeVisibilityBoxRun::guard($value);\n",
            "    }\n",
            "    private function secret($value) { return \"p\" . strtoupper($value); }\n",
            "    protected static function guard($value) { return \"s\" . strtolower($value); }\n",
            "}\n",
            "$box = new NativeVisibilityBoxRun();\n",
            "echo $box->reveal(\"Go\"), \"\\n\";\n",
        ),
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native non-public method class-context executable {}: {error}",
            output_path.display()
        )
    });

    assert!(
        run.status.success(),
        "native executable failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"pGO:sgo\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_routes_declared_constructors_through_frame_dispatch() {
    let program = parse(NATIVE_DECLARED_CLASS_CONSTRUCTOR_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_declared_method_")
            && source.contains("phpc_NativeValueHandle phpc_this"),
        "{source}"
    );
    assert!(
        body.contains("phpc_native_value_new_declared_class_with_diagnostic"),
        "constructors should still allocate through the shared object ABI:\n{source}"
    );
    assert!(
        body.contains("constructor_status") && body.contains("__construct"),
        "new expressions should dispatch supported constructors through generated frames:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_object_property_mutation_operation_with_diagnostic")
            && source.contains("phpc_native_value_object_public_property_operation_with_diagnostic"),
        "$this constructor writes and method reads should stay on the shared property ABIs:\n{source}"
    );
    assert!(
        !source.contains("object-instantiation lowering rejects")
            && !source.contains("method-call lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_dynamic_declared_constructors_through_frame_dispatch() {
    let program = parse(NATIVE_DECLARED_CLASS_DYNAMIC_CONSTRUCTOR_NEW_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_dynamic_class_name_matches"),
        "dynamic class-name new should match generated declared-class candidates through the shared class-name helper:\n{source}"
    );
    assert!(
        body.matches("constructor_status").count() >= 4
            && source.contains("phpc_NativeValueHandle phpc_this"),
        "dynamic constructor new should dispatch matched public constructors through generated frames with $this:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_new_declared_class_with_diagnostic")
            && (body.contains("phpc_native_value_new_declared_class_with_ancestors_and_diagnostic")
                || body.contains(
                    "phpc_native_value_new_declared_class_with_relationships_and_diagnostic"
                )),
        "dynamic constructor new should keep declared allocation on shared object helpers, including inherited/relationship metadata receivers:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_object_property_mutation_operation_with_diagnostic")
            && source.contains("phpc_native_value_object_public_property_operation_with_diagnostic")
            && source.contains("phpc_native_value_type_name_result")
            && source.contains("phpc_native_value_instanceof_class_with_diagnostic"),
        "dynamic constructor results should compose with property writes/reads, debug type, and ancestor checks:\n{source}"
    );
    assert!(
        !source.contains("object-instantiation lowering rejects")
            && !source.contains("object/class lowering rejects")
            && !source.contains("method-call lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_reports_constructor_value_returns_without_dropping_value() {
    let program = parse(NATIVE_DECLARED_CLASS_CONSTRUCTOR_VALUE_RETURN_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("*phpc_call_status = 2; return"),
        "constructor value-return statements should leave a distinct frame status:\n{source}"
    );
    assert!(
        body.contains("constructor_status")
            && body.contains("== 2")
            && body.contains("method_dispatch_reason_bytes"),
        "constructor callers should diagnose value returns explicitly:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_free(constructor_result")
            && body.contains("phpc_native_value_object_method_failure_with_diagnostic")
            && body.contains("phpc_native_value_free(declared_class_object"),
        "constructor value-return diagnostics must free the returned value and receiver instead of dropping through:\n{source}"
    );
    assert!(
        !source.contains("object-instantiation lowering rejects")
            && !source.contains("method-call lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_declared_inheritance_through_ancestor_metadata() {
    let program = parse(NATIVE_DECLARED_CLASS_INHERITANCE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("phpc_native_value_new_declared_class_with_relationships_and_diagnostic")
            && source.contains("declared_class_property_declaring_ids")
            && source.contains("declared_class_property_declaring_name_ptrs")
            && source.contains("declared_class_ancestor_name_ptrs"),
        "inherited declared objects should allocate through relationship-aware object metadata:\n{source}"
    );
    assert!(
        body.matches("phpc_native_value_instanceof_class_with_diagnostic")
            .count()
            >= 5,
        "inherited instanceof, instance methods, and dynamic methods should share class-relation checks:\n{source}"
    );
    assert!(
        body.contains("receiver_method_source_call_args_")
            && body.contains(
                "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
            )
            && body.contains("PHPC_NATIVE_CALLABLE_ACCESS_OBJECT_RECEIVER")
            && body.contains("dynamic_method_dispatch_status")
            && body.contains("static_method_source_call_args_")
            && body.contains(
                "phpc_native_static_method_invoke_value_with_access_context_diagnostic_and_free_scope_method_arguments"
            )
            && body.contains("phpc_native_static_method_scope_from_receiver_with_diagnostic_and_free")
            && body.contains(
                "phpc_native_static_method_invoke_value_with_access_context_diagnostic_and_free_scope_method_arguments"
            )
            && body.contains("object_static_method_source_call_args_")
            && body.contains("PHPC_NATIVE_CALLABLE_ACCESS_STATIC")
            && body.contains("constructor_status"),
        "inherited public methods, dynamic methods, static methods, object static source-call carriers, and constructors should stay routed through their supported paths:\n{source}"
    );
    assert!(
        !body.contains("static_method_status") && !body.contains("object_static_method_status"),
        "inherited static and object static calls should not revive generated frame-dispatch ladders:\n{source}"
    );
    assert!(
        !source.contains("object/class lowering rejects")
            && !source.contains("object-instantiation lowering rejects")
            && !source.contains("method-call lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_keeps_unsupported_declared_class_features_blocked() {
    for source in [
        "<?php\nclass Child extends Missing {}\nnew Child();\n",
        "<?php\nfinal class Base {}\nclass Child extends Base {}\nnew Child();\n",
        "<?php\nclass Box { public object $name; }\nnew Box();\n",
    ] {
        let program = parse(source).expect("unsupported declared-class source parses");
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error.message.contains("object/class lowering rejects")
                || error
                    .message
                    .contains("object-instantiation lowering rejects"),
            "{source}\n{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_keeps_unsupported_constructor_shapes_blocked() {
    for source in [
        "<?php\nclass Box { private function __construct() {} }\nnew Box();\n",
        "<?php\nclass Box { protected function __construct() {} }\nnew Box();\n",
        "<?php\nclass Box { public static function __construct() {} }\nnew Box();\n",
        "<?php\nclass Box { public function __construct() { global $x; } }\nnew Box();\n",
        "<?php\nclass Box { private function __construct() {} }\n$class = \"Box\";\nnew $class();\n",
    ] {
        let program = parse(source).expect("unsupported constructor source parses");
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error
                .message
                .contains("object-instantiation lowering rejects")
                || error.message.contains("object/class lowering rejects"),
            "{source}\n{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_keeps_unsupported_method_shapes_blocked() {
    for source in [
        "<?php\nclass Box { public function go() { return 1; } }\nBox::go();\n",
        "<?php\nclass Box { public function go() { return 1; } }\n$box = new Box();\necho $box::go();\n",
        "<?php\nclass Box { private function go() { return 1; } }\n$box = new Box();\necho $box->go();\n",
        "<?php\nclass Box { public function go() { return 1; } public function GO() { return 2; } }\nnew Box();\n",
    ] {
        let program = parse(source).expect("unsupported method source parses");
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error.message.contains("method-call lowering rejects")
                || error.message.contains("object/class lowering rejects"),
            "{source}\n{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_blocks_declared_static_method_bodies_that_require_this() {
    let source =
        "<?php\nclass Box { public static function go() { return $this->x; } }\nBox::go();\n";
    let program = parse(source).expect("unsupported static method $this source parses");
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("variable-read lowering rejects"),
        "{source}\n{error:?}"
    );
}

#[test]
fn native_executable_c_source_keeps_unsupported_object_property_shapes_blocked() {
    for source in [
        "<?php\nclass Box { public $name; }\n$box = new Box();\n$prop = \"name\";\necho $box->$prop;\n",
        "<?php\nclass Box { public $name; }\n$box = new Box();\n$prop = \"name\";\nunset($box->$prop);\n",
        "<?php\nclass Box { public $items; }\n$box = new Box();\n$box->items[\"x\"] = 1;\n",
        "<?php\nclass Box { public $items; }\n$box = new Box();\nunset($box->items[\"x\"]);\n",
        "<?php\nclass Box { public $name; }\necho Box::$name;\n",
    ] {
        let program = parse(source).expect("unsupported object-property source parses");
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error.message.contains("object-property lowering rejects")
                || error.message.contains("ArrayAccess lowering rejects")
                || error.message.contains("static-member lowering rejects")
                || error.message.contains("mutation lowering rejects"),
            "{source}\n{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_merges_cleanup_free_if_branch_state() {
    let program = parse(NATIVE_BRANCH_STATE_MERGE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches(" ? ").count() >= 3,
        "post-branch scalar/string variables should be represented by conditional values:\n{source}"
    );
    assert!(
        body.contains("if (native_value_truthy_") && body.contains("} else {"),
        "branch bodies should still be emitted as scoped C control flow:\n{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_joins_if_branch_native_value_owners() {
    let program = parse(NATIVE_BRANCH_NATIVE_VALUE_OWNER_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches("phpc_NativeValueHandle if_native_value_join_")
            .count()
            >= 3,
        "branch-created and branch-selected native values should transfer into post-branch owners:\n{source}"
    );
    assert!(
        body.contains("strtoupper_result_")
            && body.contains("strrev_result_")
            && body.contains("native_value_array_query_"),
        "branch producers should remain inside scoped branch bodies across string and array-query value families:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_format_stdout_with_diagnostic(if_native_value_join_")
            && body.contains("phpc_native_value_free(if_native_value_join_"),
        "joined owner handles should feed later consumers and final cleanup:\n{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_releases_branch_local_native_value_cleanup() {
    let program = parse(
        "<?php\n$flags = [\"take\" => \"1\", \"skip\" => \"0\"];\nif ($flags[\"take\"]) { array_sum([2, 3]); echo \"T\"; } else { array_product([4, 5]); echo \"E\"; }\nif ($flags[\"skip\"]) { array_sum([9]); echo \"bad\"; }\necho \"done\";\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches("phpc_native_value_free(native_value_array_query_")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_releases_branch_local_array_and_byte_buffer_cleanup() {
    let program = parse(NATIVE_BRANCH_LOCAL_NON_VALUE_OWNER_CLEANUP_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches("phpc_native_array_free(array_").count() >= 3,
        "branch-local array owners should be cleaned on branch exits:\n{source}"
    );
    assert!(
        body.matches("phpc_native_byte_buffer_free(string_offset_read_buffer_")
            .count()
            >= 3,
        "branch-local string offset buffers should be cleaned on branch exits:\n{source}"
    );
    assert!(
        body.contains(" ? (\"T\") : (\"E\")"),
        "scalar branch state should still merge while local owners are cleaned:\n{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_state_stable_while_loops_through_scoped_cleanup() {
    let program = parse(NATIVE_STATE_STABLE_WHILE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches("while (1)").count() >= 3,
        "state-stable while loops should lower to scoped C loops:\n{source}"
    );
    assert!(
        body.matches("if (!(native_value_truthy_").count() >= 3,
        "loop conditions should evaluate through the shared PHP truthiness guard:\n{source}"
    );
    assert!(
        body.matches("phpc_native_array_lvalue_owner_pointer_result")
            .count()
            >= 8,
        "direct and nested array pointer calls should feed loop conditions and bodies through the lvalue owner ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_compare_result"),
        "loop conditions should compose existing native value comparison results:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_free(native_value_array_query_"),
        "loop-body local native value results must be released before the next iteration:\n{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_rejects_while_state_without_loop_join() {
    for source in [
        "<?php\n$items = [\"go\" => \"1\"];\nwhile ($items[\"go\"]) { $value = \"loop\"; }\necho $value;\n",
        "<?php\n$items = [\"go\" => \"1\"];\nwhile ($items[\"go\"]) { $value = strtoupper(\"loop\"); }\necho $value;\n",
        "<?php\nwhile (true) { echo \"forever\"; }\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error
                .message
                .contains("while/for loops outside state-stable condition/body/increment cleanup boundaries"),
            "{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_while_loop_transfers() {
    let program = parse(NATIVE_WHILE_LOOP_TRANSFER_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("continue;"),
        "loop-local continue should lower inside generated C while body:\n{source}"
    );
    assert!(
        body.matches("break;").count() >= 3,
        "loop-local break should lower inside generated C while body:\n{source}"
    );
    assert!(
        body.matches("phpc_native_value_free(native_value_array_query_")
            .count()
            >= 2,
        "loop transfer branches should release discarded native values before transfer:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_compare_result"),
        "loop transfer guards should compose existing native value comparison results:\n{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_rejects_unsupported_loop_transfer_depths() {
    for source in [
        "<?php\nbreak;\n",
        "<?php\ncontinue;\n",
        "<?php\n$items = [\"x\"];\nwhile (current($items) !== false) { break 2; }\n",
        "<?php\n$items = [\"x\"];\nwhile (current($items) !== false) { continue 2; }\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error.message.contains(
                "while/for loops outside state-stable condition/body/increment cleanup boundaries"
            ),
            "{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_multi_level_loop_transfers() {
    let program = parse(NATIVE_MULTI_LEVEL_LOOP_TRANSFER_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("goto while_continue_") && body.contains("goto while_break_"),
        "nested while continue/break depths should target enclosing loop labels:\n{source}"
    );
    assert!(
        body.contains("goto for_continue_") && body.contains("goto for_break_"),
        "nested for continue/break depths should target enclosing loop labels:\n{source}"
    );
    assert!(
        body.contains("while_continue_")
            && body.contains("while_break_")
            && body.contains("for_continue_")
            && body.contains("for_break_"),
        "all generated multi-level transfer targets should be emitted:\n{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_state_stable_switch_dispatch() {
    let program = parse(NATIVE_SWITCH_DISPATCH_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_compare_result"),
        "switch case matching should use the shared PHP value comparison result ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_value_compare_result").count() >= 5,
        "switch subjects and case labels should compare through one reusable runtime path:\n{source}"
    );
    assert!(
        body.contains("switch_case_") && body.contains("switch_break_"),
        "switch lowering should emit explicit fallthrough and break labels:\n{source}"
    );
    assert!(
        body.contains("goto switch_case_") && body.contains("goto switch_break_"),
        "matched cases and PHP break should route through generated labels:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_free(native_value_array_query_"),
        "case-body discarded native values should be released before switch transfer:\n{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_rejects_switch_state_joins_and_continue() {
    for source in [
        "<?php\n$value = \"base\";\nswitch (1) { case 1: $value = \"changed\"; break; }\necho $value;\n",
        "<?php\nswitch (1) { case 1: continue; }\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error.message.contains(
                "switch statements outside state-stable condition/case-body cleanup boundaries"
            ),
            "{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_state_stable_goto_labels() {
    let program = parse(NATIVE_STATE_STABLE_GOTO_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("goto goto_label_") && body.contains("goto_label_"),
        "top-level goto statements and labels should emit C goto targets:\n{source}"
    );
    assert!(
        body.matches("goto goto_label_").count() >= 3,
        "multiple source-level goto transfers should route through generated labels:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_free(native_value_array_query_"),
        "skipped state-stable statements may still contain scoped native cleanup:\n{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_rejects_goto_target_state_joins_and_nested_gotos() {
    for source in [
        "<?php\ngoto done;\n$value = \"changed\";\ndone:\necho $value;\n",
        "<?php\ndone:\n$value = \"changed\";\ngoto done;\n",
        "<?php\nif (true) { goto done; }\ndone:\necho \"after\";\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error.message.contains("goto labels outside top-level state-stable target snapshots")
                || error.message.contains(
                    "if/else branch state merges outside cleanup-free scalar/string/bool variable values"
                ),
            "{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_try_finally_normal_flow() {
    let program = parse(NATIVE_TRY_FINALLY_NORMAL_FLOW_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        !body.contains("catch"),
        "catch bodies should not be emitted for the bounded no-throw native path:\n{source}"
    );
    assert!(
        body.matches("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free")
            .count()
            >= 4,
        "try body, finally body, and following statements should all emit output report sinks:\n{source}"
    );
    assert!(
        body.contains("stmt_diagnostic_result_")
            && body.contains("phpc_native_diagnostic_result_report_stderr_list_and_free"),
        "try-body discarded native values should be consumed by the diagnostics-only statement sink before finally/fallthrough:\n{source}"
    );
    assert!(
        !source.contains("try/catch/finally lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_keeps_top_level_try_return_cleanup_blocked() {
    let program = parse(NATIVE_TRY_FINALLY_RETURN_SOURCE).unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert!(
        error
            .message
            .contains("try blocks outside the bounded generated-C normal-flow subset"),
        "{error:?}"
    );
}

#[test]
fn native_executable_c_source_runs_finally_inside_user_function_frames() {
    let program = parse(NATIVE_FUNCTION_TRY_FINALLY_FRAME_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("static phpc_NativeValueHandle phpc_user_function_0_finish(")
            && source.contains("static phpc_NativeValueHandle phpc_user_function_1_fallthrough(")
            && source
                .contains("static phpc_NativeValueHandle phpc_user_function_2_nested_finally("),
        "try/finally functions should still lower to reusable frame entries:\n{source}"
    );
    let stdout_emit_count = source
        .matches("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free")
        .count();
    assert!(
        stdout_emit_count >= 6,
        "try bodies and active finally bodies inside frames should emit through shared stdout paths:\n{source}"
    );
    assert!(
        source.contains("cleanup_diagnostic_result_")
            && source.contains("phpc_native_diagnostic_result_terminal_kind_transfer_cleanup_and_free(1")
            && source.contains("diagnostic_result_operands_"),
        "return-through-finally inside frames should queue finalizer cleanup operands before handoff:\n{source}"
    );
    assert!(
        source.contains("*phpc_call_status = 1; return"),
        "return-through-finally inside frames should still use the owned frame return handoff:\n{source}"
    );
    assert!(
        !source.contains("try/catch/finally lowering rejects")
            && !source.contains("assembly user-function lowering rejects"),
        "bounded function-local try/finally should not hit try or frame blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_function_and_method_returns_through_terminal_kind_handoff() {
    let program = parse(NATIVE_RETURN_TERMINAL_KIND_HANDOFF_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source
            .matches("phpc_native_diagnostic_result_terminal_kind_transfer_cleanup_and_free(1")
            .count()
            >= 2,
        "function and method returns should transfer return terminal kind through the diagnostic-result ABI:\n{source}"
    );
    let return_transfer_cleanup_count = source
        .lines()
        .filter(|line| {
            line.contains("phpc_native_diagnostic_result_terminal_kind_transfer_cleanup_and_free(1")
                && line.contains("diagnostic_result_operands_")
                && !line.contains("NULL, 0")
        })
        .count();
    assert!(
        return_transfer_cleanup_count >= 2,
        "function and method returns through finally should pass cleanup-frame operands into terminal transfer:\n{source}"
    );
    assert!(
        source.contains("cleanup_diagnostic_result_")
            && source.contains("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free"),
        "finally output should execute through output sinks and queue cleanup-surface operands:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_diagnostic_result_return_take_value_and_free")
            .count()
            >= 2,
        "function and method returns should hand the transferred return value back to caller frames:\n{source}"
    );
    assert!(
        source.contains("return phpc_native_call_result_from_value(phpc_call_result);"),
        "callable wrappers should preserve the existing call-result frame handoff after terminal return extraction:\n{source}"
    );
    assert!(
        !source.contains("try/catch/finally lowering rejects")
            && !source.contains("assembly user-function lowering rejects")
            && !source.contains("assembly method-call lowering rejects"),
        "bounded return terminal-kind handoff should not widen unsupported frame shapes:\n{source}"
    );
}

#[test]
fn native_executable_c_source_preserves_finally_cleanup_diagnostics_during_return_transfer() {
    let program = parse(NATIVE_TRY_FINALLY_RETURN_CLEANUP_DIAGNOSTIC_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    let return_transfer_cleanup_count = source
        .lines()
        .filter(|line| {
            line.contains("phpc_native_diagnostic_result_terminal_kind_transfer_cleanup_and_free(1")
                && line.contains("diagnostic_result_operands_")
                && !line.contains("NULL, 0")
        })
        .count();
    assert!(
        return_transfer_cleanup_count >= 2,
        "function and method return-through-finally should transfer queued cleanup operands:\n{source}"
    );
    assert!(
        source.contains("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free")
            && source.contains("cleanup_diagnostic_result_")
            && source.contains("phpc_native_diagnostic_result_return_take_value_and_free"),
        "finally output should keep stdout/diagnostic execution, cleanup-frame capture, and return value handoff:\n{source}"
    );
    assert!(
        !source.contains("try/catch/finally lowering rejects")
            && !source.contains("assembly user-function lowering rejects")
            && !source.contains("assembly method-call lowering rejects"),
        "bounded function/method return-through-finally cleanup should stay inside supported frame shapes:\n{source}"
    );
}

#[test]
fn native_executable_c_source_runs_finally_before_loop_transfers() {
    let program = parse(NATIVE_TRY_FINALLY_LOOP_TRANSFER_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("continue;") && body.contains("break;"),
        "loop transfers should still lower to their selected C transfer targets:\n{source}"
    );
    assert!(
        body.matches("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free")
            .count()
            >= 6,
        "try bodies, finally bodies, and post-loop output should all emit through stdout report sinks:\n{source}"
    );
    assert!(
        !source.contains("try/catch/finally lowering rejects")
            && !source.contains("assembly control-flow lowering rejects"),
        "break/continue through active finally should not hit try/control blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_distinguishes_exiting_and_inner_loop_transfers() {
    for source_text in [
        NATIVE_TRY_FINALLY_NESTED_LOOP_TRANSFER_SOURCE,
        NATIVE_TRY_FINALLY_INNER_LOOP_TRANSFER_SOURCE,
    ] {
        let program = parse(source_text).unwrap();
        let source = emit_native_executable_c_source(&program).unwrap();
        let body = main_body(&source);

        assert!(
            body.matches("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free")
                .count()
                >= 2,
            "nested/inner transfer programs should preserve finally output paths:\n{source}"
        );
        assert!(
            !source.contains("try/catch/finally lowering rejects")
                && !source.contains("assembly control-flow lowering rejects"),
            "transfer finalizer depth should be accepted for supported loop targets:\n{source}"
        );
    }
}

#[test]
fn native_executable_c_source_rejects_try_unwind_transfers() {
    for source in [
        "<?php\ntry { return; } catch (Exception $e) { echo \"catch\"; }\n",
        "<?php\ntry { echo \"try\"; } finally { return; }\n",
        "<?php\n$e = \"boom\";\ntry { throw $e; } catch (Exception $caught) { echo \"catch\"; } finally { echo \"finally\"; }\n",
        "<?php\ntry { goto done; } finally { echo \"finally\"; }\ndone:\necho \"after\";\n",
        "<?php\n$i = 0;\nwhile ($i < 1) { try { echo \"try\"; } finally { break; } }\n",
        "<?php\n$i = 0;\nwhile ($i < 1) { try { echo \"try\"; } finally { continue; } }\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error
                .message
                .contains("try blocks outside the bounded generated-C normal-flow subset"),
            "{source}\n{error:?}"
        );
    }
    for source in [
        "<?php\ntry { return exit(\"bye\"); } finally { echo \"finally\"; }\n",
        "<?php\ntry { exit(\"bye\"); } finally { echo \"finally\"; }\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error
                .message
                .contains("try blocks outside the bounded generated-C normal-flow subset")
                || error
                    .message
                    .contains("assembly function-call lowering rejects function calls"),
            "{source}\n{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_rejects_unsupported_function_try_unwind_transfers() {
    for source in [
        "<?php\nfunction bad() { try { exit(\"bye\"); } finally { echo \"finally\"; } }\necho bad();\n",
        "<?php\nfunction bad() { try { return \"ok\"; } finally { return \"override\"; } }\necho bad();\n",
        "<?php\nfunction bad() { $i = 0; while ($i < 1) { try { echo \"try\"; } finally { break; } } }\necho bad();\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error.message.contains("bounded generated-C frame subset"),
            "{source}\n{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_state_stable_for_loops() {
    let program = parse(NATIVE_STATE_STABLE_FOR_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches("while (1)").count() >= 3,
        "state-stable for loops should lower to scoped C loops:\n{source}"
    );
    assert!(
        body.contains("goto for_continue_") && body.contains("for_continue_"),
        "for-loop continue should run header increments through a generated label:\n{source}"
    );
    assert!(
        body.matches("if (!(native_value_truthy_").count() >= 3,
        "for-loop conditions should evaluate through the shared PHP truthiness guard:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_free(native_value_array_query_"),
        "for-loop body-local native value results must be released before increment/next iteration:\n{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_rejects_for_state_without_loop_join() {
    for source in [
        "<?php\nfor (;;) { echo \"forever\"; }\n",
        "<?php\n$items = [\"go\" => \"1\"];\nfor (; $items[\"go\"]; ) { $value = \"loop\"; }\necho $value;\n",
        "<?php\n$items = [\"go\" => \"1\"];\nfor (; $items[\"go\"]; ) { $value = strtoupper(\"loop\"); }\necho $value;\n",
        "<?php\n$items = [\"x\"];\nfor (; current($items) !== false, current($items) !== false; next($items)) { echo current($items); }\n",
        "<?php\n$value = 1;\nwhile ($value < 3) { $value = \"changed\"; }\necho $value;\n",
        "<?php\n$value = 0;\nwhile ($value < 1) { $value = strtoupper(\"native\"); }\necho $value;\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error
                .message
                .contains("while/for loops outside state-stable condition/body/increment cleanup boundaries"),
            "{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_state_stable_do_while_loops() {
    let program = parse(NATIVE_STATE_STABLE_DO_WHILE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches("while (1)").count() >= 4,
        "state-stable do-while loops should lower to scoped C loops:\n{source}"
    );
    assert!(
        body.contains("goto do_while_continue_") && body.contains("do_while_continue_"),
        "do-while continue should target the trailing condition:\n{source}"
    );
    assert!(
        body.contains("do_while_break_"),
        "do-while break targets should be emitted for nested transfer routing:\n{source}"
    );
    assert!(
        body.contains("long long loop_phi_")
            && body.contains("_Bool loop_phi_")
            && body.contains("double loop_phi_"),
        "do-while loop-carried scalar state should use mutable scalar slots:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_free(native_value_array_query_"),
        "do-while body-local native value results must be released before the condition:\n{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_rejects_do_while_state_without_loop_join() {
    for source in [
        "<?php\n$items = [\"go\" => \"1\"];\ndo { $value = \"loop\"; } while ($items[\"go\"]); echo $value;\n",
        "<?php\n$items = [\"go\" => \"1\"];\ndo { $value = strtoupper(\"loop\"); } while ($items[\"go\"]); echo $value;\n",
        "<?php\ndo { echo \"forever\"; } while (true);\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error
                .message
                .contains("do-while loops outside state-stable body/condition cleanup boundaries"),
            "{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_loop_carried_scalar_state() {
    let program = parse(NATIVE_LOOP_CARRIED_SCALAR_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("long long loop_phi_"),
        "integer loop-carried state should use mutable scalar storage:\n{source}"
    );
    assert!(
        body.contains("_Bool loop_phi_"),
        "boolean loop-carried state should use mutable scalar storage:\n{source}"
    );
    assert!(
        body.matches("while (1)").count() >= 3,
        "while and for loop phis should still lower through scoped C loops:\n{source}"
    );
    assert!(
        (body.contains(" = (loop_phi_") || body.contains(" = ((loop_phi_"))
            && body.contains("(int64_t)(loop_phi_"),
        "loop conditions and writes should read and update the same scalar slots:\n{source}"
    );
    assert!(
        !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );

    let float_program = parse(NATIVE_LOOP_CARRIED_FLOAT_SOURCE).unwrap();
    let float_source = emit_native_executable_c_source(&float_program).unwrap();
    let float_body = main_body(&float_source);
    assert!(
        float_body.contains("double loop_phi_"),
        "float loop-carried state should use mutable scalar storage:\n{float_source}"
    );
    assert!(
        float_body.contains(" = (loop_phi_") || float_body.contains(" = ((loop_phi_"),
        "float loop conditions and writes should share the same scalar slot:\n{float_source}"
    );
    assert!(
        !float_source.contains("assembly control-flow lowering rejects"),
        "{float_source}"
    );
}

#[test]
fn native_executable_c_source_routes_top_level_return_through_cleanup() {
    let program = parse(NATIVE_TOP_LEVEL_RETURN_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);
    let return_pos = body
        .find("return 0;")
        .unwrap_or_else(|| panic!("top-level return should terminate main:\n{source}"));

    assert!(
        body[..return_pos].contains("phpc_native_value_free(native_value_array_query_"),
        "discarded return-branch value results should be released before returning:\n{source}"
    );
    assert!(
        body[..return_pos].contains("phpc_native_value_free(strtoupper_result_"),
        "return operand and live native values should be cleaned before returning:\n{source}"
    );
    assert!(
        body.contains("if (native_value_truthy_") && body.contains("return 0;"),
        "return should compose with existing scoped branch lowering:\n{source}"
    );
    assert!(
        !source.contains("assembly user-function lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_rejects_branch_local_byte_buffer_state_join() {
    let program = parse(
        "<?php\n$flags = [\"take\" => \"1\"];\nif ($flags[\"take\"]) { $letter = \"abc\"[1]; } else { $letter = \"xyz\"[2]; }\necho $letter;\n",
    )
    .unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert!(
        error.message.contains("control-flow lowering rejects"),
        "{error:?}"
    );
}

#[test]
fn native_executable_c_source_discards_native_value_statement_results() {
    let program = parse(NATIVE_DISCARDED_VALUE_STATEMENT_CLEANUP_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches("phpc_native_value_free(native_value_array_query_")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        !source.contains("assembly mutation lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_native_value_ternaries_through_lazy_branches() {
    let program = parse(NATIVE_VALUE_TERNARY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches("phpc_NativeValueHandle native_value_ternary_")
            .count()
            >= 2,
        "native-value ternaries should materialize an owned result handle:\n{source}"
    );
    assert!(
        body.matches("if (native_value_truthy_").count() >= 2,
        "ternary conditions should use the shared PHP truthiness ABI:\n{source}"
    );
    assert!(
        body.contains("} else {")
            && body.contains("strtoupper_result_")
            && body.contains("strrev_result_")
            && body.contains("escapeshellarg_result_"),
        "branch value producers should live inside generated C branches:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_format_stdout_with_diagnostic(native_value_ternary_"),
        "direct ternary output should consume the shared owned result handle:\n{source}"
    );
    assert!(
        !source.contains("assembly conditional lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_native_value_short_ternaries_through_lazy_owner_transfer() {
    let program = parse(NATIVE_VALUE_SHORT_TERNARY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches("phpc_NativeValueHandle native_value_short_ternary_")
            .count()
            >= 4,
        "short ternaries should materialize selected owned result handles:\n{source}"
    );
    assert!(
        body.matches("if (native_value_truthy_").count() >= 4,
        "short ternary conditions should use the shared PHP truthiness ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_free(value_offset_read_")
            && body.contains("phpc_native_value_free(native_value_array_query_"),
        "false branches should release the unselected condition owner before fallback materialization:\n{source}"
    );
    assert!(
        body.contains("native_exit_result_")
            && body.contains("strrev_result_")
            && body.contains("strtoupper_result_"),
        "fallback producers should remain inside generated C branch bodies:\n{source}"
    );
    assert!(
        !source.contains("assembly conditional lowering rejects"),
        "{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_lazy_native_value_ternary_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("lazy_native_value_ternary", NATIVE_VALUE_TERNARY_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run lazy ternary executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"GO|ok");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_lazy_native_value_short_ternary_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "lazy_native_value_short_ternary",
        NATIVE_VALUE_SHORT_TERNARY_SOURCE,
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run lazy short ternary executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"go|ok|FALLBACK|2");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn native_executable_c_source_rejects_if_branch_state_without_cleanup_free_join() {
    for source in [
        "<?php\n$flag = 1 == \"1\";\nif ($flag) { $value = \"then\"; }\necho $value;\n",
        "<?php\n$flag = 1 == \"1\";\nif ($flag) { $value = \"then\"; } else { $value = 1; }\necho $value;\n",
        "<?php\n$flag = 1 == \"1\";\nif ($flag) { $value = [1]; } else { $value = [2]; }\necho $value[0];\n",
        "<?php\n$flag = 1 == \"1\";\nif ($flag) { $value = strtoupper(\"then\"); } else { $value = 1; }\necho $value;\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error
                .message
                .contains("if/else branch state merges outside cleanup-free scalar/string/bool variable values, owned native-value handle joins, and branch-local native-value cleanup joins"),
            "{error:?}"
        );
    }
}

#[test]
fn emit_exe_links_and_runs_array_pointer_lvalue_owner_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("array_pointer_lvalue_owner.php");
    let output_path = native_link_output_path("array_pointer_lvalue_owner");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        "<?php\n$items = [10 => \"first\", 20 => \"second\", 30 => \"third\"];\n$box = [\"nested\" => [\"n1\", \"n2\", \"n3\"]];\necho current($items), \"|\", key($items), \"|\", next($items), \"|\", key($items), \"|\", prev($items), \"|\", key($items), \"|\", end($items), \"|\", key($items), \"|\", reset($items), \"|\", key($items), \"|\", next($box[\"nested\"]), \"|\", end($box[\"nested\"]), \"|\", reset($box[\"nested\"]);\n",
    )
    .expect("write array pointer native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array pointer source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native array pointer executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native array pointer executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        run.stdout,
        b"first|10|second|20|first|10|third|30|first|10|n2|n3|n1"
    );
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_array_sort_lvalue_owner_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("array_sort_lvalue_owner.php");
    let output_path = native_link_output_path("array_sort_lvalue_owner");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        "<?php\n$values = [3, 1, 2];\n$mode = 1;\nsort($values, $mode);\nforeach ($values as $v) { echo $v; }\necho \"|\";\n$box = [\"items\" => [\"a\" => \"b2\", \"b\" => \"b10\"], \"keys\" => [\"a\" => 1, \"b\" => 2], \"natural\" => [\"z\" => \"img10\", \"y\" => \"img2\", \"x\" => \"img01\"], \"case\" => [\"up\" => \"Img12\", \"low\" => \"img2\", \"first\" => \"img1\"]];\nasort($box[\"items\"], 2);\nforeach ($box[\"items\"] as $k => $v) { echo $k, \"=\", $v, \";\"; }\necho \"|\";\nkrsort($box[\"keys\"]);\nforeach ($box[\"keys\"] as $k => $v) { echo $k, \"=\", $v, \";\"; }\necho \"|\";\nnatsort($box[\"natural\"]);\nforeach ($box[\"natural\"] as $k => $v) { echo $k, \"=\", $v, \";\"; }\necho \"|\";\nnatcasesort($box[\"case\"]);\nforeach ($box[\"case\"] as $k => $v) { echo $k, \"=\", $v, \";\"; }\necho \"|\";\necho rsort($values) ? \"T\" : \"F\";\n",
    )
    .expect("write array sort native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array sort source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native array sort executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native array sort executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        run.stdout,
        b"123|b=b10;a=b2;|b=2;a=1;|x=img01;y=img2;z=img10;|first=img1;low=img2;up=Img12;|T"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_array_mutation_lvalue_owner_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("array_mutation_lvalue_owner.php");
    let output_path = native_link_output_path("array_mutation_lvalue_owner");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        "<?php\n$items = [1, 2];\n$box = [\"items\" => [\"a\", \"b\"], \"head\" => 9];\n$push = array_push($items, $box[\"value\"] = 3, $box[\"fallback\"] ??= 4);\n$pop = array_pop($box[\"items\"]);\n$shift = array_shift($box[\"items\"]);\n$unshift = array_unshift($box[\"items\"], $box[\"head\"] += 1);\necho $push, \"|\", $items[2], \"|\", $items[3], \"|\", $pop, \"|\", $shift, \"|\", $unshift, \"|\", $box[\"items\"][0], \"|\", $box[\"value\"], \"|\", $box[\"fallback\"], \"|\", $box[\"head\"];\n",
    )
    .expect("write array mutation native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array mutation source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native array mutation executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native array mutation executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"4|3|4|b|a|1|10|3|4|10");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_array_callback_null_result_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("array_callback_null_result.php");
    let output_path = native_link_output_path("array_callback_null_result");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        "<?php\n$callback = null;\n$filtered = array_filter([\"zero\" => 0, \"name\" => \"Ada\", \"empty\" => \"\", \"word\" => \"Bee\"], $callback, \"2\");\necho $filtered[\"name\"], \"|\", $filtered[\"word\"];\n$mapped = array_map(null, [\"name\" => \"Ada\", 5 => \"five\"]);\necho \"|\", $mapped[\"name\"], \"|\", $mapped[5];\n",
    )
    .expect("write array callback native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array callback source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native array callback executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native array callback executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"Ada|Bee|Ada|five");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_array_query_family_through_shared_value_operation() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("array_query_family_value_operation.php");
    let output_path = native_link_output_path("array_query_family_value_operation");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        "<?php\n$items = [\"zero\" => 0, \"string_zero\" => \"0\", \"name\" => \"Ada\", \"empty\" => \"\"];\n$labels = [\"a\", \"b\"];\n$counted = [\"a\", \"a\", 2];\n$nums = [1, \"2\", 3];\n$fillKeys = [\"x\", 7];\n$combineKeys = [\"left\", \"right\"];\n$combineValues = [1, 2];\necho array_keys($items, \"0\", false)[0], \",\", array_keys($items, \"0\", false)[1], \"|\", in_array(\"Ada\", $items, true), \"|\", array_search(\"\", $items), \"|\", array_flip($labels)[\"a\"], array_flip($labels)[\"b\"], \"|\", array_count_values($counted)[\"a\"], \",\", array_count_values($counted)[2], \"|\", array_sum($nums), \"|\", array_product($nums), \"|\", array_fill_keys($fillKeys, \"v\")[\"x\"], \",\", array_fill_keys($fillKeys, \"v\")[7], \"|\", array_combine($combineKeys, $combineValues)[\"left\"], \",\", array_combine($combineKeys, $combineValues)[\"right\"];\n",
    )
    .expect("write array query native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array query source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native array query executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native array query executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "zero,string_zero|1|empty|01|2,1|6|6|v,v|1,2"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_array_change_key_case_through_array_query_operation() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("array_change_key_case_query_operation.php");
    let output_path = native_link_output_path("array_change_key_case_query_operation");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        "<?php\n$items = [\"Name\" => \"Ada\", \"MiXeD\" => \"mixed\", 7 => \"seven\"];\necho array_change_key_case($items)[\"name\"], \"|\", array_change_key_case($items, 1)[\"MIXED\"], \"|\", array_change_key_case($items, -1)[7];\n",
    )
    .expect("write array_change_key_case native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array_change_key_case source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native array_change_key_case executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native array_change_key_case executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "Ada|mixed|seven");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_array_column_through_operand_list_query_operation() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("array_column_operand_list_query.php");
    let output_path = native_link_output_path("array_column_operand_list_query");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        "<?php\n$rows = [];\n$rows[] = [\"id\" => \"a\", \"name\" => \"Ada\"];\n$rows[] = [\"id\" => \"b\", \"name\" => \"Bee\"];\n$rows[] = [\"name\" => \"NoId\"];\n$names = array_column($rows, \"name\", \"id\");\n$whole = array_column($rows, null);\necho $names[\"a\"], \"|\", $names[\"b\"], \"|\", $names[0], \"|\", $whole[0][\"name\"], \"|\", $whole[2][\"name\"];\n",
    )
    .expect("write array_column native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array_column source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native array_column executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native array_column executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "Ada|Bee|NoId|Ada|NoId"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_value_result_offset_read_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("value_result_offset_read.php");
    let output_path = native_link_output_path("value_result_offset_read");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        "<?php\necho array_map(null, [\"L\", \"M\"])[1];\necho \"|\";\necho ((array) \"Q\")[0];\necho \"|\";\necho ((array) \"NO\")[0][1];\necho \"|\";\necho (\"A\" . \"B\")[1];\n",
    )
    .expect("write value-result offset read native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native value-result offset read source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native value-result offset read executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native value-result offset read executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"M|Q|O|B");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_native_value_truthiness_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("native_value_truthiness.php");
    let output_path = native_link_output_path("native_value_truthiness");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(&source_path, NATIVE_VALUE_TRUTHINESS_SOURCE)
        .expect("write native value truthiness source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native value truthiness source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native value truthiness executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native value truthiness executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "TTFTFF");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_native_truthiness_boundary_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("native_truthiness_boundary", NATIVE_VALUE_TRUTHINESS_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native truthiness executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"TTFTFF");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_native_short_circuit_logical_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "native_short_circuit_logical",
        NATIVE_SHORT_CIRCUIT_LOGICAL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native short-circuit logical executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "F|T|T|T");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_scoped_if_branch_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("scoped_if_branches", NATIVE_SCOPED_IF_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native scoped-if executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"truthy|numeric|equal");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_if_branch_state_merge_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("if_branch_state_merge", NATIVE_BRANCH_STATE_MERGE_SOURCE);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native if-branch state merge executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"then|10|Hhigh");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_if_branch_native_value_owner_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "if_branch_native_value_owner",
        NATIVE_BRANCH_NATIVE_VALUE_OWNER_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native if-branch native-value owner executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"GO|keep|5");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_if_branch_local_native_value_cleanup_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "if_branch_local_native_value_cleanup",
        NATIVE_BRANCH_LOCAL_VALUE_CLEANUP_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native if-branch cleanup executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"T|done");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_if_branch_local_non_value_owner_cleanup_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "if_branch_local_non_value_owner_cleanup",
        NATIVE_BRANCH_LOCAL_NON_VALUE_OWNER_CLEANUP_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native if-branch non-value cleanup executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"b|T|done");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_state_stable_while_loop_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("state_stable_while_loop", NATIVE_STATE_STABLE_WHILE_SOURCE);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native state-stable while executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"ABCD|x=ef;y=gh;|done");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_while_loop_transfer_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("while_loop_transfer", NATIVE_WHILE_LOOP_TRANSFER_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native while-transfer executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"AB|X|done");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_multi_level_loop_transfer_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "multi_level_loop_transfer",
        NATIVE_MULTI_LEVEL_LOOP_TRANSFER_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native multi-level loop-transfer executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"01|01|done");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_state_stable_switch_dispatch_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "state_stable_switch_dispatch",
        NATIVE_SWITCH_DISPATCH_SOURCE,
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native switch-dispatch executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"B|onedefaulttwo|later|done");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_state_stable_goto_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("state_stable_goto", NATIVE_STATE_STABLE_GOTO_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native goto executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"ABCD");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_try_finally_normal_flow_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "try_finally_normal_flow",
        NATIVE_TRY_FINALLY_NORMAL_FLOW_SOURCE,
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native try/finally executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"try|BODY|finally|after");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_try_finally_return_cleanup_diagnostic_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "try_finally_return_cleanup_diagnostic",
        NATIVE_TRY_FINALLY_RETURN_CLEANUP_DIAGNOSTIC_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native try/finally return cleanup executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"fArrayGO|mArrayhi");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.matches("Array to string conversion").count() >= 2,
        "finally cleanup diagnostics should be reported during return handoff, got stderr:\n{stderr}"
    );

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_function_try_finally_frame_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "function_try_finally_frame",
        NATIVE_FUNCTION_TRY_FINALLY_FRAME_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native function try/finally executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        run.stdout,
        b"try:finally:go|body:cleanup:cba|inner:outer:done|after"
    );
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_return_terminal_kind_handoff_function_and_method_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "return_terminal_kind_handoff",
        NATIVE_RETURN_TERMINAL_KIND_HANDOFF_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run return terminal-kind handoff executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"fGO|mhi");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_try_finally_loop_transfer_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "try_finally_loop_transfer",
        NATIVE_TRY_FINALLY_LOOP_TRANSFER_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native try/finally loop-transfer executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"try0|finally1|try1|finally1|after1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_nested_try_finally_loop_transfer_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "try_finally_nested_loop_transfer",
        NATIVE_TRY_FINALLY_NESTED_LOOP_TRANSFER_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native nested try/finally loop-transfer executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"inner|outer|after");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_inner_loop_transfer_inside_try_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "try_finally_inner_loop_transfer",
        NATIVE_TRY_FINALLY_INNER_LOOP_TRANSFER_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native inner-loop try/finally executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"inside|finally");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_state_stable_for_loop_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("state_stable_for_loop", NATIVE_STATE_STABLE_FOR_SOURCE);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native state-stable for executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"ABCD|EG|X|done");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_state_stable_do_while_loop_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "state_stable_do_while_loop",
        NATIVE_STATE_STABLE_DO_WHILE_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native state-stable do-while executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"K|01!|1.52.5|B|done");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_loop_carried_scalar_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "loop_carried_scalar_state",
        NATIVE_LOOP_CARRIED_SCALAR_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native loop-carried scalar executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"01|0|124");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_top_level_return_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("top_level_return", NATIVE_TOP_LEVEL_RETURN_SOURCE);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native top-level return executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"before|");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_discarded_native_value_statement_cleanup_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "discarded_native_value_statement_cleanup",
        NATIVE_DISCARDED_VALUE_STATEMENT_CLEANUP_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run discarded native-value statement executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"AB");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_blocks_foreach_forms_without_symbol_or_reference_storage() {
    for source in [
        "<?php\n$a = [\"new\"];\nforeach ($a as $v) { $seen = $v; }\n",
        "<?php\n$a = [\"new\"];\nforeach ($a as $v) { unset($missing[0]); }\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();
        assert!(
            error.message.contains("assembly array lowering")
                || error.message.contains("assembly mutation lowering"),
            "{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_by_reference_foreach_through_reference_slots() {
    let program = parse(NATIVE_BY_REFERENCE_FOREACH_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_NativeArrayLvalueReferenceResult"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_lvalue_owner_foreach_value_reference_result"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_lvalue_reference_result_free"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_reference_set_value"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_reference_value_clone"),
        "{source}"
    );
    assert!(
        body.matches("phpc_NativeReferenceHandle array_foreach_value_reference_")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        !source.contains("native executable by-reference foreach lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_blocks_unsupported_by_reference_foreach_forms() {
    for source in [
        "<?php\nforeach ([\"a\" => 1, \"b\" => 2] as $key => &$value) { echo $key, $value; }\n",
        "<?php\n$a = [1, 2];\nforeach ($a as &$value) { $seen = $value; }\n",
        "<?php\n$a = [1, 2];\nforeach ($a as $value => &$value) { echo $value; }\n",
        "<?php\n$a = [1, 2];\nforeach ($a as &$value) { $value = $value + 1; }\necho $value;\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();
        assert!(
            error
                .message
                .contains("native executable by-reference foreach lowering rejects"),
            "{error:?}"
        );
        assert!(
            error.message.contains("temporary iterable owners")
                && error.message.contains("arbitrary body mutation")
                && error
                    .message
                    .contains("lingering post-loop reference binding"),
            "{error:?}"
        );
        assert!(
            error
                .message
                .contains("phpc_native_array_lvalue_owner_foreach_value_reference_result()"),
            "{error:?}"
        );
    }
}

#[test]
fn emit_exe_links_and_runs_by_value_foreach_array_lvalue_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("array_foreach_lvalue.php");
    let output_path = native_link_output_path("array_foreach_lvalue");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        "<?php\n$a = [\"x\" => \"ab\", \"y\" => \"cd\"];\nforeach ($a as $k => $v) { echo $k, \"=\", strtoupper($v), \";\"; }\n$b = [];\n$b[\"nested\"][\"n\"] = \"ef\";\nforeach ($b[\"nested\"] as $nk => $nv) { print $nk; print \"=\"; print strtoupper($nv); print \";\"; }\nforeach ([\"lit\" => \"gh\"] as $lk => $lv) { echo $lk, \"=\", strtoupper($lv), \";\"; }\n",
    )
    .expect("write foreach native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native foreach source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native foreach executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native foreach executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"x=AB;y=CD;n=EF;lit=GH;");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_by_reference_foreach_array_lvalue_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "foreach_by_reference_lvalue",
        NATIVE_BY_REFERENCE_FOREACH_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native by-reference foreach executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"a=ab;b=cd;|AB:CD|hi!");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_prior_foreach_cursor_storage_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "foreach_prior_cursor_storage",
        NATIVE_FOREACH_PRIOR_CURSOR_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native foreach prior-cursor executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"a=A;b=B;|b=B|R|x=R|keep");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_routes_descriptor_closures_through_shared_runtime_abi() {
    let source = concat!(
        "<?php\n",
        "function invoke_later($callback, $value) { return $callback($value); }\n",
        "$callback = function ($value) { return $value + 2; };\n",
        "echo invoke_later($callback, 3);\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_from_closure_descriptor"),
        "closure values should be materialized through the shared descriptor ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_is_descriptor_closure"),
        "dynamic callable dispatch should test runtime descriptor-closure values:\n{source}"
    );
    assert!(
        source.contains("phpc_native_closure_invoke_value_with_diagnostic"),
        "dynamic callable dispatch should invoke closure descriptors through the runtime ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_closure_frame_"),
        "generated closure bodies should lower as reusable frame callbacks:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_CLOSURE_REJECTION),
        "descriptor-ready closure frames should not hit the closure blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_by_value_closure_captures_through_descriptor_abi() {
    let source = concat!(
        "<?php\n",
        "$base = 10;\n",
        "$callback = function ($value) use ($base) { return $base + $value; };\n",
        "echo $callback(5);\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_from_closure_descriptor_captures_and_free"),
        "captured descriptor closures should use the shared capture-aware descriptor ABI:\n{source}"
    );
    assert!(
        source.contains("closure_capture_names_"),
        "capture names should be materialized as runtime metadata:\n{source}"
    );
    assert!(
        source.contains("closure_capture_values_"),
        "capture values should be materialized through value semantics:\n{source}"
    );
    assert!(
        source.contains("phpc_closure_arg_count != 2"),
        "generated closure callbacks should bind one call argument plus one capture:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_CLOSURE_REJECTION),
        "by-value captured descriptor closures should not hit the closure blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_by_reference_closure_captures_through_descriptor_abi() {
    let source = concat!(
        "<?php\n",
        "function invoke_capture($callback, $suffix) { return $callback($suffix); }\n",
        "function make_ref_capture(&$slot) { return function ($suffix) use (&$slot) { $slot = $slot . $suffix; return $slot; }; }\n",
        "class CaptureRelay { public static function apply($callback, $suffix) { return $callback($suffix); } }\n",
        "$slot = \"A\";\n",
        "$direct = function ($suffix) use (&$slot) { $slot = $slot . $suffix; return $slot; };\n",
        "echo $direct(\"0\"), \":\", $slot, \"|\";\n",
        "$slot = \"B\";\n",
        "echo invoke_capture($direct, \"1\"), \":\", $slot, \"|\";\n",
        "$outer = \"O\";\n",
        "$nested = function ($suffix) use (&$outer) { $inner = function ($tail) use (&$outer) { $outer = $outer . $tail; return $outer; }; return $inner($suffix); };\n",
        "echo CaptureRelay::apply($nested, \"2\"), \":\", $outer, \"|\";\n",
        "$target = \"T\";\n",
        "$mix = function (&$target, $value) use (&$slot) { $target = $slot . $value; $slot = $target; return $slot; };\n",
        "echo $mix($target, \"3\"), \":\", $target, \":\", $slot, \"|\";\n",
        "$factorySlot = \"F\";\n",
        "$factory = make_ref_capture($factorySlot);\n",
        "echo $factory(\"4\"), \":\", $factorySlot;\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_from_closure_descriptor_capture_arguments_and_free")
            && source.contains("closure_capture_args_"),
        "by-reference captured descriptor closures should use the shared capture-argument ABI:\n{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_CLOSURE_ARGUMENT_BY_REFERENCE")
            && source.contains("phpc_native_reference_clone(phpc_closure_args"),
        "by-reference captures should bind through native closure reference carriers:\n{source}"
    );
    assert!(
        source.contains("phpc_user_function_0_invoke_capture(")
            && source.contains("phpc_user_function_1_make_ref_capture(")
            && source.contains("phpc_declared_method_"),
        "by-reference captures should flow through direct, user-function, nested, by-reference-parameter, and static-method consumers:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_CLOSURE_REJECTION),
        "supported by-reference capture closures should not hit the closure blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_promotes_frame_local_by_reference_closure_captures() {
    let source = concat!(
        "<?php\n",
        "function make_local_capture($seed) {\n",
        "    $local = $seed;\n",
        "    $callback = function ($suffix) use (&$local) { $local = $local . $suffix; return $local; };\n",
        "    $local = $local . \"!\";\n",
        "    return $callback;\n",
        "}\n",
        "function make_param_capture($seed) {\n",
        "    $callback = function ($suffix) use (&$seed) { $seed = $seed . $suffix; return $seed; };\n",
        "    $seed = $seed . \"?\";\n",
        "    return $callback;\n",
        "}\n",
        "function make_nested_local_capture($seed) {\n",
        "    $local = $seed;\n",
        "    return function ($suffix) use (&$local) {\n",
        "        $inner = function ($tail) use (&$local) { $local = $local . $tail; return $local; };\n",
        "        return $inner($suffix);\n",
        "    };\n",
        "}\n",
        "class LocalCaptureRelay { public static function apply($callback, $suffix) { return $callback($suffix); } }\n",
        "$local = make_local_capture(\"L\");\n",
        "echo $local(\"1\"), \":\", $local(\"2\"), \"|\";\n",
        "$param = make_param_capture(\"P\");\n",
        "echo $param(\"3\"), \":\", $param(\"4\"), \"|\";\n",
        "$nested = make_nested_local_capture(\"N\");\n",
        "echo LocalCaptureRelay::apply($nested, \"5\"), \":\", $nested(\"6\");\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_reference_from_value_and_free")
            && source.contains("phpc_native_value_from_closure_descriptor_capture_arguments_and_free"),
        "frame local by-reference captures should promote through the shared reference ABI:\n{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_CLOSURE_ARGUMENT_BY_REFERENCE")
            && source.contains("phpc_native_reference_set_value"),
        "promoted captures should continue through descriptor reference carriers and assignment references:\n{source}"
    );
    assert!(
        source.contains("phpc_user_function_0_make_local_capture(")
            && source.contains("phpc_user_function_1_make_param_capture(")
            && source.contains("phpc_user_function_2_make_nested_local_capture(")
            && source.contains("phpc_declared_method_"),
        "local, parameter, nested, returned, and static-method relay shapes should share the same lowering path:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_CLOSURE_REJECTION),
        "supported frame-local by-reference capture closures should not hit the closure blocker:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_immediate_descriptor_closure_invocation() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "echo (function ($value) { return $value; })(\"A\");\n",
        "echo \"|\";\n",
        "echo (function ($left, $right) { return $left + $right; })(2, 3);\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("immediate_descriptor_closure_invocation", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A|5");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_descriptor_closure_after_by_value_frame_transfer() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class Caller {\n",
        "    public static function run($callback, $value) { return $callback($value); }\n",
        "    public static function discard($callback) { $callback(2); return 0; }\n",
        "}\n",
        "function choose($value) { return $value; }\n",
        "function invoke_later($callback, $value) { return $callback($value); }\n",
        "function relay($callback, $value) { return invoke_later($callback, $value); }\n",
        "function apply_dynamic($function, $callback, $value) { return $function($callback, $value); }\n",
        "$callback = function ($value) { return $value + 10; };\n",
        "echo invoke_later($callback, 5);\n",
        "echo \"|\";\n",
        "echo relay($callback, 7);\n",
        "echo \"|\";\n",
        "echo apply_dynamic(choose(\"invoke_later\"), $callback, 9);\n",
        "echo \"|\";\n",
        "echo Caller::run($callback, 11);\n",
        "echo \"|\";\n",
        "echo invoke_later(function ($value) { return $value * 2; }, 6);\n",
        "echo \"|\";\n",
        "$printer = function ($value) { echo $value + 20; return $value; };\n",
        "Caller::discard($printer);\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("descriptor_closure_by_value_frame_transfer", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"15|17|19|21|12|22");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_binds_descriptor_closure_by_reference_parameters() {
    let source = concat!(
        "<?php\n",
        "function apply($callback, &$slot, $value) { return $callback($slot, $value); }\n",
        "function relay($callback, &$slot) { return apply($callback, $slot, \"relay\"); }\n",
        "$callback = function (&$slot, $value) { $slot = $value; return $slot; };\n",
        "$direct = \"old\";\n",
        "echo $callback($direct, \"direct\"), \":\", $direct, \"|\";\n",
        "$frame = \"old\";\n",
        "echo apply($callback, $frame, \"frame\"), \":\", $frame, \"|\";\n",
        "$dynamic = \"old\";\n",
        "$name = \"apply\";\n",
        "echo $name($callback, $dynamic, \"dynamic\"), \":\", $dynamic, \"|\";\n",
        "$nested = \"old\";\n",
        "echo relay($callback, $nested), \":\", $nested, \"|\";\n",
        "$items = [\"a\" => \"old\", \"b\" => [\"c\" => \"deep\"]];\n",
        "$callback($items[\"a\"], \"array\");\n",
        "$callback($items[\"b\"][\"c\"], \"nested\");\n",
        "echo $items[\"a\"], \":\", $items[\"b\"][\"c\"];\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("PHPC_NATIVE_CLOSURE_ARGUMENT_BY_REFERENCE")
            && source.contains("phpc_native_closure_value_param_is_by_reference")
            && source.contains("phpc_native_closure_reference_argument_failure_with_diagnostic"),
        "closure descriptors should carry reusable by-reference parameter metadata:\n{source}"
    );
    assert!(
        source.contains("phpc_NativeClosureArgument")
            && source.contains("phpc_native_symbol_table_reference_for_path")
            && source.contains("phpc_native_reference_value_clone")
            && source.contains("phpc_native_reference_set_value"),
        "closure invocation should bind lvalue arguments through the shared reference ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_user_function_0_apply(")
            && source.contains("phpc_user_function_1_relay(")
            && source.contains("phpc_native_value_dynamic_call_name_matches"),
        "by-reference descriptor closures should flow through direct, nested, and runtime dynamic callable consumers:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_CLOSURE_REJECTION)
            && !source.contains("assembly dynamic function-call lowering rejects"),
        "supported by-reference descriptor closures should not hit closure or dynamic-call blockers:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_by_value_captured_descriptor_closure_invocation() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function invoke_later($callback, $value) { return $callback($value); }\n",
        "$base = 10;\n",
        "$callback = function ($value) use ($base) { $base = $base + $value; return $base; };\n",
        "$base = 40;\n",
        "echo $callback(5);\n",
        "echo \"|\";\n",
        "echo $callback(7);\n",
        "echo \"|\";\n",
        "echo invoke_later($callback, 3);\n",
        "echo \"|\";\n",
        "echo (function ($value) use ($base) { return $base + $value; })(2);\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("descriptor_closure_by_value_captures", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"15|17|13|42");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_by_reference_captured_descriptor_closure_invocation() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function invoke_capture($callback, $suffix) { return $callback($suffix); }\n",
        "function make_ref_capture(&$slot) { return function ($suffix) use (&$slot) { $slot = $slot . $suffix; return $slot; }; }\n",
        "class CaptureRelay { public static function apply($callback, $suffix) { return $callback($suffix); } }\n",
        "$slot = \"A\";\n",
        "$direct = function ($suffix) use (&$slot) { $slot = $slot . $suffix; return $slot; };\n",
        "echo $direct(\"0\"), \":\", $slot, \"|\";\n",
        "$slot = \"B\";\n",
        "echo invoke_capture($direct, \"1\"), \":\", $slot, \"|\";\n",
        "$outer = \"O\";\n",
        "$nested = function ($suffix) use (&$outer) { $inner = function ($tail) use (&$outer) { $outer = $outer . $tail; return $outer; }; return $inner($suffix); };\n",
        "echo CaptureRelay::apply($nested, \"2\"), \":\", $outer, \"|\";\n",
        "$target = \"T\";\n",
        "$mix = function (&$target, $value) use (&$slot) { $target = $slot . $value; $slot = $target; return $slot; };\n",
        "echo $mix($target, \"3\"), \":\", $target, \":\", $slot, \"|\";\n",
        "$factorySlot = \"F\";\n",
        "$factory = make_ref_capture($factorySlot);\n",
        "echo $factory(\"4\"), \":\", $factorySlot;\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("descriptor_closure_by_reference_captures", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native by-reference capture closure executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A0:A0|B1:B1|O2:O2|B13:B13:B13|F4:F4");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_frame_local_by_reference_captured_descriptor_closure_invocation() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function make_local_capture($seed) {\n",
        "    $local = $seed;\n",
        "    $callback = function ($suffix) use (&$local) { $local = $local . $suffix; return $local; };\n",
        "    $local = $local . \"!\";\n",
        "    return $callback;\n",
        "}\n",
        "function make_param_capture($seed) {\n",
        "    $callback = function ($suffix) use (&$seed) { $seed = $seed . $suffix; return $seed; };\n",
        "    $seed = $seed . \"?\";\n",
        "    return $callback;\n",
        "}\n",
        "function make_nested_local_capture($seed) {\n",
        "    $local = $seed;\n",
        "    return function ($suffix) use (&$local) {\n",
        "        $inner = function ($tail) use (&$local) { $local = $local . $tail; return $local; };\n",
        "        return $inner($suffix);\n",
        "    };\n",
        "}\n",
        "class LocalCaptureRelay { public static function apply($callback, $suffix) { return $callback($suffix); } }\n",
        "$local = make_local_capture(\"L\");\n",
        "echo $local(\"1\"), \":\", $local(\"2\"), \"|\";\n",
        "$param = make_param_capture(\"P\");\n",
        "echo $param(\"3\"), \":\", $param(\"4\"), \"|\";\n",
        "$nested = make_nested_local_capture(\"N\");\n",
        "echo LocalCaptureRelay::apply($nested, \"5\"), \":\", $nested(\"6\");\n",
    );
    let (source_path, output_path) = compile_native_link_fixture(
        "descriptor_closure_frame_local_by_reference_captures",
        source,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native frame-local by-reference capture closure executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"L!1:L!12|P?3:P?34|N5:N56");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_descriptor_closure_by_reference_parameter_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function apply($callback, &$slot, $value) { return $callback($slot, $value); }\n",
        "function relay($callback, &$slot) { return apply($callback, $slot, \"relay\"); }\n",
        "$callback = function (&$slot, $value) { $slot = $value; return $slot; };\n",
        "$direct = \"old\";\n",
        "echo $callback($direct, \"direct\"), \":\", $direct, \"|\";\n",
        "$frame = \"old\";\n",
        "echo apply($callback, $frame, \"frame\"), \":\", $frame, \"|\";\n",
        "$dynamic = \"old\";\n",
        "$name = \"apply\";\n",
        "echo $name($callback, $dynamic, \"dynamic\"), \":\", $dynamic, \"|\";\n",
        "$nested = \"old\";\n",
        "echo relay($callback, $nested), \":\", $nested, \"|\";\n",
        "$items = [\"a\" => \"old\", \"b\" => [\"c\" => \"deep\"]];\n",
        "$callback($items[\"a\"], \"array\");\n",
        "$callback($items[\"b\"][\"c\"], \"nested\");\n",
        "echo $items[\"a\"], \":\", $items[\"b\"][\"c\"];\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("descriptor_closure_by_reference_parameters", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        run.stdout,
        b"direct:direct|frame:frame|dynamic:dynamic|relay:relay|array:nested"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_binds_typed_default_closure_parameters() {
    let source = concat!(
        "<?php\n",
        "function apply_default($callback, $value) { return $callback($value); }\n",
        "class ClosureParamApply {\n",
        "    public static function apply($callback, $value) { return $callback($value); }\n",
        "}\n",
        "$default = function (int $value = 4, string $suffix = \"x\"): string { return $suffix . \":\" . $value; };\n",
        "echo $default(), \"|\", $default(\"5\", \"y\"), \"|\";\n",
        "echo apply_default(function (int $value = 6) { return $value + 1; }, \"7\"), \"|\";\n",
        "echo ClosureParamApply::apply(fn(int $value = 8): int => $value + 2, \"9\"), \"|\";\n",
        "$base = \"B\";\n",
        "$captured = function (string $suffix = \"d\") use ($base) { return $base . $suffix; };\n",
        "echo $captured(), \":\", $captured(\"e\");\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_closure_call_arg_count")
            && source.contains("phpc_closure_arg_count <")
            && source.contains("phpc_native_value_coerce_call_type_with_diagnostic"),
        "typed/default closure parameters should reuse descriptor arity and call-frame type boundaries:\n{source}"
    );
    assert!(
        source.contains("phpc_user_function_0_apply_default(")
            && source.contains("phpc_declared_method_")
            && source.contains("phpc_native_value_from_closure_descriptor_captures_and_free"),
        "typed/default closure parameters should flow through function, static-method, arrow, and captured-closure consumers:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_CLOSURE_REJECTION)
            && !source.contains("assembly dynamic function-call lowering rejects"),
        "supported typed/default descriptor closures should not hit closure or dynamic-call blockers:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_typed_default_closure_parameter_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function apply_default($callback, $value) { return $callback($value); }\n",
        "class ClosureParamApply {\n",
        "    public static function apply($callback, $value) { return $callback($value); }\n",
        "}\n",
        "$default = function (int $value = 4, string $suffix = \"x\"): string { return $suffix . \":\" . $value; };\n",
        "echo $default(), \"|\", $default(\"5\", \"y\"), \"|\";\n",
        "echo apply_default(function (int $value = 6) { return $value + 1; }, \"7\"), \"|\";\n",
        "echo ClosureParamApply::apply(fn(int $value = 8): int => $value + 2, \"9\"), \"|\";\n",
        "$base = \"B\";\n",
        "$captured = function (string $suffix = \"d\") use ($base) { return $base . $suffix; };\n",
        "echo $captured(), \":\", $captured(\"e\");\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("typed_default_closure_parameters", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"x:4|y:5|8|11|Bd:Be");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_binds_variadic_closure_parameters() {
    let source = concat!(
        "<?php\n",
        "function apply_variadic($callback, $head, $a, $b) { return $callback($head, $a, $b); }\n",
        "class ClosureVariadicApply {\n",
        "    public static function apply($callback, $value) { return $callback(\"S\", $value, \"9\"); }\n",
        "}\n",
        "$rest = function (...$tail) { return $tail[0] ?? \"empty\"; };\n",
        "$head = function ($prefix = \"D\", int ...$tail): string { return $prefix . \":\" . $tail[1]; };\n",
        "echo $rest(), \"|\", $rest(\"A\"), \"|\";\n",
        "echo $head(\"H\", \"4\", 5), \"|\";\n",
        "echo apply_variadic(function ($head, int ...$tail) { return $head . \":\" . $tail[0] . \":\" . $tail[1]; }, \"F\", \"6\", 7), \"|\";\n",
        "echo ClosureVariadicApply::apply(fn(string $prefix, int ...$tail): string => $prefix . \":\" . $tail[0] . \":\" . $tail[1], \"8\"), \"|\";\n",
        "$base = \"B\";\n",
        "$captured = function (string ...$tail) use ($base) { return $base . $tail[0] . $tail[1]; };\n",
        "echo $captured(\"x\", \"y\"), \"|\";\n",
        "$slot = \"old\";\n",
        "$mixed = function (&$slot, string ...$tail) { $slot = $tail[0]; return $slot . $tail[1]; };\n",
        "echo $mixed($slot, \"m\", \"x\"), \":\", $slot;\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("PHPC_NATIVE_CLOSURE_ARGUMENT_VARIADIC")
            && source.contains("PHPC_NATIVE_CLOSURE_ARGUMENT_BY_REFERENCE")
            && source.contains("phpc_native_array_empty")
            && source.contains("phpc_native_array_append_value_with_diagnostic")
            && source.contains("phpc_native_value_from_array"),
        "variadic closure parameters should pack surplus arguments through the shared native array/value ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_coerce_call_type_with_diagnostic"),
        "typed variadic closure arguments should reuse the shared call-frame type ABI per supplied value:\n{source}"
    );
    assert!(
        source.contains("phpc_closure_call_arg_count")
            && source.contains("phpc_user_function_0_apply_variadic(")
            && source.contains("phpc_declared_method_")
            && source.contains("phpc_native_value_from_closure_descriptor_captures_and_free"),
        "variadic closure parameters should flow through function, static-method, arrow, and captured-closure consumers:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_CLOSURE_REJECTION)
            && !source.contains("assembly dynamic function-call lowering rejects"),
        "supported variadic descriptor closures should not hit closure or dynamic-call blockers:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_variadic_closure_parameter_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function apply_variadic($callback, $head, $a, $b) { return $callback($head, $a, $b); }\n",
        "class ClosureVariadicApply {\n",
        "    public static function apply($callback, $value) { return $callback(\"S\", $value, \"9\"); }\n",
        "}\n",
        "$rest = function (...$tail) { return $tail[0] ?? \"empty\"; };\n",
        "$head = function ($prefix = \"D\", int ...$tail): string { return $prefix . \":\" . $tail[1]; };\n",
        "echo $rest(), \"|\", $rest(\"A\"), \"|\";\n",
        "echo $head(\"H\", \"4\", 5), \"|\";\n",
        "echo apply_variadic(function ($head, int ...$tail) { return $head . \":\" . $tail[0] . \":\" . $tail[1]; }, \"F\", \"6\", 7), \"|\";\n",
        "echo ClosureVariadicApply::apply(fn(string $prefix, int ...$tail): string => $prefix . \":\" . $tail[0] . \":\" . $tail[1], \"8\"), \"|\";\n",
        "$base = \"B\";\n",
        "$captured = function (string ...$tail) use ($base) { return $base . $tail[0] . $tail[1]; };\n",
        "echo $captured(\"x\", \"y\"), \"|\";\n",
        "$slot = \"old\";\n",
        "$mixed = function (&$slot, string ...$tail) { $slot = $tail[0]; return $slot . $tail[1]; };\n",
        "echo $mixed($slot, \"m\", \"x\"), \":\", $slot;\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("variadic_closure_parameters", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"empty|A|H:5|F:6:7|S:8:9|Bxy|mx:m");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_captures_arrow_variables_through_descriptor_abi() {
    let source = concat!(
        "<?php\n",
        "function invoke_arrow($callback, $value) { return $callback($value); }\n",
        "function make_arrow($prefix) { return fn($suffix) => $prefix . $suffix; }\n",
        "class ArrowApply {\n",
        "    public static function apply($callback, $value) { return $callback($value); }\n",
        "}\n",
        "$top = \"T\";\n",
        "$items = [\"slot\" => \"A\"];\n",
        "$key = \"slot\";\n",
        "$direct = fn($suffix) => $top . $suffix;\n",
        "$array = fn($suffix) => $items[$key] . $suffix;\n",
        "$setter = fn(&$target, $value) => $target = $value;\n",
        "$nested = fn() => fn($suffix) => $top . $suffix;\n",
        "$regularUse = fn() => function($suffix) use ($top) { return $top . $suffix; };\n",
        "echo $direct(\"0\"), \"|\";\n",
        "echo invoke_arrow(fn($suffix) => $top . $suffix, \"1\"), \"|\";\n",
        "echo ArrowApply::apply(fn($suffix) => $top . $suffix, \"2\"), \"|\";\n",
        "$made = make_arrow(\"M\");\n",
        "echo $made(\"3\"), \"|\";\n",
        "echo $array(\"4\"), \"|\";\n",
        "$slot = \"old\";\n",
        "echo $setter($slot, \"new\"), \":\", $slot, \"|\";\n",
        "$top = \"changed\";\n",
        "echo $direct(\"5\"), \"|\";\n",
        "echo $nested()(\"6\"), \"|\";\n",
        "echo $regularUse()(\"7\");\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_from_closure_descriptor_captures_and_free"),
        "arrow closures should reuse the descriptor capture ABI:\n{source}"
    );
    assert!(
        source.contains("closure_capture_names_") && source.contains("closure_capture_values_"),
        "implicit arrow captures should materialize ordinary capture metadata:\n{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_CLOSURE_ARGUMENT_BY_REFERENCE"),
        "arrow closure parameters should reuse descriptor by-reference metadata:\n{source}"
    );
    assert!(
        source.contains("phpc_user_function_0_invoke_arrow(")
            && source.contains("phpc_user_function_1_make_arrow(")
            && source.contains("phpc_declared_method_"),
        "arrow closures should flow through function, returned-closure, and static-method consumers:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_CLOSURE_REJECTION),
        "supported arrow closure captures should not hit the closure blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_does_not_invent_regular_closure_captures_inside_arrow() {
    let source = concat!(
        "<?php\n",
        "$top = \"T\";\n",
        "$maker = fn() => function () { return $top; };\n",
        "echo $maker()();\n",
    );
    let program = parse(source).unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("variable-read lowering rejects"),
        "regular closures without use() inside arrows must not receive invented captures:\n{source}\n{error:?}"
    );
}

#[test]
fn emit_exe_links_and_runs_arrow_closure_implicit_capture_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function invoke_arrow($callback, $value) { return $callback($value); }\n",
        "function make_arrow($prefix) { return fn($suffix) => $prefix . $suffix; }\n",
        "class ArrowApply {\n",
        "    public static function apply($callback, $value) { return $callback($value); }\n",
        "}\n",
        "$top = \"T\";\n",
        "$items = [\"slot\" => \"A\"];\n",
        "$key = \"slot\";\n",
        "$direct = fn($suffix) => $top . $suffix;\n",
        "$array = fn($suffix) => $items[$key] . $suffix;\n",
        "$setter = fn(&$target, $value) => $target = $value;\n",
        "$nested = fn() => fn($suffix) => $top . $suffix;\n",
        "$regularUse = fn() => function($suffix) use ($top) { return $top . $suffix; };\n",
        "echo $direct(\"0\"), \"|\";\n",
        "echo invoke_arrow(fn($suffix) => $top . $suffix, \"1\"), \"|\";\n",
        "echo ArrowApply::apply(fn($suffix) => $top . $suffix, \"2\"), \"|\";\n",
        "$made = make_arrow(\"M\");\n",
        "echo $made(\"3\"), \"|\";\n",
        "echo $array(\"4\"), \"|\";\n",
        "$slot = \"old\";\n",
        "echo $setter($slot, \"new\"), \":\", $slot, \"|\";\n",
        "$top = \"changed\";\n",
        "echo $direct(\"5\"), \"|\";\n",
        "echo $nested()(\"6\"), \"|\";\n",
        "echo $regularUse()(\"7\");\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("arrow_closure_implicit_captures", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"T0|T1|T2|M3|A4|new:new|T5|T6|T7");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_binds_non_static_closure_this_through_descriptor_captures() {
    let source = concat!(
        "<?php\n",
        "function invoke_bound_this($callback, $value) { return $callback($value); }\n",
        "class ClosureThisApply {\n",
        "    public $value;\n",
        "    public function __construct($value) { $this->value = $value; }\n",
        "    public function makeRegular($mark) { return function ($suffix) use ($mark) { $this->value = $this->value . $mark . $suffix; return $this->value; }; }\n",
        "    public function makeArrow($mark) { return fn($suffix) => $this->value . $mark . $suffix; }\n",
        "    public function applyOwn($value) { $callback = function ($suffix) { $this->value = $this->value . $suffix; return $this->value; }; return $callback($value); }\n",
        "    public static function relay($callback, $value) { return $callback($value); }\n",
        "}\n",
        "$box = new ClosureThisApply(\"A\");\n",
        "$regular = $box->makeRegular(\":\");\n",
        "echo $regular(\"0\"), \":\", $box->value, \"|\";\n",
        "echo invoke_bound_this($regular, \"1\"), \":\", $box->value, \"|\";\n",
        "echo ClosureThisApply::relay($box->makeArrow(\"-\"), \"2\"), \"|\";\n",
        "echo $box->applyOwn(\"3\"), \":\", $box->value;\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_from_closure_descriptor_captures_and_free")
            && source.contains("closure_capture_names_")
            && source.contains("closure_capture_values_"),
        "non-static closures in object frames should carry $this through the descriptor capture ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_NativeValueHandle phpc_this")
            && source.contains("phpc_native_value_clone(phpc_this)")
            && source.contains("phpc_native_value_object_public_property_operation_with_diagnostic")
            && source.contains("PHPC_NATIVE_OBJECT_PUBLIC_PROPERTY_READ")
            && source.contains("PHPC_NATIVE_OBJECT_PUBLIC_PROPERTY_WRITE"),
        "$this should bind as an ordinary object value consumed by the shared property ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_user_function_")
            && source.contains("invoke_bound_this")
            && source.contains("phpc_declared_method_"),
        "$this-bound closures should flow through direct, user-function, static-method, arrow, and in-method callback consumers:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_CLOSURE_REJECTION),
        "supported non-static $this-bound closures should not hit the closure blocker:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_non_static_closure_this_binding_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function invoke_bound_this($callback, $value) { return $callback($value); }\n",
        "class ClosureThisApply {\n",
        "    public $value;\n",
        "    public function __construct($value) { $this->value = $value; }\n",
        "    public function makeRegular($mark) { return function ($suffix) use ($mark) { $this->value = $this->value . $mark . $suffix; return $this->value; }; }\n",
        "    public function makeArrow($mark) { return fn($suffix) => $this->value . $mark . $suffix; }\n",
        "    public function applyOwn($value) { $callback = function ($suffix) { $this->value = $this->value . $suffix; return $this->value; }; return $callback($value); }\n",
        "    public static function relay($callback, $value) { return $callback($value); }\n",
        "}\n",
        "$box = new ClosureThisApply(\"A\");\n",
        "$regular = $box->makeRegular(\":\");\n",
        "echo $regular(\"0\"), \":\", $box->value, \"|\";\n",
        "echo invoke_bound_this($regular, \"1\"), \":\", $box->value, \"|\";\n",
        "echo ClosureThisApply::relay($box->makeArrow(\"-\"), \"2\"), \"|\";\n",
        "echo $box->applyOwn(\"3\"), \":\", $box->value;\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("non_static_closure_this_binding", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native non-static closure $this executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A:0:A:0|A:0:1:A:0:1|A:0:1-2|A:0:13:A:0:13");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_lowers_static_arrow_closures_without_this_binding() {
    let source = concat!(
        "<?php\n",
        "function invoke_static_arrow($callback, $value) { return $callback($value); }\n",
        "function make_static_arrow($prefix) { return static fn($suffix) => $prefix . $suffix; }\n",
        "class StaticArrowApply {\n",
        "    public static function apply($callback, $value) { return $callback($value); }\n",
        "    public function make($prefix) { $mark = \":\"; return static fn($suffix) => $prefix . $mark . $suffix; }\n",
        "}\n",
        "$top = \"T\";\n",
        "$items = [\"slot\" => \"A\"];\n",
        "$key = \"slot\";\n",
        "$direct = static fn($suffix) => $top . $suffix;\n",
        "$array = static fn($suffix) => $items[$key] . $suffix;\n",
        "$setter = static fn(&$target, $value) => $target = $value;\n",
        "$nested = static fn() => static fn($suffix) => $top . $suffix;\n",
        "$packed = static fn(string $prefix = \"D\", int ...$nums): string => $prefix . \":\" . $nums[1];\n",
        "echo $direct(\"0\"), \"|\";\n",
        "echo invoke_static_arrow(static fn($suffix) => $top . $suffix, \"1\"), \"|\";\n",
        "echo StaticArrowApply::apply(static fn($suffix) => $top . $suffix, \"2\"), \"|\";\n",
        "$made = make_static_arrow(\"F\");\n",
        "echo $made(\"3\"), \"|\";\n",
        "$maker = new StaticArrowApply();\n",
        "echo $maker->make(\"M\")(\"4\"), \"|\";\n",
        "echo $array(\"5\"), \"|\";\n",
        "$slot = \"old\";\n",
        "echo $setter($slot, \"new\"), \":\", $slot, \"|\";\n",
        "$top = \"changed\";\n",
        "echo $direct(\"6\"), \"|\";\n",
        "echo $nested()(\"7\"), \"|\";\n",
        "echo $packed(\"P\", \"8\", \"9\");\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_from_closure_descriptor")
            && source.contains("phpc_native_value_from_closure_descriptor_captures_and_free"),
        "static arrows should reuse the descriptor closure and capture ABI:\n{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_CLOSURE_ARGUMENT_BY_REFERENCE")
            && source.contains("PHPC_NATIVE_CLOSURE_ARGUMENT_VARIADIC")
            && source.contains("phpc_closure_call_arg_count"),
        "static arrows should reuse descriptor parameter metadata, by-reference carriers, and variadic/default binding:\n{source}"
    );
    assert!(
        source.contains("phpc_user_function_")
            && source.contains("invoke_static_arrow")
            && source.contains("make_static_arrow")
            && source.contains("phpc_declared_method_"),
        "static arrows should flow through direct, user-function, static-method, and instance-method-return consumers:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_CLOSURE_REJECTION),
        "supported static arrows should not hit the closure blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_blocks_static_arrow_this_binding_on_shared_variable_read_boundary() {
    let source = concat!(
        "<?php\n",
        "class StaticArrowThisBinding {\n",
        "    public function make() { return static fn() => $this; }\n",
        "}\n",
        "$callback = (new StaticArrowThisBinding())->make();\n",
        "echo $callback();\n",
    );
    let program = parse(source).unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("variable-read lowering rejects"),
        "static arrows must not receive an implicit $this binding:\n{error:?}"
    );
}

#[test]
fn emit_exe_links_and_runs_static_arrow_closure_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function invoke_static_arrow($callback, $value) { return $callback($value); }\n",
        "function make_static_arrow($prefix) { return static fn($suffix) => $prefix . $suffix; }\n",
        "class StaticArrowApply {\n",
        "    public static function apply($callback, $value) { return $callback($value); }\n",
        "    public function make($prefix) { $mark = \":\"; return static fn($suffix) => $prefix . $mark . $suffix; }\n",
        "}\n",
        "$top = \"T\";\n",
        "$items = [\"slot\" => \"A\"];\n",
        "$key = \"slot\";\n",
        "$direct = static fn($suffix) => $top . $suffix;\n",
        "$array = static fn($suffix) => $items[$key] . $suffix;\n",
        "$setter = static fn(&$target, $value) => $target = $value;\n",
        "$nested = static fn() => static fn($suffix) => $top . $suffix;\n",
        "$packed = static fn(string $prefix = \"D\", int ...$nums): string => $prefix . \":\" . $nums[1];\n",
        "echo $direct(\"0\"), \"|\";\n",
        "echo invoke_static_arrow(static fn($suffix) => $top . $suffix, \"1\"), \"|\";\n",
        "echo StaticArrowApply::apply(static fn($suffix) => $top . $suffix, \"2\"), \"|\";\n",
        "$made = make_static_arrow(\"F\");\n",
        "echo $made(\"3\"), \"|\";\n",
        "$maker = new StaticArrowApply();\n",
        "echo $maker->make(\"M\")(\"4\"), \"|\";\n",
        "echo $array(\"5\"), \"|\";\n",
        "$slot = \"old\";\n",
        "echo $setter($slot, \"new\"), \":\", $slot, \"|\";\n",
        "$top = \"changed\";\n",
        "echo $direct(\"6\"), \"|\";\n",
        "echo $nested()(\"7\"), \"|\";\n",
        "echo $packed(\"P\", \"8\", \"9\");\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("static_arrow_closure_implicit_captures", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native static arrow executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"T0|T1|T2|F3|M:4|A5|new:new|T6|T7|P:9");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_lowers_static_descriptor_closures_without_this_binding() {
    let source = concat!(
        "<?php\n",
        "function apply_static_closure($callback, $value) { return $callback($value); }\n",
        "class StaticClosureApply {\n",
        "    public static function apply($callback, $value) { return $callback($value); }\n",
        "    public function make($prefix) { $mark = \":\"; return static function ($suffix) use ($prefix, $mark) { return $prefix . $mark . $suffix; }; }\n",
        "}\n",
        "$base = \"B\";\n",
        "$direct = static function ($suffix) use ($base) { return $base . $suffix; };\n",
        "echo $direct(\"0\"), \"|\";\n",
        "echo apply_static_closure(static function ($suffix) { return \"F\" . $suffix; }, \"1\"), \"|\";\n",
        "echo StaticClosureApply::apply(static function ($suffix) { return \"S\" . $suffix; }, \"2\"), \"|\";\n",
        "$maker = new StaticClosureApply();\n",
        "$made = $maker->make(\"M\");\n",
        "echo $made(\"3\"), \"|\";\n",
        "$slot = \"old\";\n",
        "$mutator = static function (&$target, $suffix = \"4\"): string { $target = \"R\" . $suffix; return $target; };\n",
        "echo $mutator($slot), \":\", $slot, \"|\";\n",
        "$packed = static function (string $prefix = \"D\", int ...$nums): string { return $prefix . \":\" . $nums[1]; };\n",
        "echo $packed(\"V\", \"5\", 6), \"|\";\n",
        "$ref = \"Q\";\n",
        "$capturedRef = static function ($suffix) use (&$ref) { $ref = $ref . $suffix; return $ref; };\n",
        "echo $capturedRef(\"R\"), \":\", $ref;\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_from_closure_descriptor")
            && source.contains("phpc_native_value_from_closure_descriptor_captures_and_free"),
        "static closures should reuse the descriptor closure and capture ABI:\n{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_CLOSURE_ARGUMENT_BY_REFERENCE"),
        "static closures should reuse descriptor by-reference parameter/capture carriers:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_CLOSURE_REJECTION),
        "supported static descriptor closures should not hit the closure blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_blocks_static_closure_this_binding_on_shared_variable_read_boundary()
{
    let source = concat!(
        "<?php\n",
        "class StaticClosureThisBinding {\n",
        "    public function make() { return static function () { return $this; }; }\n",
        "}\n",
        "$callback = (new StaticClosureThisBinding())->make();\n",
        "echo $callback();\n",
    );
    let program = parse(source).unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("variable-read lowering rejects"),
        "static closures must not receive an implicit $this binding:\n{error:?}"
    );
}

#[test]
fn emit_exe_links_and_runs_static_descriptor_closure_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function apply_static_closure($callback, $value) { return $callback($value); }\n",
        "class StaticClosureApply {\n",
        "    public static function apply($callback, $value) { return $callback($value); }\n",
        "    public function make($prefix) { $mark = \":\"; return static function ($suffix) use ($prefix, $mark) { return $prefix . $mark . $suffix; }; }\n",
        "}\n",
        "$base = \"B\";\n",
        "$direct = static function ($suffix) use ($base) { return $base . $suffix; };\n",
        "echo $direct(\"0\"), \"|\";\n",
        "echo apply_static_closure(static function ($suffix) { return \"F\" . $suffix; }, \"1\"), \"|\";\n",
        "echo StaticClosureApply::apply(static function ($suffix) { return \"S\" . $suffix; }, \"2\"), \"|\";\n",
        "$maker = new StaticClosureApply();\n",
        "$made = $maker->make(\"M\");\n",
        "echo $made(\"3\"), \"|\";\n",
        "$slot = \"old\";\n",
        "$mutator = static function (&$target, $suffix = \"4\"): string { $target = \"R\" . $suffix; return $target; };\n",
        "echo $mutator($slot), \":\", $slot, \"|\";\n",
        "$packed = static function (string $prefix = \"D\", int ...$nums): string { return $prefix . \":\" . $nums[1]; };\n",
        "echo $packed(\"V\", \"5\", 6), \"|\";\n",
        "$ref = \"Q\";\n",
        "$capturedRef = static function ($suffix) use (&$ref) { $ref = $ref . $suffix; return $ref; };\n",
        "echo $capturedRef(\"R\"), \":\", $ref;\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("static_descriptor_closures", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native static descriptor closure executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"B0|F1|S2|M:3|R4:R4|V:6|QR:QR");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_keeps_unsupported_closure_shapes_on_shared_blocker() {
    for source in [
        concat!(
            "<?php\n",
            "function make_capture() { return function () use (&$missing) { return $missing; }; }\n",
            "$callback = make_capture();\n",
            "echo $callback();\n",
        ),
        concat!(
            "<?php\n",
            "$callback = function (&...$values) { return 1; };\n",
            "echo $callback(1, 2);\n",
        ),
        concat!(
            "<?php\n",
            "function from_root() { global $value; return $value; }\n",
            "$value = 1;\n",
            "$callback = function () { return from_root(); };\n",
            "echo $callback();\n",
        ),
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, ASSEMBLY_CLOSURE_REJECTION);
    }
}

#[test]
fn emit_exe_links_and_runs_foreach_body_array_offset_unset_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("foreach_body_array_offset_unset.php");
    let output_path = native_link_output_path("foreach_body_array_offset_unset");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        "<?php\n$items = [\"x\" => \"A\", \"y\" => \"B\", \"z\" => \"C\"];\n$other = [\"z\" => \"Z\", \"keep\" => \"K\"];\n$nested = [\"outer\" => [\"drop\" => \"D\", \"keep\" => \"N\"]];\nforeach ($items as $key => $value) { unset($items[$key], $other[\"z\"], $nested[\"outer\"][\"drop\"]); echo $key, \":\", $value, \";\"; }\necho \"|\", isset($items[\"x\"]) ? 1 : 0, isset($items[\"z\"]) ? 1 : 0, isset($other[\"z\"]) ? 1 : 0, isset($other[\"keep\"]) ? 1 : 0, isset($nested[\"outer\"][\"drop\"]) ? 1 : 0, isset($nested[\"outer\"][\"keep\"]) ? 1 : 0;\n",
    )
    .expect("write foreach body unset native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native foreach body unset source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native foreach body unset executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native foreach body unset executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"x:A;y:B;z:C;|000101");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

const NATIVE_VALUE_RESULT_STRLEN_SOURCE: &str = concat!(
    "<?php\n",
    "$items = [\"word\" => \"go\"];\n",
    "echo strlen(strtoupper($items[\"word\"])), \"|\";\n",
    "echo strlen((string)array_sum([12, 3])), \"|\";\n",
    "echo strlen(gettype((array)null)), \"|\";\n",
    "echo strlen($items[\"word\"] . \"!\");\n",
);

#[test]
fn native_executable_c_source_routes_strlen_through_string_conversion_result() {
    let program = parse(
        "<?php\n$payload = \"A\0B\";\necho strlen(42);\necho strlen(false);\necho strlen(null);\necho strlen($payload);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_NativeStringConversionResult"),
        "{source}"
    );
    assert!(source.contains("phpc_NativeByteBuffer"), "{source}");
    assert!(source.contains("phpc_native_value_from_scalar"), "{source}");
    assert!(
        source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_value_to_string_bytes(")
            .count(),
        4,
        "{source}"
    );
    assert_eq!(
        source
            .matches("  phpc_native_string_conversion_result_free(")
            .count(),
        4,
        "{source}"
    );
    assert!(
        source.contains(".bytes.len"),
        "strlen should use runtime conversion byte lengths:\n{source}"
    );
    assert!(
        !source.contains("strlen((const char *)"),
        "generated C should not use C strlen for PHP strlen operands:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_strlen_value_results_through_string_conversion() {
    let program = parse(NATIVE_VALUE_RESULT_STRLEN_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    for producer in [
        "strtoupper_result_",
        "native_value_cast_",
        "native_value_type_name_",
        "native_value_binary_",
    ] {
        assert!(
            body.contains(&format!("phpc_native_value_to_string_bytes({producer}")),
            "strlen should consume {producer} through the native string conversion boundary:\n{source}"
        );
        assert!(
            body.contains(&format!("phpc_native_value_free({producer}")),
            "strlen should release the owned value-result producer {producer}:\n{source}"
        );
    }

    assert!(
        body.matches(" = phpc_native_value_to_string_bytes(")
            .count()
            >= 4,
        "{source}"
    );
    assert!(
        !source.contains("function-call lowering rejects function calls"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_predicates_through_runtime_contract() {
    let program = parse(
        "<?php\n$payload = \"A\0B\";\necho str_starts_with($payload, \"A\0\");\necho str_ends_with($payload, \"\0B\");\necho str_contains(42, \"2\");\necho str_contains($payload, \"C\");\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_predicate_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_value_string_predicate_with_diagnostic(")
            .count(),
        4,
        "{source}"
    );
    assert_eq!(
        source
            .matches("phpc_NativeDiagnosticHandle string_predicate_diagnostic_")
            .count(),
        4,
        "{source}"
    );
    assert!(
        source.contains("static const uint8_t phpc_native_value_bytes_"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_from_scalar"),
        "scalar operands should be admitted through the native value boundary:\n{source}"
    );
    assert!(
        !source.contains("strncmp(")
            && !source.contains("strstr(")
            && !source.contains("strlen((const char *)"),
        "string predicates should not use C string APIs for PHP byte semantics:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_int_builtins_through_runtime_contract() {
    let program = parse(
        "<?php\n$payload = \"A\0B\";\necho strcasecmp($payload, \"a\0b\");\necho strcmp($payload, \"a\0b\");\necho strncmp($payload, \"A\0C\", 3);\necho strncasecmp($payload, \"a\0c\", \"2\");\necho ord($payload);\necho ord(42042);\necho crc32(\"123456789\");\necho crc32($payload);\necho crc32(null);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_int_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = (long long)phpc_native_value_string_int_operation_with_diagnostic(")
            .count(),
        9,
        "{source}"
    );
    assert_eq!(
        source
            .matches("phpc_NativeDiagnosticHandle string_int_diagnostic_")
            .count(),
        9,
        "{source}"
    );
    assert!(
        source.contains(", 0, &string_int_diagnostic_")
            && source.contains(", 2, &string_int_diagnostic_")
            && source.contains(", 3, &string_int_diagnostic_")
            && source.contains(", 4, &string_int_diagnostic_")
            && source.contains(", 5, &string_int_diagnostic_")
            && source.contains(", 6, &string_int_diagnostic_"),
        "byte compare, prefix compare, ord, and crc32 should share the tagged string-int ABI:\n{source}"
    );
    assert_eq!(
        source
            .matches(" = (long long)phpc_native_value_to_int64_with_diagnostic(")
            .count(),
        2,
        "prefix compare lengths should share the native int conversion ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_from_scalar")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "scalar and string operands should both enter the native value boundary:\n{source}"
    );
    assert!(
        !source.contains("strlen((const char *)"),
        "string-int builtins should use PHP value-to-string byte conversion:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_search_builtins_through_value_results() {
    let program = parse(
        "<?php\n$payload = \"A\0B\";\n$repeated = \"A\0BA\0B\";\necho strpos($repeated, $payload);\necho strpos($repeated, $payload, 2);\necho strpos($repeated, \"missing\");\necho substr_count($repeated, $payload, 0, 6);\necho substr_count(42042, 42);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_search_result_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_value_string_search_result_with_diagnostic(")
            .count(),
        5,
        "{source}"
    );
    assert!(
        source.contains(", 0, &string_search_diagnostic_")
            && source.contains(", 1, &string_search_diagnostic_"),
        "strpos and substr_count should share the tagged string-search value-result ABI:\n{source}"
    );
    assert_eq!(
        source
            .matches(" = (long long)phpc_native_value_to_int64_with_diagnostic(")
            .count(),
        3,
        "strpos offset and substr_count offset/length should share the native int conversion ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_format_stdout_with_diagnostic")
            && source.contains("phpc_native_value_free"),
        "string-search results should stay PHP-shaped values through stdout and cleanup:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_distance_builtins_through_runtime_contract() {
    let program = parse(
        "<?php\n$left = \"kitten\";\n$right = \"sitting\";\n$insert = 1;\n$replace = 2;\n$delete = 1;\necho levenshtein($left, $right);\necho levenshtein(\"A\0B\", \"A\0C\", $insert, $replace, $delete);\necho similar_text(42042, 42);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_distance_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = (long long)phpc_native_value_string_distance_operation_with_diagnostic(")
            .count(),
        3,
        "{source}"
    );
    assert_eq!(
        source
            .matches("phpc_NativeDiagnosticHandle string_distance_diagnostic_")
            .count(),
        3,
        "{source}"
    );
    assert!(
        source.contains(", 0, &string_distance_diagnostic_")
            && source.contains(", 1, &string_distance_diagnostic_"),
        "levenshtein and similar_text should share the tagged string-distance ABI:\n{source}"
    );
    assert_eq!(
        source
            .matches(" = (long long)phpc_native_value_to_int64_with_diagnostic(")
            .count(),
        3,
        "levenshtein costs should share the native int conversion ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_from_scalar")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "scalar and string operands should both enter the native value boundary:\n{source}"
    );
    assert!(
        !source.contains("strlen((const char *)"),
        "string-distance builtins should use PHP value-to-string byte conversion:\n{source}"
    );
}

const NATIVE_STRING_RESULT_SOURCE: &str = "<?php\n$payload = \"A\0B\";\necho strrev($payload), \"|\";\nprint str_rot13(\"Az-09\");\necho \"|\";\necho bin2hex($payload), \"|\";\necho strtolower(\"MiXeD\"), \"|\";\necho strtoupper(strtolower(\"MiXeD\")), \"|\";\necho ucfirst(\"word\"), \"|\";\necho lcfirst(\"Word\"), \"|\";\necho strrev(42042);\n";
const SHELL_ESCAPE_STRING_RESULT_SOURCE: &str = concat!(
    "<?php\n",
    "$payload = \"X ;\\$'Q\\\"\";\n",
    "echo escapeshellarg($payload), \"|\";\n",
    "echo escapeshellcmd($payload), \"|\";\n",
    "echo escapeshellarg(42042);\n",
);

#[test]
fn native_executable_c_source_routes_unary_string_results_through_runtime_contract() {
    let program = parse(NATIVE_STRING_RESULT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_value_string_result_operation_with_diagnostic(")
            .count(),
        9,
        "{source}"
    );
    assert_eq!(
        source
            .matches("phpc_NativeDiagnosticHandle string_result_diagnostic_")
            .count(),
        9,
        "{source}"
    );
    for operation_tag in ["4", "5", "13", "48", "49", "53", "54"] {
        assert!(
            source.contains(&format!(", {operation_tag}, &string_result_diagnostic_")),
            "tagged unary string-result operation {operation_tag} should route through the shared ABI:\n{source}"
        );
    }
    assert!(
        source.contains("phpc_native_value_from_scalar")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "scalar and string operands should both enter the native value boundary:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_format_stdout_with_diagnostic("),
        "string-result handles should be consumed through native value output:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_shell_escape_results_through_runtime_contract() {
    let program = parse(SHELL_ESCAPE_STRING_RESULT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_value_string_result_operation_with_diagnostic(")
            .count(),
        3,
        "{source}"
    );
    for operation_tag in ["70", "71"] {
        assert!(
            source.contains(&format!(", {operation_tag}, &string_result_diagnostic_")),
            "tagged shell-escape string-result operation {operation_tag} should route through the shared ABI:\n{source}"
        );
    }
    assert!(
        source.contains("phpc_native_value_from_scalar")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "scalar and string shell-escape operands should both enter the native value boundary:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_format_stdout_with_diagnostic("),
        "shell-escape result handles should be consumed through native value output:\n{source}"
    );
}

const STRING_OFFSET_ISSET_EMPTY_SOURCE: &str = "<?php\n$selected = \"A0\0B\";\n$offset = \"1\";\necho isset($selected[0], $selected[$offset]) ? 1 : 0;\necho \"|\";\necho empty($selected[$offset]) ? 1 : 0;\necho \"|\";\necho isset($selected[99]) ? 1 : 0;\necho \"|\";\necho empty((\"102\")[1]) ? 1 : 0;\necho \"|\";\necho empty(strrev(\"za\")[1]) ? 1 : 0;\n";

const VALUE_OFFSET_PRESENCE_SOURCE: &str = "<?php\n$items = [\"hit\" => \"V\", \"null\" => null, \"empty\" => \"\"];\n$key = \"hit\";\n$text = \"A0\";\necho isset($items[$key]) ? 1 : 0;\necho \"|\";\necho empty($items[\"null\"]) ? 1 : 0;\necho \"|\";\necho isset($items[\"missing\"]) ? 1 : 0;\necho \"|\";\necho empty($items[\"empty\"]) ? 1 : 0;\necho \"|\";\necho isset($text[1]) ? 1 : 0;\necho \"|\";\necho empty($text[0]) ? 1 : 0;\n";

const STRING_OFFSET_READ_SOURCE: &str = concat!(
    "<?php\n",
    "$selected = \"A",
    "\0",
    "B\";\n",
    "$offset = \"1\";\n",
    "echo $selected[0], \"|\";\n",
    "echo $selected[$offset], \"|\";\n",
    "echo strlen($selected[$offset]), \"|\";\n",
    "$a = [];\n",
    "$a[$selected[0]] = $selected[2];\n",
    "echo $a[\"A\"];\n",
);

const STRING_OFFSET_WRITE_SOURCE: &str = concat!(
    "<?php\n",
    "$flag = (1 + 2) === 3;\n",
    "$s = $flag ? \"ABCD\" : \"WXYZ\";\n",
    "$i = \"1\";\n",
    "$rep = $flag ? \"",
    "\0",
    "\" : \"Q\";\n",
    "$s[$i] = $rep;\n",
    "echo $s;\n",
    "$a = [];\n",
    "$a[$s] = $flag ? \"V",
    "\0",
    "\" : \"Z\";\n",
    "echo \"|\", $a[$s];\n",
    "$s[3] = \"!\";\n",
    "echo \"|\", $s;\n",
);

const VALUE_OFFSET_MUTATION_ARRAY_WRITE_SOURCE: &str = "<?php\n$items = [\"seed\" => \"A\"];\n$key = \"dyn\";\n$items[$key] = \"B\";\n$slot = 2;\n$items[$slot] = \"C\";\necho $items[\"seed\"], \"|\", $items[$key], \"|\", $items[$slot], \"|\";\necho isset($items[$key]) ? 1 : 0;\n";

const ARRAY_LVALUE_NESTED_WRITE_SOURCE: &str = "<?php\n$outer = \"outer\";\n$inner = \"inner\";\n$items = [$outer => [$inner => \"old\", \"stay\" => \"S\"], \"root\" => \"R\"];\n$value = \"new\";\n$items[$outer][$inner] = $value;\n$items[$outer][\"added\"] = \"A\" . \"B\";\necho isset($items[$outer][$inner]) ? 1 : 0;\necho \"|\";\necho empty($items[$outer][\"added\"]) ? 1 : 0;\necho \"|\";\necho isset($items[$outer][\"stay\"]) ? 1 : 0;\necho \"|\", $items[\"root\"];\n";

const ARRAY_LVALUE_NESTED_APPEND_SOURCE: &str = "<?php\n$outer = \"outer\";\n$leaf = \"leaf\";\n$items = [$outer => [\"stay\" => \"S\"], \"root\" => \"R\"];\n$value = \"new\";\n$items[$outer][] = $value;\n$items[\"created\"][] = \"C\";\n$items[][$leaf] = \"Z\";\necho isset($items[$outer][0]) ? 1 : 0;\necho \"|\";\necho empty($items[\"created\"][0]) ? 1 : 0;\necho \"|\";\necho isset($items[0][$leaf]) ? 1 : 0;\necho \"|\", $items[\"root\"];\n";

const ARRAY_LVALUE_NESTED_ASSIGNMENT_EXPR_SOURCE: &str = "<?php\n$outer = \"outer\";\n$leaf = \"leaf\";\n$items = [$outer => [\"stay\" => \"S\"]];\necho ($items[$outer][$leaf] = \"A\"), \"|\";\necho ($items[$outer][] = \"B\"), \"|\";\necho ($items[][$leaf] = \"C\"), \"|\";\necho isset($items[$outer][$leaf]) ? 1 : 0;\necho \"|\";\necho empty($items[$outer][0]) ? 1 : 0;\necho \"|\";\necho isset($items[0][$leaf]) ? 1 : 0;\necho \"|\";\necho isset($items[$outer][\"stay\"]) ? 1 : 0;\n";

const ARRAY_LVALUE_NESTED_READ_SOURCE: &str = "<?php\n$outer = \"outer\";\n$inner = \"inner\";\n$leaf = \"leaf\";\n$items = [$outer => [$inner => \"v\"], \"other\" => [$leaf => \"x\"]];\n$out = [];\n$out[] = $items[$outer][$inner];\necho $items[$outer][$inner], \"|\";\nprint $items[\"other\"][$leaf];\necho \"|\", strtoupper($items[$outer][$inner]), \"|\", $out[0];\n";

const ARRAY_LVALUE_COMPOUND_ASSIGNMENT_SOURCE: &str = "<?php\n$key = \"slot\";\n$alt = \"alt\";\n$items = [$key => 2, $alt => 10, \"text\" => \"A\"];\n$out = [];\n$items[$key] += 3;\n$twenty = ($items[$alt] *= 2);\necho $twenty;\n$out[($items[$key] .= \"x\")] = ($items[$alt] -= 5);\necho \"|\", $out[\"5x\"], \"|\", $items[$alt], \"|\", $items[$key];\n";

const DIRECT_VARIABLE_COMPOUND_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$count = 2;\n",
    "$count += 5;\n",
    "$again = ($count += 1);\n",
    "echo $count, \":\", $again, \"|\";\n",
    "$text = \"A\";\n",
    "$text .= \"b\";\n",
    "echo $text, \"|\";\n",
    "$product = 6;\n",
    "$product *= 3;\n",
    "echo $product, \"|\";\n",
    "$items = [\"left\" => 1];\n",
    "$items += [\"right\" => 2, \"left\" => 9];\n",
    "echo $items[\"left\"], \":\", $items[\"right\"], \"|\";\n",
    "function bump(&$slot) {\n",
    "    $slot += 4;\n",
    "    return $slot;\n",
    "}\n",
    "$value = 6;\n",
    "echo bump($value), \":\", $value;\n",
    "$value += 1;\n",
    "echo \":\", $value;\n",
);

const DIRECT_VARIABLE_ASSIGNMENT_EXPRESSION_SOURCE: &str = concat!(
    "<?php\n",
    "function overwrite(&$alias) {\n",
    "    echo ($alias = 9), \":\";\n",
    "}\n",
    "$value = 1;\n",
    "echo ($value = 2), \":\", $value, \"|\";\n",
    "$sum = (($left = 3) + ($right = 4));\n",
    "echo $left, \":\", $right, \":\", $sum, \"|\";\n",
    "$word = ($copy = strtoupper(\"go\"));\n",
    "echo $word, \":\", $copy, \"|\";\n",
    "$slot = 5;\n",
    "overwrite($slot);\n",
    "echo $slot, \"|\";\n",
    "$g = \"old\";\n",
    "echo $GLOBALS[\"g\"], \"|\";\n",
    "echo ($g = strtoupper(\"new\")), \":\", $GLOBALS[\"g\"];\n",
);

const DIRECT_VARIABLE_NATIVE_RESULT_ASSIGNMENT_EXPRESSION_SOURCE: &str = concat!(
    "<?php\n",
    "echo ($upper = strtoupper(\"go\")), \":\", $upper, \"|\";\n",
    "echo ($pos = strpos(\"abc\", \"b\")), \":\", $pos;\n",
);

const ARRAY_LVALUE_COMPOUND_ARRAY_UNION_SOURCE: &str = "<?php\n$box = [];\n$box[\"left\"] = [0 => \"left-zero\", \"name\" => \"left-name\"];\n$box[\"left\"] += [0 => \"right-zero\", 1 => \"right-one\", \"name\" => \"right-name\", \"role\" => \"right-role\"];\n$outer = \"outer\";\n$slot = \"slot\";\n$box[$outer][$slot] = [\"keep\" => \"nested-left\"];\n$box[$outer][$slot] += [\"keep\" => \"nested-right\", \"add\" => \"nested-add\"];\necho $box[\"left\"][0], \"|\", $box[\"left\"][1], \"|\", $box[\"left\"][\"name\"], \"|\", $box[\"left\"][\"role\"], \"|\", $box[$outer][$slot][\"keep\"], \"|\", $box[$outer][$slot][\"add\"];\n";

const ARRAY_CAST_VALUE_RESULT_SOURCE: &str = "<?php\n$box = [];\n$fallback = [0 => 2];\n$box[\"union\"] = [\"left\" => 1];\necho (string)($box[\"direct\"] = [\"a\" => 1]), \"|\", strval($box[\"coalesced\"] ??= $fallback), \"|\", (string)($box[\"union\"] += [\"right\" => 2]), \"|\", $box[\"union\"][\"right\"];\n";

const ARRAY_LVALUE_INCREMENT_DECREMENT_SOURCE: &str = "<?php\n$key = \"slot\";\n$float = \"float\";\n$items = [$key => 4, $float => 1.5, \"other\" => 9];\n$items[$key]++;\necho ++$items[$key], \"|\", $items[$key]--, \"|\", $items[$key], \"|\";\n$oldFloat = $items[$float]--;\necho $oldFloat, \"|\", $items[$float], \"|\";\n$out = [];\n$out[++$items[$key]] = $items[$key]--;\necho $out[6], \"|\", $items[$key];\n";

const ARRAY_LVALUE_INCREMENT_DECREMENT_MISSING_SOURCE: &str = "<?php\n$key = \"missing\";\n$leaf = \"leaf\";\n$items = [\"nil\" => null, \"outer\" => [\"null_leaf\" => null]];\n$post = $items[$key]++;\n$pre = ++$items[\"outer\"][$leaf];\n$items[\"down\"]--;\n$nullPost = $items[\"nil\"]++;\n$nullPre = ++$items[\"outer\"][\"null_leaf\"];\necho $post, \"|\", $items[$key], \"|\", $pre, \"|\", $items[\"outer\"][$leaf], \"|\", empty($items[\"down\"]) ? 1 : 0, \"|\", $nullPost, \"|\", $items[\"nil\"], \"|\", $nullPre, \"|\", $items[\"outer\"][\"null_leaf\"];\n";

const ARRAY_LVALUE_APPEND_INCREMENT_DECREMENT_SOURCE: &str = "<?php\n$outer = \"outer\";\n$items = [$outer => []];\n$items[]++;\necho $items[0], \"|\";\necho ++$items[], \"|\", $items[1], \"|\";\n$post = $items[$outer][]++;\necho $post, \"|\", $items[$outer][0], \"|\";\n$old = $items[]++;\necho $old, \"|\", $items[2];\n";

const ARRAY_LVALUE_NESTED_RMW_SOURCE: &str = "<?php\n$outer = \"outer\";\n$leaf = \"leaf\";\n$other = \"other\";\n$items = [$outer => [$leaf => 3, $other => 10]];\n$items[$outer][$leaf] += 4;\n$compound = ($items[$outer][$other] *= 2);\necho $items[$outer][$leaf], \"|\", $compound, \"|\";\n$post = $items[$outer][$leaf]++;\necho $post, \"|\", ++$items[$outer][$leaf], \"|\";\n$out = [];\n$out[$items[$outer][$leaf] += 1] = $items[$outer][$other]--;\necho $out[10], \"|\", $items[$outer][$other], \"|\", $items[$outer][$leaf];\n";

const ARRAY_LVALUE_RMW_OWNER_BOUNDARY_SOURCE: &str = concat!(
    "<?php\n",
    "$local = [\"n\" => 1, \"maybe\" => null, \"inc\" => 4];\n",
    "$local[\"n\"] += 5;\n",
    "echo $local[\"n\"], \"|\";\n",
    "echo ($local[\"missing\"] ??= \"L\"), \"|\";\n",
    "echo ++$local[\"inc\"];\n",
    "$bag = [\"n\" => 2, \"maybe\" => null, \"inc\" => 3];\n",
    "function mutate_global_bag() {\n",
    "    global $bag;\n",
    "    $bag[\"n\"] += 7;\n",
    "    echo \"|\", $bag[\"n\"];\n",
    "    echo \":\", ($bag[\"maybe\"] ??= \"G\");\n",
    "    echo \":\", $bag[\"inc\"]++;\n",
    "    echo \":\", $bag[\"inc\"];\n",
    "}\n",
    "mutate_global_bag();\n",
    "echo \"|\", $bag[\"n\"], \":\", $bag[\"maybe\"], \":\", $bag[\"inc\"];\n",
);

const VALUE_OFFSET_MUTATION_ARRAY_APPEND_SOURCE: &str = "<?php\n$items = [\"seed\" => \"A\"];\n$items[] = \"B\";\n$value = \"C\";\n$items[] = $value;\necho $items[\"seed\"], \"|\", $items[0], \"|\", $items[1], \"|\";\necho isset($items[1]) ? 1 : 0;\n";

const VALUE_OFFSET_MUTATION_VALUE_WRITE_SOURCE: &str = "<?php\n$key = \"dyn\";\n$missing[$key] = \"U\";\n$null = null;\n$null[\"n\"] = \"N\";\n$false = false;\n$false[2] = \"F\";\n$int = 3;\n$int[\"x\"] = \"bad\";\necho $missing[$key], \"|\", $null[\"n\"], \"|\", $false[2], \"|\", $int;\n";

const VALUE_OFFSET_MUTATION_VALUE_APPEND_SOURCE: &str = "<?php\n$null = null;\n$null[] = \"A\";\n$false = false;\n$false[] = \"BC\";\n$int = 3;\n$int[] = \"x\";\necho $null[0], \"|\", $false[0], \"|\", $int;\n";

const VALUE_OFFSET_PATH_MUTATION_SOURCE: &str = "<?php\n$key = \"a\";\n$null = null;\n$null[$key][\"b\"] = \"A\\0\";\n$false = false;\n$false[\"bucket\"][][\"leaf\"] = \"B\";\n$int = 3;\n$int[\"x\"][\"y\"] = \"z\";\necho isset($null[$key]) ? 1 : 0, \"|\", isset($false[\"bucket\"]) ? 1 : 0, \"|\", $int;\n";

const VALUE_OFFSET_PATH_ASSIGNMENT_EXPR_SOURCE: &str = "<?php\n$key = \"a\";\n$null = null;\necho ($null[$key][\"b\"] = \"A\"), \"|\", isset($null[$key]) ? 1 : 0, \"|\";\n$false = false;\n$value = \"B\";\necho ($false[\"bucket\"][][\"leaf\"] = $value), \"|\", isset($false[\"bucket\"]) ? 1 : 0, \"|\";\n$source = [\"rhs\" => \"C\"];\n$target = null;\n$stored = ($target[\"copy\"][\"leaf\"] = $source[\"rhs\"]);\necho $stored, \"|\", isset($target[\"copy\"]) ? 1 : 0;\n";

const VALUE_OFFSET_PATH_UNSET_SOURCE: &str = "<?php\n$key = \"drop\";\n$leaf = \"leaf\";\n$items = null;\n$items[$key][\"value\"] = \"D\";\n$items[\"root\"][$leaf] = \"L\";\nunset($items[$key]);\nunset($items[\"root\"][$leaf]);\n$null = null;\nunset($null[\"missing\"]);\necho isset($items[$key]) ? 1 : 0;\necho \"|\";\necho isset($items[\"root\"][$leaf]) ? 1 : 0;\necho \"|\";\necho isset($items[\"root\"]) ? 1 : 0;\necho \"|\", $null;\n";

const VALUE_OFFSET_MUTATION_ARRAY_ASSIGNMENT_EXPR_SOURCE: &str = "<?php\n$items = [\"seed\" => \"A\"];\n$key = \"named\";\n$value = \"C\";\necho ($items[] = \"B\"), \"|\";\necho ($items[] = $value), \"|\";\necho ($items[$key] = \"D\"), \"|\";\necho $items[0], \"|\", $items[1], \"|\", $items[$key], \"|\";\necho isset($items[1]) ? 1 : 0;\n";

const VALUE_OFFSET_ARRAY_READ_SOURCE: &str = "<?php\n$items = [\"first\" => \"q\", 2 => \"B\"];\n$key = \"first\";\n$out = [];\n$out[] = $items[$key];\necho $items[$key], \"|\";\nprint $items[2];\necho \"|\", $out[0], \"|\";\necho strtoupper($items[$key]);\n";

const VALUE_OFFSET_READ_RECOVERY_SOURCE: &str = "<?php\n$items = [\"present\" => \"P\", \"outer\" => [\"scalar\" => 7, \"nullish\" => null]];\n$missing = \"missing\";\necho $items[\"present\"], \"|\";\necho $items[$missing], \"|\";\necho $items[\"outer\"][\"scalar\"][\"leaf\"], \"|\";\n$slot = $items[\"outer\"][\"absent\"];\n$copy = $slot;\nprint $copy;\necho \"|\";\necho isset($items[\"present\"]) ? 1 : 0;\n";

const VALUE_OFFSET_NULL_COALESCE_SOURCE: &str = "<?php\n$items = [\"present\" => \"L\", \"nullish\" => null, 2 => \"N\"];\n$key = \"present\";\n$missing = \"missing\";\n$text = \"abc\";\n$offset = \"1\";\necho ($items[$key] ?? \"fallback\");\necho \"|\";\necho ($items[$missing] ?? \"fallback\");\necho \"|\";\necho ($items[\"nullish\"] ?? \"fallback\");\necho \"|\";\necho ($text[$offset] ?? \"fallback\");\necho \"|\";\necho ($text[9] ?? \"fallback\");\necho \"|\";\necho strtoupper($items[2] ?? \"x\");\n";

const VALUE_OFFSET_NULL_COALESCE_ASSIGN_SOURCE: &str = "<?php\n$items = [\"kept\" => \"K\", \"nullish\" => null, 2 => \"two\"];\n$key = \"missing\";\n$items[$key] ??= \"M\";\n$items[\"nullish\"] ??= \"N\";\n$items[\"kept\"] ??= $items[\"absent\"];\necho $items[$key], \"|\", $items[\"nullish\"], \"|\", $items[\"kept\"], \"|\";\necho ($items[2] ??= \"bad\"), \"|\";\necho ($items[\"expr\"] ??= (string) 7), \"|\";\n$stored = ($items[\"stored\"] ??= $items[$key]);\necho $stored, \"|\", $items[\"stored\"], \"|\";\necho isset($items[\"absent\"]) ? 1 : 0;\n";

const ARRAY_LVALUE_NESTED_NULL_COALESCE_ASSIGN_SOURCE: &str = "<?php\n$outer = \"outer\";\n$items = [$outer => [\"kept\" => \"K\", \"nullish\" => null, \"falsey\" => false]];\n$key = \"missing\";\n$items[$outer][$key] ??= \"M\";\n$items[$outer][\"nullish\"] ??= \"N\";\n$items[$outer][\"kept\"] ??= \"bad\";\n$items[\"fresh\"][\"created\"] ??= \"F\";\n$items[\"nullable\"] = null;\n$items[\"nullable\"][\"created\"] ??= \"NP\";\necho $items[$outer][$key], \"|\", $items[$outer][\"nullish\"], \"|\", $items[$outer][\"kept\"], \"|\";\necho $items[\"fresh\"][\"created\"], \"|\", $items[\"nullable\"][\"created\"], \"|\";\necho ($items[\"expr\"][\"slot\"] ??= (string) 7), \"|\";\n$stored = ($items[$outer][\"stored\"] ??= $items[\"fresh\"][\"created\"]);\necho $stored, \"|\", $items[$outer][\"stored\"], \"|\";\n$items[$outer][\"falsey\"] ??= \"wrong\";\necho empty($items[$outer][\"falsey\"]) ? 1 : 0;\n";

const REQUEST_SUPERGLOBAL_ROOT_SOURCE: &str = concat!(
    "<?php\n",
    "echo empty($_GET);\n",
    "echo \"|\";\n",
    "echo isset($_POST, $_COOKIE);\n",
    "echo \"|\";\n",
    "echo gettype($_FILES);\n",
    "echo \"|\";\n",
    "echo $_SERVER;\n",
);

const GLOBALS_SNAPSHOT_SOURCE: &str = concat!(
    "<?php\n",
    "$alpha = \"A\";\n",
    "$count = 2;\n",
    "$bag = [\"slot\" => \"B\"];\n",
    "$copy = $GLOBALS;\n",
    "echo $copy[\"alpha\"];\n",
    "echo \"|\";\n",
    "echo $copy[\"count\"];\n",
    "echo \"|\";\n",
    "echo gettype($copy[\"bag\"]);\n",
    "echo \"|\";\n",
    "echo gettype($GLOBALS);\n",
);

const GLOBALS_SYMBOL_PATH_SOURCE: &str = concat!(
    "<?php\n",
    "$alpha = \"A\";\n",
    "$bag = [\"slot\" => \"B\", \"empty\" => \"\"];\n",
    "$key = \"alpha\";\n",
    "$slot = \"slot\";\n",
    "$absent_key = \"absent\";\n",
    "echo $GLOBALS[$key];\n",
    "echo \"|\";\n",
    "echo $GLOBALS[\"bag\"][$slot];\n",
    "echo \"|\";\n",
    "echo isset($GLOBALS[$key]) ? 1 : 0;\n",
    "echo isset($GLOBALS[$absent_key]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($GLOBALS[\"bag\"][\"empty\"]) ? 1 : 0;\n",
    "echo empty($GLOBALS[\"bag\"][\"slot\"]) ? 1 : 0;\n",
);

const GLOBALS_SYMBOL_PATH_WRITE_SOURCE: &str = concat!(
    "<?php\n",
    "$key = \"alpha\";\n",
    "$slot = \"slot\";\n",
    "$GLOBALS[$key] = strtoupper(\"a\");\n",
    "$GLOBALS[\"bag\"][$slot] = \"B\";\n",
    "echo $alpha;\n",
    "echo \"|\";\n",
    "echo $GLOBALS[\"bag\"][$slot];\n",
    "echo \"|\";\n",
    "$alpha = \"C\";\n",
    "echo $GLOBALS[$key];\n",
    "echo \"|\";\n",
    "echo isset($alpha) ? 1 : 0;\n",
    "echo empty($missing) ? 1 : 0;\n",
);

const GLOBALS_SYMBOL_PATH_UNSET_SOURCE: &str = concat!(
    "<?php\n",
    "$alpha = \"A\";\n",
    "$bag = [\"slot\" => \"B\", \"keep\" => \"K\"];\n",
    "$key = \"alpha\";\n",
    "$slot = \"slot\";\n",
    "$absent_key = \"absent\";\n",
    "unset($GLOBALS[$key], $GLOBALS[\"bag\"][$slot], $GLOBALS[$absent_key][$slot]);\n",
    "echo isset($alpha) ? 1 : 0;\n",
    "echo empty($alpha) ? 1 : 0;\n",
    "echo isset($GLOBALS[\"bag\"][$slot]) ? 1 : 0;\n",
    "echo $GLOBALS[\"bag\"][\"keep\"];\n",
    "$GLOBALS[$key] = \"C\";\n",
    "echo $alpha;\n",
);

const GLOBALS_SYMBOL_PATH_APPEND_SOURCE: &str = concat!(
    "<?php\n",
    "$key = \"bag\";\n",
    "$slot = \"leaf\";\n",
    "$bag = [\"existing\" => \"E\"];\n",
    "$GLOBALS[$key][] = \"A\";\n",
    "$GLOBALS[$key][\"nested\"][][$slot] = \"B\";\n",
    "$GLOBALS[\"missing\"][\"items\"][] = \"M\";\n",
    "echo $GLOBALS[$key][0];\n",
    "echo \"|\";\n",
    "echo $GLOBALS[$key][\"nested\"][0][$slot];\n",
    "echo \"|\";\n",
    "echo $GLOBALS[\"missing\"][\"items\"][0];\n",
    "echo \"|\";\n",
    "echo ($GLOBALS[$key][] = strtoupper(\"c\"));\n",
    "echo \"|\";\n",
    "echo $GLOBALS[$key][1];\n",
);

const GLOBALS_ROOT_APPEND_REJECTION: &str = "Cannot append to $GLOBALS";

const GLOBALS_DIRECT_ROOT_APPEND_VALUE_SOURCE: &str = concat!(
    "<?php\n",
    "$first = \"A\";\n",
    "$GLOBALS[] = strtoupper(\"b\");\n",
    "echo \"after\";\n",
);

const GLOBALS_DIRECT_ROOT_APPEND_EXPR_SOURCE: &str =
    concat!("<?php\n", "echo ($GLOBALS[] = \"D\");\n",);

const GLOBALS_DIRECT_ROOT_APPEND_REFERENCE_TARGET_SOURCE: &str = concat!(
    "<?php\n",
    "$source = \"A\";\n",
    "$GLOBALS[] =& $source;\n",
    "echo \"after\";\n",
);

const GLOBALS_DIRECT_ROOT_APPEND_REFERENCE_SOURCE: &str =
    concat!("<?php\n", "$alias =& $GLOBALS[];\n", "echo \"after\";\n",);

const SYMBOL_REFERENCE_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$source = \"A\";\n",
    "$alias =& $source;\n",
    "$alias = \"B\";\n",
    "echo $source;\n",
    "echo \"|\";\n",
    "$items = [\"outer\" => [\"slot\" => \"C\"]];\n",
    "$key = \"slot\";\n",
    "$slot =& $items[\"outer\"][$key];\n",
    "$slot = \"D\";\n",
    "$again =& $items[\"outer\"][$key];\n",
    "echo $again;\n",
    "echo \"|\";\n",
    "$copy =& $source;\n",
    "$items[\"copy\"] =& $copy;\n",
    "$source = \"E\";\n",
    "$copyAgain =& $items[\"copy\"];\n",
    "echo $copyAgain;\n",
    "echo \"|\";\n",
    "$append =& $items[\"list\"][];\n",
    "$append = \"F\";\n",
    "$appendAgain =& $items[\"list\"][0];\n",
    "echo $appendAgain;\n",
    "echo \"|\";\n",
    "$targetList = [];\n",
    "$targetList[] =& $copy;\n",
    "$source = \"G\";\n",
    "$targetAgain =& $targetList[0];\n",
    "echo $targetAgain;\n",
);

const SYMBOL_REFERENCE_ARRAY_LVALUE_OWNER_SOURCE: &str = concat!(
    "<?php\n",
    "$root = [\"keep\" => \"old\"];\n",
    "$alias =& $root;\n",
    "$root[\"keep\"] = \"new\";\n",
    "$alias[\"nested\"][\"leaf\"] = \"via-alias\";\n",
    "$alias[\"nested\"][] = \"appended\";\n",
    "unset($root[\"keep\"]);\n",
    "echo empty($alias[\"keep\"]);\n",
    "echo \"|\";\n",
    "echo $root[\"nested\"][\"leaf\"];\n",
    "echo \"|\";\n",
    "echo $alias[\"nested\"][0];\n",
);

const GLOBALS_SYMBOL_REFERENCE_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$source = \"A\";\n",
    "$GLOBALS[\"alpha\"] =& $source;\n",
    "$source = \"B\";\n",
    "echo $alpha;\n",
    "echo \"|\";\n",
    "$bag = [\"outer\" => [\"slot\" => \"C\"]];\n",
    "$key = \"slot\";\n",
    "$slot =& $GLOBALS[\"bag\"][\"outer\"][$key];\n",
    "$slot = \"D\";\n",
    "echo $GLOBALS[\"bag\"][\"outer\"][\"slot\"];\n",
    "echo \"|\";\n",
    "$target = \"E\";\n",
    "$GLOBALS[\"refs\"][] =& $target;\n",
    "$target = \"F\";\n",
    "$again =& $GLOBALS[\"refs\"][0];\n",
    "echo $again;\n",
    "echo \"|\";\n",
    "$list = [];\n",
    "$list[] =& $GLOBALS[\"bag\"][\"outer\"][$key];\n",
    "$slot = \"G\";\n",
    "$listAlias =& $list[0];\n",
    "echo $listAlias;\n",
    "echo \"|\";\n",
    "$fromAppend =& $GLOBALS[\"refs\"][];\n",
    "$fromAppend = \"H\";\n",
    "echo $GLOBALS[\"refs\"][1];\n",
);

const GLOBALS_DYNAMIC_REFERENCE_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$getRoot = \"_GET\";\n",
    "$postRoot = \"_POST\";\n",
    "$plainRoot = \"plain\";\n",
    "$slot = \"slot\";\n",
    "$source = \"A\";\n",
    "$GLOBALS[$getRoot] =& $source;\n",
    "$source = [\"name\" => \"Ada\"];\n",
    "echo $_GET[\"name\"];\n",
    "echo \"|\";\n",
    "$_POST[\"box\"][$slot] = \"B\";\n",
    "$alias =& $GLOBALS[$postRoot][\"box\"][$slot];\n",
    "$alias = \"C\";\n",
    "echo $_POST[\"box\"][\"slot\"];\n",
    "echo \"|\";\n",
    "$target = \"D\";\n",
    "$GLOBALS[$plainRoot] =& $target;\n",
    "$target = \"E\";\n",
    "echo $plain;\n",
    "echo \"|\";\n",
    "$copy =& $GLOBALS[$plainRoot];\n",
    "$copy = \"F\";\n",
    "echo $target;\n",
    "echo \"|\";\n",
    "$cookieRoot = \"_COOKIE\";\n",
    "$GLOBALS[$cookieRoot][\"leaf\"] =& $copy;\n",
    "$copy = \"G\";\n",
    "echo $_COOKIE[\"leaf\"];\n",
);

const REQUEST_SUPERGLOBAL_ROOT_REFERENCE_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$source = \"A\";\n",
    "$_GET =& $source;\n",
    "$source = \"B\";\n",
    "echo $_GET;\n",
    "echo \"|\";\n",
    "$items = [\"outer\" => [\"slot\" => \"C\"]];\n",
    "$key = \"slot\";\n",
    "$_POST =& $items[\"outer\"][$key];\n",
    "echo $_POST;\n",
    "echo \"|\";\n",
    "$appendList = [];\n",
    "$_COOKIE =& $appendList[];\n",
    "echo gettype($_COOKIE);\n",
    "echo \"|\";\n",
    "$bag = [\"id\" => \"R\"];\n",
    "$_REQUEST =& $bag;\n",
    "echo $_REQUEST[\"id\"];\n",
    "echo \"|\";\n",
    "$bag = [\"id\" => \"S\"];\n",
    "echo $_REQUEST[\"id\"];\n",
);

const REQUEST_SUPERGLOBAL_ROOT_REFERENCE_SOURCE_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$_GET[\"id\"] = \"A\";\n",
    "$alias =& $_GET;\n",
    "$alias = [\"id\" => \"B\"];\n",
    "echo $_GET[\"id\"];\n",
    "echo \"|\";\n",
    "$_POST = \"C\";\n",
    "$items = [];\n",
    "$key = \"post\";\n",
    "$items[$key] =& $_POST;\n",
    "$postAlias =& $items[$key];\n",
    "$postAlias = \"D\";\n",
    "echo $_POST;\n",
    "echo \"|\";\n",
    "$_COOKIE = \"E\";\n",
    "$list = [];\n",
    "$list[] =& $_COOKIE;\n",
    "$cookieAlias =& $list[0];\n",
    "$cookieAlias = \"F\";\n",
    "echo $_COOKIE;\n",
    "echo \"|\";\n",
    "$_REQUEST = \"R\";\n",
    "$requestAlias =& $_REQUEST;\n",
    "$requestAlias = \"S\";\n",
    "echo $_REQUEST;\n",
);

const REQUEST_SUPERGLOBAL_ROOT_TO_ROOT_REFERENCE_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$_GET[\"id\"] = \"A\";\n",
    "$_POST =& $_GET;\n",
    "$postAlias =& $_POST;\n",
    "$postAlias = [\"id\" => \"B\"];\n",
    "echo $_GET[\"id\"];\n",
    "echo \"|\";\n",
    "echo $_POST[\"id\"];\n",
    "echo \"|\";\n",
    "$_COOKIE = \"C\";\n",
    "$_REQUEST =& $_COOKIE;\n",
    "$requestAlias =& $_REQUEST;\n",
    "$requestAlias = \"D\";\n",
    "echo $_COOKIE;\n",
    "echo \"|\";\n",
    "echo $_REQUEST;\n",
);

const REQUEST_SUPERGLOBAL_KEYED_REFERENCE_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$key = \"id\";\n",
    "$source = \"A\";\n",
    "$_GET[$key] =& $source;\n",
    "$source = \"B\";\n",
    "echo $_GET[\"id\"];\n",
    "echo \"|\";\n",
    "$items = [\"outer\" => [\"slot\" => \"C\"]];\n",
    "$slot = \"slot\";\n",
    "$_POST[42] =& $items[\"outer\"][$slot];\n",
    "$postAlias =& $_POST[42];\n",
    "$postAlias = \"D\";\n",
    "echo $_POST[\"42\"];\n",
    "echo \"|\";\n",
    "$_REQUEST[\"bag\"] = \"R\";\n",
    "$alias =& $_REQUEST[\"bag\"];\n",
    "$alias = \"S\";\n",
    "echo $_REQUEST[\"bag\"];\n",
    "echo \"|\";\n",
    "$list = [];\n",
    "$list[] =& $_COOKIE[false];\n",
    "$cookieAlias =& $list[0];\n",
    "$cookieAlias = \"T\";\n",
    "echo $_COOKIE[0];\n",
);

const REQUEST_SUPERGLOBAL_KEYED_TO_KEYED_REFERENCE_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$key = \"id\";\n",
    "$_GET[$key] = \"A\";\n",
    "$_POST[\"alias\"] =& $_GET[$key];\n",
    "$postAlias =& $_POST[\"alias\"];\n",
    "$postAlias = \"B\";\n",
    "echo $_GET[\"id\"];\n",
    "echo \"|\";\n",
    "echo $_POST[\"alias\"];\n",
    "echo \"|\";\n",
    "$_COOKIE[false] = \"C\";\n",
    "$_REQUEST[42] =& $_COOKIE[false];\n",
    "$requestAlias =& $_REQUEST[42];\n",
    "$requestAlias = \"D\";\n",
    "echo $_COOKIE[0];\n",
    "echo \"|\";\n",
    "echo $_REQUEST[\"42\"];\n",
);

const REQUEST_SUPERGLOBAL_PATH_REFERENCE_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$key = \"slot\";\n",
    "$source = \"A\";\n",
    "$_GET[\"outer\"][$key] =& $source;\n",
    "$source = \"B\";\n",
    "echo $_GET[\"outer\"][\"slot\"];\n",
    "echo \"|\";\n",
    "$_POST[\"box\"][\"leaf\"] = \"C\";\n",
    "$postAlias =& $_POST[\"box\"][\"leaf\"];\n",
    "$postAlias = \"D\";\n",
    "echo $_POST[\"box\"][\"leaf\"];\n",
    "echo \"|\";\n",
    "$_COOKIE[\"copy\"][\"leaf\"] =& $_POST[\"box\"][\"leaf\"];\n",
    "$postAlias = \"E\";\n",
    "echo $_COOKIE[\"copy\"][\"leaf\"];\n",
    "echo \"|\";\n",
    "$target[\"nested\"][\"leaf\"] =& $_COOKIE[\"copy\"][\"leaf\"];\n",
    "$targetAlias =& $target[\"nested\"][\"leaf\"];\n",
    "$postAlias = \"F\";\n",
    "echo $targetAlias;\n",
    "echo \"|\";\n",
    "$missingAlias =& $_REQUEST[\"new\"][\"leaf\"];\n",
    "$missingAlias = \"G\";\n",
    "echo $_REQUEST[\"new\"][\"leaf\"];\n",
);

const REQUEST_SUPERGLOBAL_APPEND_REFERENCE_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$source = \"A\";\n",
    "$_GET[] =& $source;\n",
    "$source = \"B\";\n",
    "echo $_GET[0];\n",
    "echo \"|\";\n",
    "$alias =& $_POST[];\n",
    "$alias = \"C\";\n",
    "echo $_POST[0];\n",
    "echo \"|\";\n",
    "$key = \"items\";\n",
    "$nested = \"D\";\n",
    "$_REQUEST[$key][] =& $nested;\n",
    "$nested = \"E\";\n",
    "echo $_REQUEST[\"items\"][0];\n",
    "echo \"|\";\n",
    "$pathAlias =& $_COOKIE[$key][];\n",
    "$pathAlias = \"F\";\n",
    "echo $_COOKIE[\"items\"][0];\n",
);

const ROOT_SYMBOL_UNDEFINED_READ_SOURCE: &str = concat!(
    "<?php\n",
    "$copy = $third;\n",
    "echo gettype($copy);\n",
    "echo \"|\";\n",
    "echo $missing;\n",
    "echo \"A\";\n",
    "print $other;\n",
    "echo \"B\";\n",
    "$discarded;\n",
    "$after = \"C\";\n",
    "echo $after;\n",
);

const DIRECT_SYMBOL_UNSET_SOURCE: &str = concat!(
    "<?php\n",
    "$first = \"A\";\n",
    "$second = \"B\";\n",
    "unset($first);\n",
    "echo isset($first);\n",
    "echo \"|\";\n",
    "echo empty($first);\n",
    "echo \"|\";\n",
    "echo $first;\n",
    "echo \"|\";\n",
    "unset($second, $missing);\n",
    "echo isset($second);\n",
    "$second = \"C\";\n",
    "echo \"|\";\n",
    "echo $second;\n",
);

const MIXED_UNSET_TARGETS_SOURCE: &str = concat!(
    "<?php\n",
    "$root = \"R\";\n",
    "$items = [\"drop\" => \"D\", \"keep\" => \"K\", \"nested\" => [\"leaf\" => \"L\", \"stay\" => \"S\"]];\n",
    "$bag = [\"slot\" => \"B\", \"keep\" => \"G\"];\n",
    "$drop = \"drop\";\n",
    "$leaf = \"leaf\";\n",
    "unset($root, $items[$drop], $items[\"nested\"][$leaf], $GLOBALS[\"bag\"][\"slot\"], $missing);\n",
    "echo isset($root) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($root) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo isset($items[$drop]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo isset($items[\"keep\"]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo isset($items[\"nested\"][$leaf]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo isset($items[\"nested\"][\"stay\"]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo isset($GLOBALS[\"bag\"][\"slot\"]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo $bag[\"keep\"];\n",
    "$root = \"N\";\n",
    "echo \"|\";\n",
    "echo $root;\n",
);

const REQUEST_SUPERGLOBAL_ROOT_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$_GET = \"alpha\";\n",
    "echo $_GET;\n",
    "echo \"|\";\n",
    "$_POST = 42;\n",
    "echo $_POST;\n",
    "echo \"|\";\n",
    "$_COOKIE = false;\n",
    "echo empty($_COOKIE);\n",
    "echo \"|\";\n",
    "$_REQUEST = [\"name\" => \"Ada\"];\n",
    "echo gettype($_REQUEST);\n",
    "echo \"|\";\n",
    "$_SERVER = strtoupper(\"srv\");\n",
    "echo $_SERVER;\n",
);

const REQUEST_SUPERGLOBAL_ROOT_UNSET_SOURCE: &str = concat!(
    "<?php\n",
    "$_GET = \"alpha\";\n",
    "echo isset($_GET) ? 1 : 0;\n",
    "echo \"|\";\n",
    "unset($_GET);\n",
    "echo isset($_GET) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($_GET) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo $_GET;\n",
    "echo \"|\";\n",
    "$_GET[\"name\"] = \"Ada\";\n",
    "echo $_GET[\"name\"];\n",
    "echo \"|\";\n",
    "echo gettype($_GET);\n",
);

const REQUEST_SUPERGLOBAL_REFERENCE_BACKED_ROOT_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$slot = \"seed\";\n",
    "$_GET =& $slot;\n",
    "$_GET = strtoupper(\"alpha\");\n",
    "echo $slot;\n",
    "echo \"|\";\n",
    "echo $_GET;\n",
    "echo \"|\";\n",
    "$bag = [\"id\" => \"old\"];\n",
    "$_POST =& $bag;\n",
    "$_POST = [\"id\" => \"new\"];\n",
    "echo gettype($bag);\n",
    "echo \"|\";\n",
    "echo $_POST[\"id\"];\n",
    "echo \"|\";\n",
    "$nil = \"seed\";\n",
    "$_COOKIE =& $nil;\n",
    "$_COOKIE = null;\n",
    "echo gettype($nil);\n",
    "echo \"|\";\n",
    "echo gettype($_COOKIE);\n",
);

const REQUEST_SUPERGLOBAL_REFERENCE_BACKED_KEYED_MUTATION_SOURCE: &str = concat!(
    "<?php\n",
    "$bag = [\"seed\" => \"old\"];\n",
    "$_GET =& $bag;\n",
    "$_POST =& $_GET;\n",
    "$_COOKIE =& $_GET;\n",
    "$_GET[\"name\"] = \"Ada\";\n",
    "echo gettype($bag);\n",
    "echo \"|\";\n",
    "echo $_POST[\"name\"];\n",
    "echo \"|\";\n",
    "$_POST[\"profile\"][\"id\"] = strtoupper(\"b\");\n",
    "echo $_GET[\"profile\"][\"id\"];\n",
    "echo \"|\";\n",
    "unset($_COOKIE[\"seed\"]);\n",
    "echo isset($_GET[\"seed\"]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "$_COOKIE[\"items\"][] = \"tail\";\n",
    "echo $_POST[\"items\"][0];\n",
);

const REQUEST_SUPERGLOBAL_REFERENCE_BACKED_KEYED_REFERENCE_SOURCE: &str = concat!(
    "<?php\n",
    "$bag = [\"seed\" => \"old\"];\n",
    "$_GET =& $bag;\n",
    "$source = \"A\";\n",
    "$_GET[\"name\"] =& $source;\n",
    "$source = \"B\";\n",
    "echo $bag[\"name\"];\n",
    "echo \"|\";\n",
    "$alias =& $_GET[\"seed\"];\n",
    "$alias = \"S\";\n",
    "echo $bag[\"seed\"];\n",
    "echo \"|\";\n",
    "$_POST =& $_GET;\n",
    "$target = \"C\";\n",
    "$_POST[\"other\"] =& $target;\n",
    "$target = \"D\";\n",
    "echo $_GET[\"other\"];\n",
    "echo \"|\";\n",
    "echo $bag[\"other\"];\n",
);

const REQUEST_SUPERGLOBAL_KEYED_STORAGE_SOURCE: &str = concat!(
    "<?php\n",
    "$key = \"name\";\n",
    "$_GET[$key] = \"Ada\";\n",
    "echo $_GET[\"name\"];\n",
    "echo \"|\";\n",
    "echo isset($_GET[$key]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "unset($_GET[$key]);\n",
    "echo isset($_GET[\"name\"]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "$_POST[42] = strtoupper(\"answer\");\n",
    "echo $_POST[\"42\"];\n",
    "echo \"|\";\n",
    "$_COOKIE[false] = null;\n",
    "echo isset($_COOKIE[0]) ? 1 : 0;\n",
);

const REQUEST_SUPERGLOBAL_KEYED_EMPTY_SOURCE: &str = concat!(
    "<?php\n",
    "$zero = \"zero\";\n",
    "$_GET[$zero] = \"0\";\n",
    "$_POST[\"name\"] = \"Ada\";\n",
    "$_COOKIE[false] = \"\";\n",
    "$_REQUEST[true] = [];\n",
    "echo empty($_GET[$zero]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($_POST[\"name\"]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($_SERVER[\"missing\"]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($_COOKIE[0]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($_REQUEST[1]) ? 1 : 0;\n",
);

const REQUEST_SUPERGLOBAL_PATH_MUTATION_SOURCE: &str = concat!(
    "<?php\n",
    "$outer = \"outer\";\n",
    "$inner = \"inner\";\n",
    "$_GET[$outer][$inner] = \"G\";\n",
    "echo empty($_GET[$outer]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "$_POST[\"box\"][0] = strtoupper(\"p\");\n",
    "echo empty($_POST[\"box\"]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "$_COOKIE[\"drop\"][\"leaf\"] = \"gone\";\n",
    "unset($_GET[$outer][$inner], $_COOKIE[\"drop\"][\"leaf\"]);\n",
    "echo empty($_GET[$outer]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($_COOKIE[\"drop\"]) ? 1 : 0;\n",
);

const REQUEST_SUPERGLOBAL_PATH_APPEND_SOURCE: &str = concat!(
    "<?php\n",
    "$slot = \"items\";\n",
    "$nested = \"nested\";\n",
    "$_GET[] = \"A\";\n",
    "$_GET[$slot][] = \"B\";\n",
    "$_POST[$slot][$nested][] = strtoupper(\"c\");\n",
    "echo $_GET[0];\n",
    "echo \"|\";\n",
    "echo $_GET[$slot][0];\n",
    "echo \"|\";\n",
    "echo $_POST[$slot][$nested][0];\n",
    "echo \"|\";\n",
    "echo ($_COOKIE[$slot][] = \"D\");\n",
    "echo \"|\";\n",
    "echo $_COOKIE[$slot][0];\n",
);

const REQUEST_SUPERGLOBAL_APPEND_SUFFIX_SOURCE: &str = concat!(
    "<?php\n",
    "$slot = \"items\";\n",
    "$leaf = \"leaf\";\n",
    "$inner = \"inner\";\n",
    "$_GET[$slot][][$leaf] = \"G\";\n",
    "$_POST[][$leaf] = strtoupper(\"p\");\n",
    "$GLOBALS[\"_COOKIE\"][$slot][][$inner] = \"C\";\n",
    "$_GET[$slot][][$inner][$leaf] = \"N\";\n",
    "echo $_GET[$slot][0][$leaf];\n",
    "echo \"|\";\n",
    "echo $_POST[0][$leaf];\n",
    "echo \"|\";\n",
    "echo $_COOKIE[$slot][0][$inner];\n",
    "echo \"|\";\n",
    "echo $_GET[$slot][1][$inner][$leaf];\n",
    "echo \"|\";\n",
    "echo ($_REQUEST[$slot][][$leaf] = strrev(\"zyx\"));\n",
    "echo \"|\";\n",
    "echo $_REQUEST[$slot][0][$leaf];\n",
);

const REQUEST_SUPERGLOBAL_PATH_READ_PROBE_SOURCE: &str = concat!(
    "<?php\n",
    "$outer = \"outer\";\n",
    "$inner = \"inner\";\n",
    "$_GET = [$outer => [$inner => \"G\", \"zero\" => \"0\"]];\n",
    "$_POST = [\"box\" => [0 => strtoupper(\"p\"), \"list\" => [\"leaf\" => \"L\"]]];\n",
    "$_COOKIE = [\"flags\" => [0 => null, 1 => \"yes\"]];\n",
    "echo $_GET[$outer][$inner];\n",
    "echo \"|\";\n",
    "echo strtoupper($_POST[\"box\"][0]);\n",
    "echo \"|\";\n",
    "echo isset($_GET[$outer][$inner]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo isset($_GET[$outer][\"missing\"], $_POST[\"box\"][\"list\"][\"leaf\"]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($_GET[$outer][\"zero\"]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($_COOKIE[\"flags\"][0]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($_POST[\"box\"][\"list\"][\"leaf\"]) ? 1 : 0;\n",
);

const REQUEST_SUPERGLOBAL_ASSIGNMENT_EXPRESSION_SOURCE: &str = concat!(
    "<?php\n",
    "$key = \"name\";\n",
    "$outer = \"box\";\n",
    "$inner = \"leaf\";\n",
    "echo ($_GET[$key] = strtoupper(\"ada\"));\n",
    "echo \"|\";\n",
    "$stored = ($_POST[$outer][$inner] = strrev(\"24\"));\n",
    "echo $stored;\n",
    "echo \"|\";\n",
    "echo $_POST[$outer][$inner];\n",
    "echo \"|\";\n",
    "echo gettype($_COOKIE = [\"root\" => \"C\"]);\n",
    "echo \"|\";\n",
    "echo $_GET[\"name\"];\n",
    "echo \"|\";\n",
    "echo $_COOKIE[\"root\"];\n",
);

const REQUEST_SUPERGLOBAL_NULL_COALESCE_SOURCE: &str = concat!(
    "<?php\n",
    "$key = \"name\";\n",
    "$outer = \"box\";\n",
    "$inner = \"leaf\";\n",
    "$_GET[$key] = \"Ada\";\n",
    "$_POST[$outer][$inner] = strtoupper(\"p\");\n",
    "$_COOKIE[\"empty\"] = null;\n",
    "echo ($_GET[$key] ?? $never);\n",
    "echo \"|\";\n",
    "echo ($_GET[\"absent\"] ?? \"fallback\");\n",
    "echo \"|\";\n",
    "echo ($_COOKIE[\"empty\"] ?? \"null-fallback\");\n",
    "echo \"|\";\n",
    "echo strtoupper($_POST[$outer][$inner] ?? \"x\");\n",
    "echo \"|\";\n",
    "$stored = $_POST[$outer][\"absent\"] ?? strrev(\"zyx\");\n",
    "echo $stored;\n",
    "echo \"|\";\n",
    "echo gettype($_REQUEST ?? $root_never);\n",
);

const GLOBALS_REQUEST_ALIAS_SOURCE: &str = concat!(
    "<?php\n",
    "$key = \"name\";\n",
    "$outer = \"box\";\n",
    "$inner = \"leaf\";\n",
    "$GLOBALS[\"_GET\"] = [\"name\" => \"Ada\", \"empty\" => \"\"];\n",
    "echo $_GET[$key];\n",
    "echo \"|\";\n",
    "$_POST[$outer][$inner] = \"P\";\n",
    "echo $GLOBALS[\"_POST\"][$outer][$inner];\n",
    "echo \"|\";\n",
    "$GLOBALS[\"_COOKIE\"][$key] = strtoupper(\"c\");\n",
    "echo $_COOKIE[$key];\n",
    "echo \"|\";\n",
    "$GLOBALS[\"_REQUEST\"][$outer][$inner] = \"R\";\n",
    "echo $GLOBALS[\"_REQUEST\"][$outer][$inner];\n",
    "echo \"|\";\n",
    "echo isset($GLOBALS[\"_GET\"][$key]) ? 1 : 0;\n",
    "echo empty($GLOBALS[\"_GET\"][\"empty\"]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "unset($GLOBALS[\"_COOKIE\"][$key]);\n",
    "echo isset($_COOKIE[$key]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "$GLOBALS[\"_POST\"][$outer][] = \"A\";\n",
    "echo $_POST[$outer][0];\n",
    "echo \"|\";\n",
    "echo ($GLOBALS[\"_GET\"][\"expr\"] = strrev(\"zyx\"));\n",
    "echo \"|\";\n",
    "echo $_GET[\"expr\"];\n",
);

const GLOBALS_SELF_REQUEST_ALIAS_SOURCE: &str = concat!(
    "<?php\n",
    "$key = \"name\";\n",
    "$outer = \"box\";\n",
    "$inner = \"leaf\";\n",
    "$GLOBALS[\"GLOBALS\"][\"_GET\"] = [\"name\" => \"Ada\", \"empty\" => \"\"];\n",
    "echo $_GET[$key];\n",
    "echo \"|\";\n",
    "$GLOBALS[\"GLO\" . \"BALS\"][\"_POST\"][$outer][$inner] = \"P\";\n",
    "echo $_POST[$outer][$inner];\n",
    "echo \"|\";\n",
    "$GLOBALS[\"GLOBALS\"][\"GLOBALS\"][\"_COOKIE\"][$key] = strtoupper(\"c\");\n",
    "echo $_COOKIE[$key];\n",
    "echo \"|\";\n",
    "unset($GLOBALS[\"GLOBALS\"][\"_COOKIE\"][$key]);\n",
    "echo isset($_COOKIE[$key]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "$GLOBALS[\"GLOBALS\"][\"_POST\"][$outer][] = \"A\";\n",
    "echo $_POST[$outer][0];\n",
    "echo \"|\";\n",
    "echo isset($GLOBALS[\"GLOBALS\"][\"_GET\"][$key]) ? 1 : 0;\n",
    "echo empty($GLOBALS[\"GLOBALS\"][\"_GET\"][\"empty\"]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo gettype($GLOBALS[\"GLOBALS\"][\"_REQUEST\"]);\n",
);

const GLOBALS_DYNAMIC_REQUEST_ROOT_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "$getRoot = \"_GET\";\n",
    "$postRoot = \"_POST\";\n",
    "$plainRoot = \"ordinary\";\n",
    "$GLOBALS[$getRoot] = [\"name\" => \"Ada\"];\n",
    "echo $_GET[\"name\"];\n",
    "echo \"|\";\n",
    "echo ($GLOBALS[$postRoot] = strtoupper(\"p\"));\n",
    "echo \"|\";\n",
    "echo $_POST;\n",
    "echo \"|\";\n",
    "$GLOBALS[$plainRoot] = \"S\";\n",
    "echo $ordinary;\n",
    "echo \"|\";\n",
    "$getRoot = \"_COOKIE\";\n",
    "$GLOBALS[$getRoot] = false;\n",
    "echo empty($_COOKIE) ? 1 : 0;\n",
);

const GLOBALS_DYNAMIC_REQUEST_ROOT_READ_SOURCE: &str = concat!(
    "<?php\n",
    "$getRoot = \"_GET\";\n",
    "$postRoot = \"_POST\";\n",
    "$cookieRoot = \"_COOKIE\";\n",
    "$plainRoot = \"ordinary\";\n",
    "$missingRoot = \"missing\";\n",
    "$_GET = \"Ada\";\n",
    "$_POST = \"\";\n",
    "$_COOKIE = false;\n",
    "$ordinary = \"S\";\n",
    "echo $GLOBALS[$getRoot];\n",
    "echo \"|\";\n",
    "echo isset($GLOBALS[$postRoot]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($GLOBALS[$postRoot]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($GLOBALS[$cookieRoot]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo $GLOBALS[$plainRoot];\n",
    "echo \"|\";\n",
    "echo isset($GLOBALS[$plainRoot]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo isset($GLOBALS[$missingRoot]) ? 1 : 0;\n",
    "echo \"|\";\n",
    "echo empty($GLOBALS[$missingRoot]) ? 1 : 0;\n",
);

const NATIVE_VALUE_VARIABLE_STORAGE_SOURCE: &str = "<?php\n$items = [0 => \"seed\", \"first\" => \"q\"];\n$key = \"first\";\n$slot = $items[$key];\n$copy = $slot;\necho $slot, \"|\", $copy, \"|\";\n$upper = strtoupper($copy);\necho $upper, \"|\";\n$fallback = $items[\"missing\"] ?? \"m\";\necho $fallback, \"|\";\n$cast = (string) 42;\necho $cast, \"|\";\n$items[] = $upper;\necho $items[1];\n";

const ACTIVE_SYMBOL_ROOT_OFFSET_MUTATION_SOURCE: &str = "<?php\n$items = [0 => \"seed\", \"first\" => \"q\"];\n$box = null;\n$path = null;\n$trigger = $items[\"missing\"] ?? \"m\";\n$items[] = strtoupper($items[\"first\"]);\n$items[\"first\"] = \"r\";\n$box[] = \"B\";\n$path[\"outer\"][] = \"P\";\necho $trigger, \"|\", $items[1], \"|\", $items[\"first\"], \"|\", $items[0], \"|\", $box[0], \"|\", isset($path[\"outer\"]) ? 1 : 0;\n";

const VALUE_OFFSET_MUTATION_ARRAY_UNSET_SOURCE: &str = "<?php\n$outer = \"outer\";\n$inner = \"inner\";\n$items = [\"keep\" => \"A\", \"drop\" => \"B\", 2 => \"C\", $outer => [$inner => \"N\", \"stay\" => \"S\"]];\n$key = \"drop\";\nunset($items[$key]);\nunset($items[2]);\nunset($items[99]);\nunset($items[$outer][$inner]);\necho isset($items[\"keep\"]) ? 1 : 0;\necho \"|\";\necho isset($items[$key]) ? 1 : 0;\necho \"|\";\necho empty($items[2]) ? 1 : 0;\necho \"|\";\necho isset($items[$outer][$inner]) ? 1 : 0;\necho \"|\";\n$items[$key] = \"D\";\necho $items[$key];\n";

const VALUE_OFFSET_MUTATION_ARRAY_MULTI_UNSET_SOURCE: &str = "<?php\n$left = [\"keep\" => \"L\", \"drop\" => \"D\", 2 => \"I\"];\n$right = [0 => \"R0\", \"drop\" => \"RD\"];\n$key = \"drop\";\nunset($left[$key], $right[0], $left[2], $right[\"missing\"]);\necho isset($left[\"keep\"]) ? 1 : 0;\necho \"|\";\necho isset($left[$key]) ? 1 : 0;\necho \"|\";\necho empty($right[0]) ? 1 : 0;\necho \"|\";\necho empty($left[2]) ? 1 : 0;\necho \"|\";\necho isset($right[\"drop\"]) ? 1 : 0;\n";

#[test]
fn native_executable_c_source_routes_string_offset_isset_empty_through_bool_boundary() {
    let program = parse(STRING_OFFSET_ISSET_EMPTY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_offset_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_bool_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_operation_with_diagnostic(")
            .count(),
        6,
        "isset/empty offsets should share the runtime value-offset operation boundary:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_bool_with_diagnostic(")
            .count(),
        6,
        "offset bool results should pass through the typed native bool boundary:\n{source}"
    );
    assert!(
        body.contains(", 1, &value_offset_bool_diagnostic_"),
        "isset offsets should use the shared operation tag:\n{source}"
    );
    assert!(
        body.contains(", 2, &value_offset_bool_diagnostic_"),
        "empty offsets should use the shared operation tag:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_value_string_offset_operation_with_diagnostic"),
        "presence paths should not use the string-only offset ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_and_string_offset_presence_through_value_boundary() {
    let program = parse(VALUE_OFFSET_PRESENCE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_offset_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_offset_operation_with_diagnostic(")
            .count()
            >= 6,
        "array and string offset presence should use one value-offset ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_from_array"),
        "array subjects should be materialized through the native value carrier:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_bool_with_diagnostic"),
        "offset presence results should pass through the native bool boundary:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_value_string_offset_operation_with_diagnostic"),
        "array/string presence should not dispatch through the string-only offset ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_offset_reads_through_byte_boundary() {
    let program = parse(STRING_OFFSET_READ_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_offset_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_string_clone_bytes"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_diagnostic_report(string_offset_read_diagnostic_"),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_string_offset_operation_with_diagnostic(")
            .count()
            >= 5,
        "string-offset reads should share the runtime string-offset operation boundary:\n{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_string_clone_bytes(")
            .count()
            >= 5,
        "offset read values should materialize through the byte clone boundary:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_byte_buffer_free(string_offset_read_buffer")
            .count()
            >= 5,
        "owned string-offset read byte buffers must be cleaned up:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_diagnostic_message_stderr(string_offset_read_diagnostic_"),
        "{source}"
    );
    assert!(!source.contains("printf(\"%s\""), "{source}");
}

#[test]
fn native_executable_c_source_routes_array_offset_writes_through_value_offset_mutation_boundary() {
    let program = parse(VALUE_OFFSET_MUTATION_ARRAY_WRITE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeArrayHandle phpc_native_value_array_clone(phpc_NativeValueHandle value);"
        ),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_mutation_operation_with_diagnostic(")
            .count(),
        2,
        "direct array offset assignments should share the value-offset mutation boundary:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_array_clone(").count(),
        2,
        "array mutation results should rematerialize through the value-array clone boundary:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_from_array(array_"),
        "array subjects should enter the mutation ABI as native values:\n{source}"
    );
    assert!(
        body.contains(", 0, &array_offset_write_diagnostic_"),
        "array offset writes should use the shared write operation tag:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nested_array_writes_through_lvalue_owner_operation() {
    let program = parse(ARRAY_LVALUE_NESTED_WRITE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_array"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_array_lvalue_owner_value_operation_result(")
            .count(),
        2,
        "nested array writes should share the lvalue owner/path operation boundary:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE")
            .count(),
        2,
        "nested array writes should use the write operation family for every target:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_KEY").count() >= 4,
        "nested write targets should materialize every path key through shared path segments:\n{source}"
    );
    assert!(
        !body.contains("array_offset_write_diagnostic_"),
        "nested writes should not fall back to the direct value-offset write path:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nested_array_appends_through_lvalue_owner_operation() {
    let program = parse(ARRAY_LVALUE_NESTED_APPEND_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("PHPC_NATIVE_ARRAY_PATH_APPEND"),
        "nested append writes should declare the append path segment tag:\n{source}"
    );
    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_array_lvalue_owner_value_operation_result(")
            .count(),
        3,
        "nested append assignments should share the lvalue owner/path operation boundary:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE")
            .count(),
        3,
        "nested append assignments should use the write operation family:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_APPEND").count(),
        3,
        "each nested append target should materialize exactly one append path segment:\n{source}"
    );
    assert!(
        !body.contains("array_offset_append_value"),
        "nested appends should not fall back to the direct value-offset append path:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nested_array_assignment_expression_values_through_lvalue_owner_operation(
) {
    let program = parse(ARRAY_LVALUE_NESTED_ASSIGNMENT_EXPR_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_array_lvalue_owner_value_operation_result(")
            .count(),
        3,
        "nested assignment expressions should share the lvalue owner/path operation boundary:\n{source}"
    );
    assert_eq!(
        body.matches("phpc_NativeArrayLvalueResult array_lvalue_assign_expr_result_")
            .count(),
        3,
        "nested assignment expressions should use the assignment-expression result path:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE")
            .count(),
        3,
        "nested assignment expressions should use the write operation family:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_APPEND").count(),
        2,
        "nested keyed and append assignment expressions should materialize append path segments only for append forms:\n{source}"
    );
    assert!(
        !body.contains("array_offset_assign_expr_diagnostic_"),
        "nested assignment expressions should not fall back to the direct value-offset assignment-expression path:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nested_array_reads_through_lvalue_owner_operation() {
    let program = parse(ARRAY_LVALUE_NESTED_READ_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ"),
        "nested array reads should declare the lvalue read operation tag:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ")
            .count()
            >= 4,
        "nested array reads should use the read operation family for output, print, string-result, and value-mutation consumers:\n{source}"
    );
    assert!(
        body.matches("phpc_NativeArrayLvalueResult array_lvalue_read_result_")
            .count()
            >= 4,
        "nested array reads should share one lvalue read-result path across consumers:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_KEY").count() >= 8,
        "nested read paths should materialize every dynamic and literal key through shared path segments:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_read_key_with_diagnostic("),
        "nested reads should not reintroduce the legacy array-read bypass:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_lvalue_compound_assignments_through_read_compute_write()
{
    let program = parse(ARRAY_LVALUE_COMPOUND_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_binary_result"),
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ")
            .count()
            >= 4,
        "compound assignments should read current lvalue values through the shared read family:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE")
            .count()
            >= 4,
        "compound assignments should write computed values through the shared write family:\n{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_binary_result(").count() >= 4,
        "compound assignments should compute through the native value binary ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_NativeValueHandle array_lvalue_compound_current_")
            && body.contains("phpc_NativeValueHandle native_value_binary_"),
        "compound assignments should own current and computed native value handles:\n{source}"
    );
    assert!(
        !body.contains("assembly mutation lowering rejects"),
        "lowerable array lvalue compound assignments should not fall through the blanket mutation blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_direct_variable_compound_assignments_through_value_results() {
    let program = parse(DIRECT_VARIABLE_COMPOUND_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.matches(" = phpc_native_value_binary_result(").count() >= 5,
        "direct variable compound assignments should compute through the shared native value binary ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_clone(")
            && source.contains("phpc_native_reference_set_value(")
            && body.contains("phpc_native_symbol_table_set_value_by_path_with_diagnostic("),
        "direct variable compound assignments should store ordinary, reference-backed, and active symbol-table variables through shared owner paths:\n{source}"
    );
    assert!(
        !body.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ")
            && !body.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE"),
        "direct variable compound assignments should not use array-offset lvalue owner read/write operations:\n{source}"
    );
    assert!(
        !source.contains("assembly mutation lowering rejects"),
        "lowerable direct variable compound assignments should not fall through the blanket mutation blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_direct_variable_assignment_expressions_through_storage_owners()
{
    let program = parse(DIRECT_VARIABLE_ASSIGNMENT_EXPRESSION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("phpc_native_value_clone(")
            && body.contains("phpc_native_symbol_table_set_value_by_path_with_diagnostic("),
        "direct variable assignment expressions should store ordinary and active symbol-table variables through shared owner paths:\n{source}"
    );
    assert!(
        source.contains("phpc_native_reference_set_value("),
        "direct variable assignment expressions inside reference-backed frame variables should write through the shared reference owner path:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_free(native_value_clone_"),
        "cloned native assignment-expression result handles should remain tracked for cleanup:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "native-value assignment-expression results should remain available to expression consumers:\n{source}"
    );
    assert!(
        !body.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE"),
        "direct variable assignment expressions should not route through array-lvalue write operations:\n{source}"
    );
    assert!(
        !source.contains("assembly mutation lowering rejects"),
        "lowerable direct variable assignment expressions should not fall through the blanket mutation blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_direct_variable_assignment_expressions_from_native_results_without_prior_helpers(
) {
    let program = parse(DIRECT_VARIABLE_NATIVE_RESULT_ASSIGNMENT_EXPRESSION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_string_result_operation_with_diagnostic")
            && source.contains("phpc_native_value_string_search_result_with_diagnostic"),
        "native-result assignment-expression RHS values should route through shared value-result materializers without depending on prior helper state:\n{source}"
    );
    assert!(
        source.matches("phpc_native_value_clone(").count() >= 2
            && source.contains("phpc_native_value_free(native_value_clone_"),
        "native-result assignment expressions should clone assigned values for storage while tracking both stored and expression-result cleanup:\n{source}"
    );
    assert!(
        !source.contains("assembly mutation lowering rejects"),
        "native-result direct variable assignment expressions should not fall through the mutation blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_compound_union_through_value_result_boundary() {
    let program = parse(ARRAY_LVALUE_COMPOUND_ARRAY_UNION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "array += should route through the shared lvalue owner boundary:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_binary_result"),
        "array += should compute through the shared native binary value-result ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_array_lvalue_owner_array")
            .count()
            >= 2,
        "array union compound assignment should cover direct and nested array-offset owners:\n{source}"
    );
    assert!(
        !source.contains("assembly arithmetic lowering rejects")
            && !source.contains("assembly mutation lowering rejects"),
        "array union compound assignment should not hit arithmetic or mutation blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_cast_warnings_through_value_result_boundary() {
    let program = parse(ARRAY_CAST_VALUE_RESULT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches(" = phpc_native_value_cast_result(").count() >= 3,
        "string casts should use the shared native value-result cast carrier:\n{source}"
    );
    assert!(
        body.matches("phpc_native_array_lvalue_owner_value_operation_result")
            .count()
            >= 3,
        "assignment, null-coalescing assignment, and compound assignment values should share the lvalue value-result carrier:\n{source}"
    );
    assert!(
        body.contains(".diagnostic")
            && !body.contains(" = phpc_native_value_cast_operation_with_diagnostic("),
        "cast warnings should flow through the value-result diagnostic carrier:\n{source}"
    );
    assert!(
        !source.contains("native value cast rejects array-to-string diagnostics")
            && !source.contains("assembly cast lowering rejects"),
        "array-to-string casts should not hit the old cast blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_lvalue_increment_decrement_through_update_boundary() {
    let program = parse(ARRAY_LVALUE_INCREMENT_DECREMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UPDATE"),
        "increment/decrement should declare the lvalue update family:\n{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_RESULT_INCREMENT_DECREMENT"),
        "increment/decrement should declare the operation tag:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UPDATE")
            .count()
            >= 5,
        "statement and expression increment/decrement forms should share the update family:\n{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_ARRAY_LVALUE_INCREMENT")
            && body.contains("PHPC_NATIVE_ARRAY_LVALUE_DECREMENT")
            && body.contains("PHPC_NATIVE_ARRAY_LVALUE_POSITION_PRE")
            && body.contains("PHPC_NATIVE_ARRAY_LVALUE_POSITION_POST"),
        "increment/decrement should carry operation and result-position tags:\n{source}"
    );
    assert!(
        !body.contains(" = phpc_native_value_binary_result("),
        "increment/decrement should not be lowered as an exact +1/-1 binary expression:\n{source}"
    );
    assert!(
        !body.contains("assembly mutation lowering rejects"),
        "lowerable array lvalue increment/decrement should not fall through the blanket mutation blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_append_array_lvalue_increment_decrement_through_update_boundary(
) {
    let program = parse(ARRAY_LVALUE_APPEND_INCREMENT_DECREMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UPDATE"),
        "append increment/decrement should declare the shared lvalue update family:\n{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_RESULT_INCREMENT_DECREMENT"),
        "append increment/decrement should use the increment/decrement operation tag:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UPDATE")
            .count()
            >= 4,
        "direct and nested append increment/decrement forms should share the update family:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_APPEND").count() >= 4,
        "each append increment/decrement target should materialize one append path segment:\n{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_ARRAY_LVALUE_POSITION_PRE")
            && body.contains("PHPC_NATIVE_ARRAY_LVALUE_POSITION_POST"),
        "append increment/decrement should preserve pre/post expression-result tags:\n{source}"
    );
    assert!(
        !body.contains(" = phpc_native_value_binary_result("),
        "append increment/decrement should not be lowered as fixture-shaped +1 binary writes:\n{source}"
    );
    assert!(
        !body.contains("assembly mutation lowering rejects"),
        "lowerable append increment/decrement should not fall through the blanket mutation blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nested_array_lvalue_rmw_through_owner_paths() {
    let program = parse(ARRAY_LVALUE_NESTED_RMW_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ")
            .count()
            >= 3,
        "nested compound assignments should read current values through the shared lvalue path:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE")
            .count()
            >= 3,
        "nested compound assignments should write computed values through the shared lvalue path:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UPDATE")
            .count()
            >= 3,
        "nested increment/decrement should share the runtime update operation:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_KEY").count() >= 18,
        "nested RMW should materialize every path segment through semantic key operands:\n{source}"
    );
    assert!(
        !body.contains("assembly mutation lowering rejects"),
        "nested RMW targets should not fall through the blanket mutation blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_lvalue_rmw_owner_families_through_shared_boundary() {
    let program = parse(ARRAY_LVALUE_RMW_OWNER_BOUNDARY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_array_lvalue_owner_array"),
        "local array-handle RMW owners should use the shared array-lvalue owner boundary:\n{source}"
    );
    assert!(
        source.contains("phpc_native_array_lvalue_owner_reference_slot"),
        "reference-slot RMW owners should use the same array-lvalue owner boundary:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_array_lvalue_owner_value_operation_result")
            .count()
            >= 8,
        "RMW owner families should share the runtime value-operation ABI:\n{source}"
    );
    assert!(
        source
            .matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ")
            .count()
            >= 2
            && source
                .matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE")
                .count()
                >= 4
            && source
                .matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_ISSET")
                .count()
                >= 2
            && source
                .matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UPDATE")
                .count()
                >= 2,
        "compound, null-coalesce assignment, and increment/decrement should share the owner materialization path:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_binary_result"),
        "compound assignment should still compute through the native binary value ABI:\n{source}"
    );
    assert!(
        !source.contains("assembly mutation lowering rejects")
            && !source.contains("assembly global-declaration lowering rejects"),
        "lowerable RMW owner families should not fall through backend blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_offset_appends_through_value_offset_mutation_boundary() {
    let program = parse(VALUE_OFFSET_MUTATION_ARRAY_APPEND_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeArrayHandle phpc_native_value_array_clone(phpc_NativeValueHandle value);"
        ),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_mutation_operation_with_diagnostic(")
            .count(),
        2,
        "direct array appends should share the value-offset mutation boundary:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_array_clone(").count(),
        2,
        "array append results should rematerialize through the value-array clone boundary:\n{source}"
    );
    assert!(
        body.contains(", 1, &array_offset_append_diagnostic_"),
        "array appends should use the shared append operation tag:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_from_array(array_"),
        "array subjects should enter the append mutation ABI as native values:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_append_value_with_diagnostic("),
        "direct array append assignments should not bypass the value-offset mutation ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_value_appends_through_mutation_storage() {
    let program = parse(VALUE_OFFSET_MUTATION_VALUE_APPEND_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.contains(", 1, &value_offset_append_diagnostic_"),
        "value append assignments should use the shared append operation tag:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_mutation_operation_with_diagnostic(")
            .count(),
        3,
        "null, false, and scalar value appends should share one mutation ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_clone(value_offset_append_value_to_clone_"),
        "value append assignments should store the selected runtime value through native value storage:\n{source}"
    );
    assert!(
        !body.contains("assembly array lowering rejects"),
        "value append assignments should not fall through to the blanket array rejection:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_unassigned_and_value_offset_writes_through_mutation_storage() {
    let program = parse(VALUE_OFFSET_MUTATION_VALUE_WRITE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.contains(", 0, &value_offset_write_diagnostic_"),
        "value writes should use the shared write operation tag:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_mutation_operation_with_diagnostic(")
            .count(),
        4,
        "missing, null, false, and scalar keyed writes should share one mutation ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_clone(value_offset_write_value_to_clone_"),
        "value writes should store the selected runtime value through native value storage:\n{source}"
    );
    assert!(
        !body.contains("assembly array lowering rejects"),
        "value writes should not fall through to the blanket array rejection:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nested_value_writes_through_path_boundaries() {
    let program = parse(VALUE_OFFSET_PATH_MUTATION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_offset_path_write_with_diagnostic"
        ) && source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_offset_path_append_with_diagnostic"
        ),
        "generated C should declare the path mutation ABI:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_path_write_with_diagnostic(")
            .count(),
        2,
        "null and scalar nested keyed assignments should share one path write ABI:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_path_append_with_diagnostic(")
            .count(),
        1,
        "nested append assignment should use the path append ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_clone(value_offset_path_write_value_to_clone_")
            && body.contains("phpc_native_value_clone(value_offset_path_append_value_to_clone_"),
        "path mutation results should be stored through native value clones:\n{source}"
    );
    assert!(
        !body.contains("assembly array lowering rejects"),
        "nested value writes should not fall through to the blanket array rejection:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nested_value_assignment_expressions_through_path_boundaries() {
    let program = parse(VALUE_OFFSET_PATH_ASSIGNMENT_EXPR_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_offset_path_write_with_diagnostic"
        ) && source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_offset_path_append_with_diagnostic"
        ),
        "generated C should declare the shared value-offset path mutation ABI:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_path_write_with_diagnostic(")
            .count(),
        2,
        "nested keyed assignment expressions should share one path write ABI:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_path_append_with_diagnostic(")
            .count(),
        1,
        "nested append assignment expressions should share one path append ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_clone(value_offset_path_write_value_to_clone_")
            && body.contains("phpc_native_value_clone(value_offset_path_append_value_to_clone_"),
        "assignment-expression path mutations should store the mutated owner through native value clones:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_format_stdout_with_diagnostic("),
        "native-value RHS assignment results should remain available to expression consumers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_value_path_unsets_through_path_boundary() {
    let program = parse(VALUE_OFFSET_PATH_UNSET_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_offset_path_unset_with_diagnostic"
        ),
        "generated C should declare the shared value-offset path unset ABI:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_path_unset_with_diagnostic(")
            .count(),
        3,
        "direct, nested, and null-root value unsets should share one path unset ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_clone(value_offset_path_unset_value_to_clone_"),
        "path unset results should be stored through native value clones:\n{source}"
    );
    assert!(
        !body.contains("assembly array lowering rejects"),
        "value path unsets should not fall through to the blanket array rejection:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_assignment_expression_values_through_value_offset_mutation_boundary(
) {
    let program = parse(VALUE_OFFSET_MUTATION_ARRAY_ASSIGNMENT_EXPR_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "extern phpc_NativeArrayHandle phpc_native_value_array_clone(phpc_NativeValueHandle value);"
        ),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_mutation_operation_with_diagnostic(")
            .count(),
        3,
        "array assignment expressions should share the value-offset mutation boundary:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_array_clone(").count(),
        3,
        "array assignment-expression mutations should rematerialize array owners:\n{source}"
    );
    assert!(
        body.contains(", 1, &array_offset_append_assign_expr_diagnostic_"),
        "append assignment expressions should use the shared append operation tag:\n{source}"
    );
    assert!(
        body.contains(", 0, &array_offset_assign_expr_diagnostic_"),
        "keyed assignment expressions should use the shared write operation tag:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_append_value_with_diagnostic("),
        "array assignment expressions should not bypass the value-offset mutation ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_offset_reads_through_value_offset_boundary() {
    let program = parse(VALUE_OFFSET_ARRAY_READ_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_offset_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_operation_with_diagnostic(")
            .count(),
        5,
        "array offset reads should share the value-offset read boundary across output and value consumers:\n{source}"
    );
    assert!(
        body.contains(", 0, &value_offset_read_diagnostic_"),
        "array offset reads should use the shared read operation tag:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "array read results should feed array value mutation consumers:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "array read results should feed native string-result consumers:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_read_key_with_diagnostic("),
        "lowerable generated-C array reads should not bypass the shared value-offset ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_reports_array_read_recovery_through_shared_result_boundaries() {
    let program = parse(VALUE_OFFSET_READ_RECOVERY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("phpc_native_diagnostic_report(value_offset_read_diagnostic_"),
        "direct array-offset reads should report recoverable diagnostics through the value-offset result path:\n{source}"
    );
    assert!(
        body.contains("phpc_native_diagnostic_report(array_lvalue_read_result_"),
        "nested array-lvalue reads should report recoverable diagnostics through the lvalue result path:\n{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_offset_operation_with_diagnostic(")
            .count()
            >= 3,
        "direct reads and probes should continue to share the value-offset ABI:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ")
            .count(),
        2,
        "nested missing/scalar reads should share the lvalue read operation family:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_clone"),
        "recovered native read values should still compose with direct-variable storage:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_offset_null_coalesce_through_value_boundary() {
    let program = parse(VALUE_OFFSET_NULL_COALESCE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_offset_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_bool_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_offset_operation_with_diagnostic(")
            .count(),
        12,
        "array and string offset null-coalescing reads should share presence and read calls:\n{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_value_bool_with_diagnostic(")
            .count(),
        6,
        "null-coalescing probes should pass through the native bool diagnostic boundary:\n{source}"
    );
    assert!(
        body.contains(", 1, &value_offset_null_coalesce_diagnostic_"),
        "null-coalescing probes should use the shared isset operation tag:\n{source}"
    );
    assert!(
        body.contains(", 0, &value_offset_null_coalesce_read_diagnostic_"),
        "present null-coalescing offsets should read through the shared read operation tag:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "offset null-coalescing values should feed downstream native value consumers:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_read_key_with_diagnostic("),
        "offset null-coalescing should not bypass the shared value-offset ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_offset_null_coalesce_assign_through_value_boundary() {
    let program = parse(VALUE_OFFSET_NULL_COALESCE_ASSIGN_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_value_offset_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_bool_with_diagnostic"),
        "{source}"
    );
    assert!(source.contains("phpc_native_value_array_clone"), "{source}");
    assert!(
        body.matches("array_offset_null_coalesce_assign_present_")
            .count()
            >= 5,
        "array-offset ??= statements and expressions should share one presence-probe path:\n{source}"
    );
    assert!(
        body.contains(", 1, &array_offset_null_coalesce_assign_diagnostic_"),
        "array-offset ??= should use the shared isset operation tag for probes:\n{source}"
    );
    assert!(
        body.contains(", 0, &array_offset_null_coalesce_assign_read_diagnostic_"),
        "array-offset ??= expression values should read present slots through the shared read tag:\n{source}"
    );
    assert!(
        body.contains(", 0, &array_offset_null_coalesce_assign_write_diagnostic_"),
        "array-offset ??= should write missing/null slots through the shared mutation tag:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_read_key_with_diagnostic("),
        "array-offset ??= should not reintroduce the legacy direct array read helper:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_nested_null_coalesce_assign_through_lvalue_owner_operation() {
    let program = parse(ARRAY_LVALUE_NESTED_NULL_COALESCE_ASSIGN_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_ISSET"),
        "nested ??= should probe through the owner/path isset operation:\n{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_READ"),
        "nested ??= expression values should read kept slots through the lvalue read operation:\n{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_WRITE"),
        "nested ??= should write nullish/missing slots through the lvalue write operation:\n{source}"
    );
    assert!(
        body.matches("array_lvalue_null_coalesce_assign_present_")
            .count()
            >= 7,
        "statement and expression nested ??= forms should share the null-aware branch path:\n{source}"
    );
    assert!(
        !body.contains("array_offset_null_coalesce_assign_present_"),
        "nested ??= should not use the direct value-offset ??= helper:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_roots_through_root_value_boundary() {
    let program = parse(REQUEST_SUPERGLOBAL_ROOT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(source.contains("phpc_NativeRequestStateHandle"), "{source}");
    assert!(
        source.contains("phpc_native_request_state_empty"),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_REQUEST_STATE_OP_ROOT_VALUE"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_free"),
        "{source}"
    );
    assert!(source.contains("phpc_native_value_cast_result"), "{source}");
    assert!(
        source.contains("phpc_native_value_type_name_result"),
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_ROOT_VALUE")
            .count()
            >= 5,
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_declares_reference_handle_for_reference_abi_uses() {
    for (label, php_source) in [
        ("request-root state", REQUEST_SUPERGLOBAL_ROOT_SOURCE),
        ("array lvalue owners", ARRAY_LVALUE_NESTED_WRITE_SOURCE),
        ("value truthiness", NATIVE_VALUE_TRUTHINESS_SOURCE),
        (
            "callable by-reference frames",
            NATIVE_BY_REFERENCE_USER_FUNCTION_FRAME_SOURCE,
        ),
    ] {
        let program = parse(php_source).unwrap();
        let source = emit_native_executable_c_source(&program).unwrap();
        assert_reference_handle_typedef_precedes_uses(label, &source);
    }
}

#[test]
fn native_executable_c_source_routes_request_root_unset_through_bag_mutation_boundary() {
    let program = parse(REQUEST_SUPERGLOBAL_ROOT_UNSET_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_bag_mutation_operation"),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_REQUEST_STATE_MUTATION_UNSET"),
        "{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_REQUEST_STATE_OP_ROOT_VALUE"),
        "root isset/empty/read should share the root-value operation:\n{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_REQUEST_STATE_STATUS_MISSING_ROOT"),
        "root probes should accept the missing-root status after unset:\n{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_direct_globals_through_symbol_snapshot() {
    let program = parse(GLOBALS_SNAPSHOT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(source.contains("phpc_NativeSymbolTableHandle"), "{source}");
    assert!(source.contains("phpc_native_symbol_table_new"), "{source}");
    assert!(
        source.contains("phpc_native_symbol_table_write"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_snapshot_value"),
        "{source}"
    );
    assert!(source.contains("phpc_native_symbol_table_free"), "{source}");
    assert!(
        body.matches("phpc_native_symbol_table_write").count() >= 3,
        "current root variables should be copied into the symbol-table snapshot:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_snapshot_value")
            .count()
            >= 2,
        "direct $GLOBALS should snapshot for storage and direct value consumers:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_clone"),
        "storing $GLOBALS should clone the owned snapshot for variable storage:\n{source}"
    );
    assert!(
        !source.contains("global-symbol-table lowering rejects $GLOBALS"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_dynamic_globals_paths_through_symbol_table_abi() {
    let program = parse(GLOBALS_SYMBOL_PATH_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_symbol_table_read_value_by_path_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_isset_value_by_path"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_empty_value_by_path"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_read_value_by_path_with_diagnostic")
            .count()
            >= 2,
        "$GLOBALS path reads should use the symbol-table path read ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_isset_value_by_path")
            .count()
            >= 2,
        "$GLOBALS path isset probes should use the quiet symbol-table path ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_empty_value_by_path")
            .count()
            >= 2,
        "$GLOBALS path empty probes should use the quiet symbol-table path ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_NativeValueHandle globals_symbol_path")
            .count()
            >= 6,
        "$GLOBALS path keys should be materialized from value expressions:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_write").count() >= 6,
        "current root variables should be copied into each symbol path snapshot:\n{source}"
    );
    assert!(
        !source.contains("global-symbol-table lowering rejects $GLOBALS"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_globals_path_writes_through_symbol_table_abi() {
    let program = parse(GLOBALS_SYMBOL_PATH_WRITE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_symbol_table_set_value_by_path_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_read_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_set_value_by_path_with_diagnostic")
            .count()
            >= 3,
        "$GLOBALS path writes and direct variable writes after activation should use the shared symbol path write ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_read_value_by_path_with_diagnostic")
            .count()
            >= 2,
        "$GLOBALS path reads should use the shared symbol path read ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_read_with_diagnostic")
            .count()
            >= 1,
        "direct variable reads after symbol-table activation should use the diagnostic root-slot read ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_NativeValueHandle globals_symbol_path")
            .count()
            >= 6,
        "$GLOBALS path writes should materialize root and nested keys as native values:\n{source}"
    );
    assert!(
        !source.contains("global-symbol-table lowering rejects $GLOBALS"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_globals_path_unsets_through_symbol_table_abi() {
    let program = parse(GLOBALS_SYMBOL_PATH_UNSET_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_symbol_table_unset_value_by_path_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_unset_value_by_path_with_diagnostic")
            .count()
            >= 3,
        "$GLOBALS path unsets should use the shared symbol path unset ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_isset_value_by_path")
            .count()
            >= 2,
        "post-unset presence probes should stay on the shared symbol path ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_NativeValueHandle globals_symbol_path")
            .count()
            >= 8,
        "$GLOBALS path unsets should materialize root and nested keys as native values:\n{source}"
    );
    assert!(
        !source.contains("global-symbol-table lowering rejects $GLOBALS"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_globals_path_appends_through_symbol_table_abi() {
    let program = parse(GLOBALS_SYMBOL_PATH_APPEND_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_symbol_table_append_value_by_path_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_read_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_append_value_by_path_with_diagnostic")
            .count()
            >= 4,
        "$GLOBALS path appends and append assignment expressions should use the shared symbol path append ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_read_value_by_path_with_diagnostic")
            .count()
            >= 4,
        "post-append $GLOBALS path reads should stay on the symbol path read ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_read_with_diagnostic")
            .count()
            >= 4,
        "dynamic key variables after activation should use the diagnostic root-slot read ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_NativeValueHandle globals_symbol_path")
            .count()
            >= 12,
        "$GLOBALS path appends should materialize dynamic prefix and suffix keys as native values:\n{source}"
    );
    assert!(
        !source.contains("global-symbol-table lowering rejects $GLOBALS"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_rejects_globals_direct_root_appends_like_php() {
    for source in [
        GLOBALS_DIRECT_ROOT_APPEND_VALUE_SOURCE,
        GLOBALS_DIRECT_ROOT_APPEND_EXPR_SOURCE,
        GLOBALS_DIRECT_ROOT_APPEND_REFERENCE_TARGET_SOURCE,
        GLOBALS_DIRECT_ROOT_APPEND_REFERENCE_SOURCE,
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();
        assert!(
            error.message.contains(GLOBALS_ROOT_APPEND_REJECTION),
            "{error:?}"
        );
    }

    let adjacent =
        emit_native_executable_c_source(&parse(GLOBALS_SYMBOL_PATH_APPEND_SOURCE).unwrap())
            .unwrap();
    assert!(
        adjacent.contains("phpc_native_symbol_table_append_value_by_path_with_diagnostic"),
        "{adjacent}"
    );
    assert!(
        !adjacent.contains("phpc_native_symbol_table_append_root_value_with_diagnostic"),
        "{adjacent}"
    );
}

#[test]
fn native_executable_c_source_routes_symbol_reference_assignments_through_path_abi() {
    let program = parse(SYMBOL_REFERENCE_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_symbol_table_reference_for_path"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_bind_reference_path"),
        "{source}"
    );
    assert!(source.contains("phpc_native_reference_free"), "{source}");
    assert!(
        body.matches("phpc_native_symbol_table_reference_for_path")
            .count()
            >= 6,
        "reference assignment sources should acquire direct, nested, and append references through the shared symbol path ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_bind_reference_path")
            .count()
            >= 6,
        "reference assignment targets should bind direct, nested, and append paths through the shared symbol path ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_NativeValueHandle symbol_reference_path_keys_"),
        "reference paths should materialize dynamic keys as native values:\n{source}"
    );
    assert!(
        !source.contains("reference-assignment lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_active_symbol_array_lvalues_through_reference_owner() {
    let program = parse(SYMBOL_REFERENCE_ARRAY_LVALUE_OWNER_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_reference_slot"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_reference_for_path"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_array_lvalue_owner_reference_slot")
            .count()
            >= 4,
        "active symbol-table array writes, appends, and unsets should borrow the root reference owner:\n{source}"
    );
    assert!(
        body.matches("phpc_native_array_lvalue_owner_value_operation_result")
            .count()
            >= 4,
        "active symbol-table array lvalues should reuse the shared value-operation owner boundary:\n{source}"
    );
    assert!(
        body.matches("phpc_native_reference_free").count() >= 4,
        "borrowed root reference handles should be released after owner operations:\n{source}"
    );
    assert!(
        !source.contains("reference-assignment lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("array lowering rejects arrays"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_globals_symbol_references_through_value_path_abi() {
    let program = parse(GLOBALS_SYMBOL_REFERENCE_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_symbol_table_reference_for_value_path"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_bind_reference_value_path"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_reference_for_path"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_bind_reference_path"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_reference_for_value_path")
            .count()
            >= 3,
        "$GLOBALS symbol reference sources should acquire root, nested, and append paths through the value-path ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_bind_reference_value_path")
            .count()
            >= 2,
        "$GLOBALS symbol reference targets should bind root and append paths through the value-path ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_NativeValueHandle globals_symbol_path"),
        "$GLOBALS symbol reference paths should materialize keys as native values:\n{source}"
    );
    assert!(
        !source.contains("reference-assignment lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("global-symbol-table lowering rejects $GLOBALS"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_globals_dynamic_references_through_request_dispatch() {
    let program = parse(GLOBALS_DYNAMIC_REFERENCE_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert_request_key_results_use_accessors(&source);
    assert!(
        source.contains("phpc_native_request_state_key_matches_superglobal"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_reference_for_root"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_replace_reference_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_path_reference_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_path_reference_bind_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_reference_for_value_path"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_bind_reference_value_path"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_key_matches_superglobal")
            .count()
            >= 5,
        "dynamic $GLOBALS reference dispatch should test request-root aliases before ordinary symbol-table fallback:\n{source}"
    );
    assert!(
        body.contains("phpc_NativeRequestStateKeyResult globals_dynamic_reference_path_key_"),
        "dynamic nested $GLOBALS references should derive request path keys from the already-materialized path values:\n{source}"
    );
    assert!(
        body.contains("phpc_NativeValueHandle globals_dynamic_reference_symbol_path_"),
        "dynamic $GLOBALS references should keep an ordinary symbol-table value-path fallback:\n{source}"
    );
    assert!(
        !source.contains("reference-assignment lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("global-symbol-table lowering rejects $GLOBALS"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_root_references_through_state_abi() {
    let program = parse(REQUEST_SUPERGLOBAL_ROOT_REFERENCE_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_replace_reference_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_reference_for_path"),
        "{source}"
    );
    assert!(source.contains("phpc_native_reference_free"), "{source}");
    assert!(
        body.matches("phpc_native_request_state_superglobal_replace_reference_with_diagnostic")
            .count()
            >= 4,
        "request roots should replace multiple request bags through the shared reference ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_reference_for_path")
            .count()
            >= 4,
        "request root references should acquire direct, nested, and append source references through the shared symbol path ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_NativeValueHandle symbol_reference_path_keys_"),
        "request root references should materialize dynamic source keys as native values:\n{source}"
    );
    assert!(
        !source.contains("reference-assignment lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_root_reference_sources_through_state_abi() {
    let program = parse(REQUEST_SUPERGLOBAL_ROOT_REFERENCE_SOURCE_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_reference_for_root"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_bind_reference_path"),
        "{source}"
    );
    assert!(source.contains("phpc_native_reference_free"), "{source}");
    assert!(
        body.matches("phpc_native_request_state_superglobal_reference_for_root")
            .count()
            >= 4,
        "request roots should be acquired as references across multiple request bags:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_bind_reference_path")
            .count()
            >= 4,
        "request root source references should bind direct, nested, and append symbol targets through the shared path ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_NativeValueHandle symbol_reference_path_keys_"),
        "request root source references should materialize dynamic target keys as native values:\n{source}"
    );
    assert!(
        !source.contains("reference-assignment lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_root_to_root_references_through_state_abi() {
    let program = parse(REQUEST_SUPERGLOBAL_ROOT_TO_ROOT_REFERENCE_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_reference_for_root"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_replace_reference_with_diagnostic"),
        "{source}"
    );
    assert!(source.contains("phpc_native_reference_free"), "{source}");
    assert!(
        body.matches("phpc_native_request_state_superglobal_replace_reference_with_diagnostic")
            .count()
            >= 2,
        "request root reference targets should bind source request roots through the shared state ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_reference_for_root")
            .count()
            >= 4,
        "request root reference sources should be acquired for root-to-root and ordinary-symbol aliases:\n{source}"
    );
    assert!(
        !source.contains("reference-assignment lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_keyed_references_through_state_abi() {
    let program = parse(REQUEST_SUPERGLOBAL_KEYED_REFERENCE_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_keyed_reference_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_keyed_reference_bind_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_reference_result_free"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_reference_for_path"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_bind_reference_path"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_keyed_reference_bind_operation")
            .count()
            >= 2,
        "keyed request reference targets should bind direct and nested symbol sources through the shared request-state ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_keyed_reference_operation")
            .count()
            >= 2,
        "keyed request reference sources should bind direct and append-created symbol targets through the shared request-state ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_key_from_value")
            .count()
            >= 4,
        "keyed request references should materialize dynamic/scalar keys through the request key ABI:\n{source}"
    );
    assert!(
        !source.contains("reference-assignment lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_keyed_to_keyed_references_through_state_abi() {
    let program = parse(REQUEST_SUPERGLOBAL_KEYED_TO_KEYED_REFERENCE_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_keyed_reference_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_keyed_reference_bind_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_reference_result_free"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_keyed_reference_bind_operation")
            .count()
            >= 2,
        "keyed request reference targets should bind keyed request source references through the shared request-state ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_keyed_reference_operation")
            .count()
            >= 4,
        "keyed request reference sources should be acquired for request-to-request and ordinary-symbol aliases:\n{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_key_from_value")
            .count()
            >= 8,
        "keyed request-to-request references should materialize source and target keys through the request key ABI:\n{source}"
    );
    assert!(
        !source.contains("reference-assignment lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_path_references_through_state_abi() {
    let program = parse(REQUEST_SUPERGLOBAL_PATH_REFERENCE_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_path_reference_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_path_reference_bind_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_reference_result_free"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_path_reference_bind_operation")
            .count()
            >= 2,
        "request path reference targets should bind symbol and request path sources through the shared request-state ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_path_reference_operation")
            .count()
            >= 4,
        "request path reference sources should be acquired for request-to-request and ordinary-symbol aliases:\n{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_key_from_value")
            .count()
            >= 14,
        "request path references should materialize dynamic path keys through the request key ABI:\n{source}"
    );
    assert!(
        !source.contains("reference-assignment lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_append_references_through_state_abi() {
    let program = parse(REQUEST_SUPERGLOBAL_APPEND_REFERENCE_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_path_reference_append_operation"),
        "{source}"
    );
    assert!(
        source.contains(
            "phpc_native_request_state_superglobal_path_reference_append_source_operation"
        ),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_reference_result_free"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_reference_for_path"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_bind_reference_path"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_path_reference_append_operation")
            .count()
            >= 2,
        "request append reference targets should append direct and nested request slots through the shared request-state ABI:\n{source}"
    );
    assert!(
        body.matches(
            "phpc_native_request_state_superglobal_path_reference_append_source_operation"
        )
        .count()
            >= 2,
        "request append reference sources should return aliasable root and nested request slots through the shared request-state ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_key_from_value")
            .count()
            >= 2,
        "request append reference paths should materialize dynamic parent keys through the request key ABI:\n{source}"
    );
    assert!(
        !source.contains("reference-assignment lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_symbol_reference_assignment_paths() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("symbol_reference_assignment_paths.php");
    let output_path = native_link_output_path("symbol_reference_assignment_paths");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(&source_path, SYMBOL_REFERENCE_ASSIGNMENT_SOURCE)
        .expect("write symbol reference assignment native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native symbol reference source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native symbol reference executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native symbol reference executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "B|D|E|F|G");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_active_symbol_array_lvalue_reference_owner_paths() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("symbol_array_lvalue_reference_owner.php");
    let output_path = native_link_output_path("symbol_array_lvalue_reference_owner");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(&source_path, SYMBOL_REFERENCE_ARRAY_LVALUE_OWNER_SOURCE)
        .expect("write symbol array lvalue owner native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native symbol array owner source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native symbol array owner executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native symbol array owner executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1|via-alias|appended");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_globals_symbol_reference_assignment_paths() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("globals_symbol_reference_assignment_paths.php");
    let output_path = native_link_output_path("globals_symbol_reference_assignment_paths");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(&source_path, GLOBALS_SYMBOL_REFERENCE_ASSIGNMENT_SOURCE)
        .expect("write globals symbol reference assignment native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native globals symbol reference source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native globals symbol reference executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native globals symbol reference executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "B|D|F|G|H");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_globals_dynamic_reference_assignment_paths() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("globals_dynamic_reference_assignment_paths.php");
    let output_path = native_link_output_path("globals_dynamic_reference_assignment_paths");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(&source_path, GLOBALS_DYNAMIC_REFERENCE_ASSIGNMENT_SOURCE)
        .expect("write dynamic globals reference assignment native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native dynamic globals reference source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native dynamic globals reference executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native dynamic globals reference executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "Ada|C|E|F|G");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_request_root_reference_assignment_paths() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("request_root_reference_assignment_paths.php");
    let output_path = native_link_output_path("request_root_reference_assignment_paths");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        REQUEST_SUPERGLOBAL_ROOT_REFERENCE_ASSIGNMENT_SOURCE,
    )
    .expect("write request root reference assignment native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request root reference source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native request root reference executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native request root reference executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "B|C|NULL|R|S");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_request_root_reference_source_assignment_paths() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("request_root_reference_source_assignment_paths.php");
    let output_path = native_link_output_path("request_root_reference_source_assignment_paths");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        REQUEST_SUPERGLOBAL_ROOT_REFERENCE_SOURCE_ASSIGNMENT_SOURCE,
    )
    .expect("write request root reference source assignment native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request root reference-source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native request root reference-source executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native request root reference-source executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "B|D|F|S");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_request_root_to_root_reference_assignment_paths() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path =
        native_link_output_path("request_root_to_root_reference_assignment_paths.php");
    let output_path = native_link_output_path("request_root_to_root_reference_assignment_paths");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        REQUEST_SUPERGLOBAL_ROOT_TO_ROOT_REFERENCE_ASSIGNMENT_SOURCE,
    )
    .expect("write request root-to-root reference assignment native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request root-to-root reference source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native request root-to-root reference executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native request root-to-root reference executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "B|B|D|D");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_request_keyed_reference_assignment_paths() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("request_keyed_reference_assignment_paths.php");
    let output_path = native_link_output_path("request_keyed_reference_assignment_paths");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        REQUEST_SUPERGLOBAL_KEYED_REFERENCE_ASSIGNMENT_SOURCE,
    )
    .expect("write request keyed reference assignment native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request keyed reference source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native request keyed reference executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native request keyed reference executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "B|D|S|T");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_request_keyed_to_keyed_reference_assignment_paths() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path =
        native_link_output_path("request_keyed_to_keyed_reference_assignment_paths.php");
    let output_path = native_link_output_path("request_keyed_to_keyed_reference_assignment_paths");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        REQUEST_SUPERGLOBAL_KEYED_TO_KEYED_REFERENCE_ASSIGNMENT_SOURCE,
    )
    .expect("write request keyed-to-keyed reference assignment native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request keyed-to-keyed reference source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native request keyed-to-keyed reference executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native request keyed-to-keyed reference executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "B|B|D|D");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_request_path_reference_assignment_paths() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("request_path_reference_assignment_paths.php");
    let output_path = native_link_output_path("request_path_reference_assignment_paths");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        REQUEST_SUPERGLOBAL_PATH_REFERENCE_ASSIGNMENT_SOURCE,
    )
    .expect("write request path reference assignment native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request path reference source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native request path reference executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native request path reference executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "B|D|E|F|G");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_request_append_reference_assignment_paths() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path("request_append_reference_assignment_paths.php");
    let output_path = native_link_output_path("request_append_reference_assignment_paths");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(
        &source_path,
        REQUEST_SUPERGLOBAL_APPEND_REFERENCE_ASSIGNMENT_SOURCE,
    )
    .expect("write request append reference assignment native link source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request append reference source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native request append reference executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native request append reference executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "B|C|E|F");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_routes_direct_root_undefined_reads_through_symbol_table_abi() {
    let program = parse(ROOT_SYMBOL_UNDEFINED_READ_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_symbol_table_read_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_read_with_diagnostic")
            .count()
            >= 5,
        "undefined and active direct root reads should use the shared diagnostic root-slot read ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_set_value_by_path_with_diagnostic")
            .count()
            >= 2,
        "assignments after a root diagnostic read should export through the active symbol table:\n{source}"
    );
    assert!(
        !source.contains("variable-read lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_direct_symbol_unsets_through_symbol_table_abi() {
    let program = parse(DIRECT_SYMBOL_UNSET_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_symbol_table_unset"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_unset").count() >= 3,
        "single and all-direct multi-unset targets should use the shared root-symbol unset ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_symbol_table_read_with_diagnostic"),
        "reads after unset should remain on the diagnostic root-symbol read ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_symbol_table_isset_value_by_path"),
        "isset after unset should remain on the active symbol-table path probe ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_symbol_table_empty_value_by_path"),
        "empty after unset should remain on the active symbol-table path probe ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_symbol_table_set_value_by_path_with_diagnostic"),
        "reassignment after unset should write through the active symbol table:\n{source}"
    );
    assert!(
        !source.contains("assembly mutation lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_sequences_mixed_unset_targets_through_existing_boundaries() {
    let program = parse(MIXED_UNSET_TARGETS_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_symbol_table_unset"),
        "{source}"
    );
    assert!(
        body.matches(" = phpc_native_symbol_table_unset(").count() >= 2,
        "direct variable operands in mixed unset should use the root-symbol unset ABI:\n{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_offset_path_unset_with_diagnostic(")
            .count()
            >= 2,
        "array-offset operands in mixed unset should use the active symbol-table value path unset/writeback boundary:\n{source}"
    );
    assert!(
        body.contains("phpc_native_symbol_table_unset_value_by_path_with_diagnostic"),
        "$GLOBALS operands in mixed unset should use the symbol path unset ABI:\n{source}"
    );
    assert!(
        !source.contains("assembly mutation lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_root_assignments_through_replace_value() {
    let program = parse(REQUEST_SUPERGLOBAL_ROOT_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_replace_value_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_replace_value_with_diagnostic")
            .count()
            >= 5,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_ROOT_VALUE")
            .count()
            >= 5,
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "request-root replacement should consume existing native value-result operands:\n{source}"
    );
    assert_no_diagnostic_report_double_free(&source);
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_reference_backed_request_root_assignments_through_state_abi() {
    let program = parse(REQUEST_SUPERGLOBAL_REFERENCE_BACKED_ROOT_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_replace_reference_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_replace_value_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_replace_reference_with_diagnostic")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_replace_value_with_diagnostic")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("reference assignment lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_reference_backed_request_keyed_mutations_through_state_abi() {
    let program = parse(REQUEST_SUPERGLOBAL_REFERENCE_BACKED_KEYED_MUTATION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_replace_reference_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_keyed_mutation_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_path_mutation_operation"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_replace_reference_with_diagnostic")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_keyed_mutation_operation")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_path_mutation_operation")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("reference assignment lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_keyed_storage_through_state_operations() {
    let program = parse(REQUEST_SUPERGLOBAL_KEYED_STORAGE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert_request_key_results_use_accessors(&source);
    assert!(
        source.contains("phpc_native_request_state_key_from_value"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_keyed_mutation_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_operation_result_report_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_VALUE").count() >= 2,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_PRESENCE")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_MUTATION_WRITE")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_REQUEST_STATE_MUTATION_UNSET"),
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_keyed_empty_through_state_operations() {
    let program = parse(REQUEST_SUPERGLOBAL_KEYED_EMPTY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_key_from_value"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_bool_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_PRESENCE")
            .count()
            >= 5,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_VALUE").count() >= 5,
        "{source}"
    );
    assert!(body.contains("request_superglobal_keyed_empty"), "{source}");
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_path_mutations_through_state_operations() {
    let program = parse(REQUEST_SUPERGLOBAL_PATH_MUTATION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert_request_key_results_use_accessors(&source);
    assert!(
        source.contains("phpc_native_request_state_superglobal_path_mutation_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_operation_result_report_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_path_mutation_operation")
            .count()
            >= 5,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_MUTATION_WRITE")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_MUTATION_UNSET")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        body.contains("const uint8_t *request_superglobal_path_key_ptrs_"),
        "{source}"
    );
    assert!(
        body.contains("size_t request_superglobal_path_key_lens_"),
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_path_appends_through_state_operations() {
    let program = parse(REQUEST_SUPERGLOBAL_PATH_APPEND_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_path_mutation_operation"),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_REQUEST_STATE_MUTATION_APPEND"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_path_mutation_operation")
            .count()
            >= 4,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_MUTATION_APPEND")
            .count()
            >= 4,
        "{source}"
    );
    assert!(
        body.contains("phpc_native_request_state_key_from_value"),
        "{source}"
    );
    assert!(
        body.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "request append assignment-expression values should feed native-value output consumers:\n{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_append_suffixes_through_state_operations() {
    let program = parse(REQUEST_SUPERGLOBAL_APPEND_SUFFIX_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_path_mutation_operation"),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_REQUEST_STATE_MUTATION_APPEND"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_path_mutation_operation")
            .count()
            >= 5,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_MUTATION_APPEND")
            .count()
            >= 5,
        "{source}"
    );
    assert!(
        body.matches("phpc_native_array_insert_key_value_with_diagnostic")
            .count()
            >= 6,
        "request append suffixes should wrap appended values as nested arrays:\n{source}"
    );
    assert!(
        body.matches("phpc_native_value_from_array").count() >= 6,
        "request append suffixes should materialize wrapped array values:\n{source}"
    );
    assert!(
        body.contains("phpc_native_request_state_key_from_value"),
        "{source}"
    );
    assert!(
        body.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "request append assignment-expression values should still feed native-value output consumers:\n{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_path_reads_and_probes_through_state_operations() {
    let program = parse(REQUEST_SUPERGLOBAL_PATH_READ_PROBE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert_request_key_results_use_accessors(&source);
    assert!(
        source.contains("phpc_native_request_state_key_from_value"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_path_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_operation_result_report_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_bool_with_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_path_operation")
            .count()
            >= 8,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_VALUE").count() >= 5,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_PRESENCE")
            .count()
            >= 5,
        "{source}"
    );
    assert!(body.contains("request_superglobal_path_read"), "{source}");
    assert!(
        body.contains("request_superglobal_path_presence"),
        "{source}"
    );
    assert!(body.contains("request_superglobal_path_empty"), "{source}");
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_assignment_expression_values_through_state_operations()
{
    let program = parse(REQUEST_SUPERGLOBAL_ASSIGNMENT_EXPRESSION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_replace_value_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_keyed_mutation_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_path_mutation_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_operation_result_report_diagnostic"),
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_MUTATION_WRITE")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        body.matches("phpc_native_value_clone").count() >= 2,
        "native-value RHS assignment results should be cloned for request storage while the expression result remains available:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "request assignment-expression values should feed native-value output consumers:\n{source}"
    );
    assert!(
        !source.contains("assembly mutation lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_request_null_coalesce_through_state_operations() {
    let program = parse(REQUEST_SUPERGLOBAL_NULL_COALESCE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_path_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_snapshot_value"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_read_with_diagnostic"),
        "lazy request null-coalesce RHS values should still lower through the shared symbol read ABI when needed:\n{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_path_operation")
            .count()
            >= 10,
        "request null-coalesce should share request-state presence and value operations across keyed and nested paths:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_PRESENCE")
            .count()
            >= 5,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_VALUE").count() >= 5,
        "{source}"
    );
    assert!(
        body.contains("request_superglobal_null_coalesce_presence"),
        "{source}"
    );
    assert!(
        body.contains("request_superglobal_null_coalesce_value"),
        "{source}"
    );
    assert!(
        body.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "fallback values should remain ordinary native-value consumers:\n{source}"
    );
    assert!(!source.contains("conditional lowering rejects"), "{source}");
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_globals_request_aliases_through_request_state() {
    let program = parse(GLOBALS_REQUEST_ALIAS_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_replace_value_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_path_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_path_mutation_operation"),
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_MUTATION_WRITE")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_REQUEST_STATE_MUTATION_APPEND"),
        "{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_REQUEST_STATE_MUTATION_UNSET"),
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_VALUE").count() >= 4,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_PRESENCE")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        !body.contains("phpc_native_symbol_table_set_value_by_path_with_diagnostic"),
        "$GLOBALS request-root aliases should not write the ordinary symbol table:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_symbol_table_read_value_by_path_with_diagnostic"),
        "$GLOBALS request-root aliases should not read the ordinary symbol table:\n{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("global-symbol-table lowering rejects $GLOBALS"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_globals_self_request_aliases_through_request_state() {
    let program = parse(GLOBALS_SELF_REQUEST_ALIAS_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_superglobal_replace_value_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_path_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_keyed_mutation_operation"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_path_mutation_operation"),
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_MUTATION_WRITE")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_REQUEST_STATE_MUTATION_APPEND"),
        "{source}"
    );
    assert!(
        body.contains("PHPC_NATIVE_REQUEST_STATE_MUTATION_UNSET"),
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_VALUE").count() >= 3,
        "{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_REQUEST_STATE_OP_PRESENCE")
            .count()
            >= 3,
        "{source}"
    );
    assert!(
        !body.contains("phpc_native_symbol_table_set_value_by_path_with_diagnostic"),
        "$GLOBALS self request-root aliases should not write the ordinary symbol table:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_symbol_table_read_value_by_path_with_diagnostic"),
        "$GLOBALS self request-root aliases should not read the ordinary symbol table:\n{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("global-symbol-table lowering rejects $GLOBALS"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_dispatches_dynamic_globals_request_root_assignments() {
    let program = parse(GLOBALS_DYNAMIC_REQUEST_ROOT_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert_request_key_results_use_accessors(&source);
    assert!(
        source.contains("phpc_native_request_state_key_matches_superglobal"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_key_from_value"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_replace_value_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_set_value_by_path_with_diagnostic"),
        "ordinary dynamic $GLOBALS roots should still fall back to the symbol-table path ABI:\n{source}"
    );
    assert!(
        body.matches("globals_dynamic_request_match_").count() >= 7,
        "dynamic root dispatch should probe the request-superglobal root family, not one concrete name:\n{source}"
    );
    assert!(
        body.matches("phpc_native_request_state_superglobal_replace_value_with_diagnostic")
            .count()
            >= 7,
        "each request root branch should share the request root replacement ABI:\n{source}"
    );
    assert!(
        body.contains("globals_dynamic_symbol_path_"),
        "non-request dynamic roots should keep the ordinary $GLOBALS symbol path fallback:\n{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("global-symbol-table lowering rejects $GLOBALS"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_dispatches_dynamic_globals_request_root_reads_and_probes() {
    let program = parse(GLOBALS_DYNAMIC_REQUEST_ROOT_READ_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_request_state_key_matches_superglobal"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_key_from_value"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_superglobal_snapshot_value"),
        "matched dynamic request roots should read snapshots through request state:\n{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_read_value_by_path_with_diagnostic"),
        "ordinary dynamic $GLOBALS roots should still read through the symbol-table path ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_isset_value_by_path"),
        "ordinary dynamic $GLOBALS root isset probes should stay on the symbol-table path ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_empty_value_by_path"),
        "ordinary dynamic $GLOBALS root empty probes should stay on the symbol-table path ABI:\n{source}"
    );
    assert!(
        body.matches("globals_dynamic_request_read_match_").count() >= 7,
        "dynamic root reads should probe the request-superglobal root family:\n{source}"
    );
    assert!(
        body.matches("globals_dynamic_request_presence_match_")
            .count()
            >= 7,
        "dynamic root isset probes should probe the request-superglobal root family:\n{source}"
    );
    assert!(
        body.matches("globals_dynamic_request_empty_match_").count() >= 7,
        "dynamic root empty probes should probe the request-superglobal root family:\n{source}"
    );
    assert!(
        body.contains("globals_dynamic_symbol_read_"),
        "non-request dynamic roots should keep the ordinary $GLOBALS symbol read fallback:\n{source}"
    );
    assert!(
        body.contains("globals_dynamic_symbol_presence_"),
        "non-request dynamic roots should keep the ordinary $GLOBALS symbol presence fallback:\n{source}"
    );
    assert!(
        body.contains("globals_dynamic_symbol_empty_"),
        "non-request dynamic roots should keep the ordinary $GLOBALS symbol empty fallback:\n{source}"
    );
    assert!(
        !source.contains("request-superglobal lowering rejects"),
        "{source}"
    );
    assert!(
        !source.contains("global-symbol-table lowering rejects $GLOBALS"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_stores_native_value_results_in_direct_variables() {
    let program = parse(NATIVE_VALUE_VARIABLE_STORAGE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("extern phpc_NativeValueHandle phpc_native_value_clone"),
        "{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_clone(").count() >= 3,
        "native-value variable reads should clone handles for variable copies and downstream consumers:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_offset_operation_with_diagnostic"),
        "stored array read and null-coalesce values should use the value-offset boundary:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "stored native values should feed string-result consumers:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_cast_result"),
        "cast results should store through the same native value handle path:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_array_read_key_with_diagnostic("),
        "stored array offset reads should not reintroduce the array-read bypass:\n{source}"
    );
}

#[test]
fn native_executable_c_source_writes_root_offset_mutations_to_active_symbol_table() {
    let program = parse(ACTIVE_SYMBOL_ROOT_OFFSET_MUTATION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "direct root offset mutations should still use the shared value-offset mutation ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_value_offset_path_append_with_diagnostic"),
        "path appends should still use the shared value-offset path append ABI:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_set_value_by_path_with_diagnostic")
            .count()
            >= 5,
        "active symbol-table root mutations should write their root value back through the symbol path ABI:\n{source}"
    );
    assert!(
        !body.contains("undefined array key 1"),
        "source generation should not encode the stale read diagnostic:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_offset_unsets_through_lvalue_owner_operation() {
    let program = parse(VALUE_OFFSET_MUTATION_ARRAY_UNSET_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("typedef struct { uint8_t tag; phpc_NativeValueHandle key; } phpc_NativeArrayPathSegment"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_lvalue_owner_array"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UNSET"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_array_lvalue_owner_value_operation_result(")
            .count(),
        4,
        "direct and nested array-offset unsets should share the lvalue owner operation boundary:\n{source}"
    );
    assert!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_KEY").count() >= 5,
        "direct and nested unset paths should materialize every key through path segments:\n{source}"
    );
    assert!(
        body.contains(", 0, &array_offset_write_diagnostic_"),
        "the adjacent write should stay on the value-offset mutation ABI:\n{source}"
    );
    assert!(
        body.matches(" = phpc_native_value_offset_mutation_operation_with_diagnostic(")
            .count()
            == 1,
        "unset should leave the value-offset mutation ABI for the lvalue owner path while preserving the follow-up write:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_array_unset_int")
            && !source.contains("phpc_native_array_unset_string"),
        "array unset should not reintroduce direct int/string unset helpers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_sequences_multi_operand_array_offset_unsets_through_lvalue_owner() {
    let program = parse(VALUE_OFFSET_MUTATION_ARRAY_MULTI_UNSET_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_native_array_lvalue_owner_value_operation_result"),
        "{source}"
    );
    assert_eq!(
        body.matches(" = phpc_native_array_lvalue_owner_value_operation_result(")
            .count(),
        4,
        "each unset operand should enter the shared lvalue owner operation boundary:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_LVALUE_VALUE_OPERATION_UNSET")
            .count(),
        4,
        "multi-operand unset should reuse the shared unset operation family for every operand:\n{source}"
    );
    assert_eq!(
        body.matches("PHPC_NATIVE_ARRAY_PATH_KEY").count(),
        4,
        "multi-operand direct unset should materialize every key through path segments:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_array_unset_int")
            && !source.contains("phpc_native_array_unset_string"),
        "multi-operand unset should not reintroduce direct array unset helpers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_offset_writes_through_value_offset_mutation_boundary() {
    let program = parse(STRING_OFFSET_WRITE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_offset_mutation_operation_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_string_clone_bytes"),
        "{source}"
    );
    assert!(source.contains("phpc_native_byte_buffer_free"), "{source}");
    assert!(
        source
            .matches(" = phpc_native_value_offset_mutation_operation_with_diagnostic(")
            .count()
            >= 2,
        "string-offset writes should share the value-offset mutation boundary:\n{source}"
    );
    assert!(
        source.contains(", 0, &string_offset_write_diagnostic_"),
        "string-offset writes should use the write operation tag:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_string_offset_write_with_diagnostic"),
        "generated-C string-offset writes should not keep the string-only write ABI:\n{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_string_clone_bytes(")
            .count()
            >= 2,
        "write results should become byte buffers through the shared clone boundary:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_byte_buffer_free(string_offset_write_buffer")
            .count()
            >= 2,
        "owned string-offset write byte buffers must be cleaned up:\n{source}"
    );
    assert!(
        source
            .contains("phpc_native_string_from_bytes((const uint8_t *)(string_offset_write_bytes"),
        "dynamic write bytes should be rematerialized by byte length:\n{source}"
    );
    assert!(
        !source.contains("strlen((const char *)(string_offset_write_bytes"),
        "write result byte lengths should come from the runtime byte buffer:\n{source}"
    );
    assert!(!source.contains("printf(\"%s\""), "{source}");
}

const FILESYSTEM_PATH_OPERATION_SOURCE: &str = "<?php\n$path = \"pmt/\\0A\";\n$flag = str_contains($path, \"\\0\");\nfile_get_contents($path, $flag);\nrealpath($path);\nfile_exists(42);\nis_writable($path);\nfilesize($path);\nfilemtime($path);\ngetcwd();\nclearstatcache($flag, $path);\nrealpath_cache_get();\nrealpath_cache_size();\necho \"done\\n\";\n";

#[test]
fn native_executable_c_source_routes_filesystem_path_builtins_through_shared_blocker() {
    let program = parse(FILESYSTEM_PATH_OPERATION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_filesystem_path_operation_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_value_filesystem_path_operation_with_diagnostic(")
            .count(),
        10,
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_string_predicate_with_diagnostic"),
        "filesystem optional flags should compose with the existing truthy value producer:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_from_scalar")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "filesystem path operands should enter the same native value boundary for scalar and string families:\n{source}"
    );
    assert!(
        source.contains(", 0, &filesystem_path_operation_diagnostic_")
            && source.contains(", 1, &filesystem_path_operation_diagnostic_")
            && source.contains(", 2, &filesystem_path_operation_diagnostic_")
            && source.contains(", 6, &filesystem_path_operation_diagnostic_")
            && source.contains(", 8, &filesystem_path_operation_diagnostic_")
            && source.contains(", 9, &filesystem_path_operation_diagnostic_")
            && source.contains(", 10, &filesystem_path_operation_diagnostic_")
            && source.contains(", 11, &filesystem_path_operation_diagnostic_")
            && source.contains(", 12, &filesystem_path_operation_diagnostic_")
            && source.contains(", 13, &filesystem_path_operation_diagnostic_"),
        "filesystem path builtins should share one operation-tagged ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_string_integer_arguments_through_value_conversion() {
    let program = parse(
        "<?php\n$offset = \"0\";\n$length = 4.0;\n$insert = true;\n$replace = \"1\";\n$delete = 1.0;\necho substr_count(\"aaaa\", \"aa\", $offset, $length);\necho levenshtein(\"kitten\", \"sitting\", $insert, $replace, $delete);\n",
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_to_int64_with_diagnostic"),
        "{source}"
    );
    assert_eq!(
        source
            .matches(" = (long long)phpc_native_value_to_int64_with_diagnostic(")
            .count(),
        5,
        "substr_count offset/length and levenshtein costs should share the same int conversion ABI:\n{source}"
    );
    assert!(
        source.contains(", 0, &int_conversion_diagnostic_")
            && source.contains(", 1, &int_conversion_diagnostic_")
            && source.contains(", 2, &int_conversion_diagnostic_"),
        "string offset, string length, and string distance cost roles should use operation tags:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_string_search_result_with_diagnostic")
            && source.contains("phpc_native_value_string_distance_operation_with_diagnostic"),
        "converted int arguments should compose with both string-search and string-distance consumers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_comparison_families_through_runtime_contract() {
    let program = parse(
        r#"<?php
echo 1 == "1", "\n";
echo 1 != "2", "\n";
echo 2 < "10", "\n";
echo 2 <= "2", "\n";
echo "10" > 2, "\n";
echo "alpha" >= "alpha", "\n";
echo "10" < "zeta", "\n";
echo "8foo" > "2", "\n";
echo ".5m" < "5.", "\n";
echo "+foo" < "-word", "\n";
echo 2 === 2, "\n";
echo null == false, "\n";
echo 1 !== "1";
"#,
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(source.contains("#include <stdbool.h>"), "{source}");
    assert!(
        source.contains("phpc_native_value_from_scalar")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("extern phpc_NativeValueOperationResult phpc_native_value_compare_result"),
        "non-strict comparison families should share the native value comparison result ABI:\n{source}"
    );
    assert!(
        source.matches(" = phpc_native_value_compare_result(").count() >= 11,
        "direct scalar/string/null comparison echoes should route through value-result comparison:\n{source}"
    );
    for op in [
        "PHPC_NATIVE_VALUE_COMPARISON_EQ",
        "PHPC_NATIVE_VALUE_COMPARISON_NE",
        "PHPC_NATIVE_VALUE_COMPARISON_LT",
        "PHPC_NATIVE_VALUE_COMPARISON_LE",
        "PHPC_NATIVE_VALUE_COMPARISON_GT",
        "PHPC_NATIVE_VALUE_COMPARISON_GE",
    ] {
        assert!(source.contains(op), "{op}\n\n{source}");
    }
    assert!(
        source.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "direct comparison echoes should keep the runtime-owned result through stdout formatting:\n{source}"
    );
    assert!(
        !source.contains("phpc_NativeComparisonOperand")
            && !source.contains("phpc_native_comparison_operand_compare_operation_relation_and_free")
            && !source.contains("phpc_native_comparison_branch_decision_result_operand")
            && !source.contains("phpc_native_comparison_branch_decision_is_true"),
        "non-strict scalar/string comparisons should not use the older comparison-decision ABI:\n{source}"
    );
}

#[test]
fn native_executable_c_source_rematerializes_nested_comparisons_as_value_operands() {
    let program = parse(
        r#"<?php
$payload = "2";
echo (($payload > 1) == true), "\n";
echo (((1 < 2) == (2 > 1)) ? 1 : 0), "\n";
echo ((null == false) != ("10" < 2));
"#,
    )
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("extern phpc_NativeValueOperationResult phpc_native_value_compare_result"),
        "nested comparisons should declare the shared native value comparison result ABI:\n{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_compare_result(")
            .count()
            >= 7,
        "nested loose comparison operands should rematerialize through value-result handles:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_is_truthy"),
        "ternary conditions should consume comparison result truth through native value truthiness:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_decision_result_operand")
            && !source.contains("phpc_NativeComparisonOperand"),
        "nested loose comparisons should not rematerialize through the older branch-decision operand ABI:\n{source}"
    );
}

const ARRAY_HANDLE_COMPARISON_SOURCE: &str = "<?php\n$left = [1, \"two\" => 2];\n$right = [1, \"two\" => 2];\necho ($left === $right), \"\\n\";\necho ([1, \"two\" => 2] !== [1, \"two\" => 3]), \"\\n\";\necho ([1] == [1]), \"\\n\";\necho ([2] > [1]), \"\\n\";\n";

#[test]
fn native_executable_c_source_routes_array_handle_comparisons_through_runtime_boundaries() {
    let program = parse(ARRAY_HANDLE_COMPARISON_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(source.contains("phpc_NativeArrayHandle"), "{source}");
    assert!(
        source
            .contains("extern phpc_NativeComparisonBranchResult phpc_native_array_compare_branch"),
        "generated C should declare the shared array comparison branch ABI:\n{source}"
    );
    assert_eq!(
        source
            .matches(" = phpc_native_array_compare_branch(")
            .count(),
        2,
        "strict array comparisons should keep the array branch ABI:\n{source}"
    );
    assert!(
        source.matches(" = phpc_native_value_compare_result(").count() >= 2,
        "non-strict array equality and ordering should route through the shared value-result comparison ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_NativeComparisonBranchDecision")
            && source.contains("phpc_native_comparison_branch_decision_from_result")
            && source.contains("phpc_native_comparison_branch_decision_abort_code")
            && source.contains("phpc_native_comparison_branch_decision_is_true"),
        "array comparison results should use the common branch-decision abort/truth ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_decision_status"),
        "array comparison guards should not duplicate branch-decision status handling outside the abort-code ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_decision_exit_code"),
        "array comparison guards should not duplicate branch-decision exit-code handling outside the abort-code ABI:\n{source}"
    );
    assert!(
        !source.contains("if (comparison_exit_code_"),
        "array comparison guards should not use exit-code checks as the status classifier:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_result_exit_code"),
        "array comparison results should not use raw branch-result exit accessors:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_comparison_branch_result_is_true"),
        "array comparison results should not use raw branch-result truth accessors:\n{source}"
    );
    assert!(
        !source.contains(" = phpc_native_comparison_operand_compare_operation_branch_and_free("),
        "array handles should not pass through scalar/string comparison operands:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_array_compare_branch_and_free("),
        "generated C should keep array handle ownership with the existing cleanup list:\n{source}"
    );
    assert!(
        source.contains("phpc_native_array_free(array_"),
        "array comparison should preserve existing generated-C array cleanup:\n{source}"
    );
}

const ARRAY_HANDLE_STRICT_COMPARISON_SOURCE: &str = "<?php\n$left = [1, \"two\" => 2];\n$right = [1, \"two\" => 2];\n$different = [1, \"two\" => 3];\necho ($left === $right), \"\\n\";\necho ([1, \"two\" => 2] !== $different), \"\\n\";\n";

#[test]
fn emit_exe_links_and_runs_array_handle_strict_comparison_program() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("array_handle_strict_comparison").with_extension("php");
    fs::write(&temp_php, ARRAY_HANDLE_STRICT_COMPARISON_SOURCE)
        .expect("write native array-handle comparison fixture");
    let output_path = native_link_output_path("array_handle_strict_comparison");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native array comparison executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native array comparison executable: {error}")
    });

    assert!(
        run.status.success(),
        "native array comparison executable failed"
    );
    assert_eq!(run.stdout, b"1\n1\n");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

const DYNAMIC_BINARY_STRING_COMPARISON_SOURCE: &str = r#"<?php
$flag = 1 < 2;
$left = $flag ? "2\x00z" : "10\x00w";
$right = $flag ? "2\x00g" : "10\x00a";
echo ($left > $right) ? 1 : 0, "\n";
echo ($right < $left) ? 1 : 0, "\n";
echo ($left != "2\x00a") ? 1 : 0, "\n";
echo ($left == "2\x00z") ? 1 : 0;
"#;

const ASSEMBLY_CONDITIONAL_REJECTION: &str = "assembly conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior";
const ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "assembly dynamic function-call lowering rejects variable-call expressions outside the bounded generated-C finite known-string dispatch to registered user-function frames, supported native builtin families, or supported mixed callable target sets, runtime string-valued dispatch to registered user-function frames or supported native builtin families, and descriptor-backed closure values, including unknown callables, unsupported runtime callable builtin families, unsupported finite target sets, unsupported by-reference argument carriers, callbacks, methods, non-descriptor closures, and exact native callable errors; phpc run handles broader dynamic function calls";

#[test]
fn native_executable_c_source_quarantines_dynamic_string_operand_lengths_at_conditional_boundary() {
    let program = parse(DYNAMIC_BINARY_STRING_COMPARISON_SOURCE).unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert_eq!(error.message, ASSEMBLY_CONDITIONAL_REJECTION);
}

#[test]
fn emit_exe_links_and_runs_direct_string_runtime_helper_program() {
    if !has_cc() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let fixture =
        workspace_root.join("tests/fixtures/milestone2300/native_link_runtime_helper.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();
    let output_path = native_link_output_path("direct_string_runtime_helper");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            &relative_fixture,
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));
    let expected = strip_fixture_editor_newline(
        fs::read_to_string(
            workspace_root.join("tests/fixtures/milestone2300/native_link_runtime_helper.stdout"),
        )
        .expect("expected native stdout fixture is readable"),
    );

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_strlen_conversion_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("strlen_conversion");
    let source_path = native_link_output_path("strlen_conversion_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        "<?php\n$payload = \"A\0B\";\necho strlen(42);\necho strlen(false);\necho strlen(null);\necho strlen($payload);\necho \"\\n\";\n",
    )
    .expect("native strlen conversion source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native strlen source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "2003\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_strlen_value_result_conversion_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "strlen_value_result_conversion",
        NATIVE_VALUE_RESULT_STRLEN_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native strlen value-result executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"2|2|5|3");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_predicate_conversion_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_predicate_conversion");
    let source_path = native_link_output_path("string_predicate_conversion_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        "<?php\n$payload = \"A\0B\";\necho (str_starts_with($payload, \"A\0\") ? 1 : 0);\necho (str_ends_with($payload, \"\0B\") ? 1 : 0);\necho (str_contains(42, \"2\") ? 1 : 0);\necho (str_contains($payload, \"\") ? 1 : 0);\necho (str_contains($payload, \"C\") ? 1 : 0);\necho \"\\n\";\n",
    )
    .expect("native string predicate source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string predicate source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "11110\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_int_operation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_int_operation");
    let source_path = native_link_output_path("string_int_operation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        "<?php\n$payload = \"A\0B\";\necho strcasecmp($payload, \"a\0b\");\necho \"\\n\";\necho strcmp($payload, \"a\0b\");\necho \"\\n\";\necho strncmp($payload, \"A\0C\", 3);\necho \"\\n\";\necho strncasecmp($payload, \"a\0c\", \"2\");\necho \"\\n\";\necho ord($payload);\necho \"\\n\";\necho ord(42042);\necho \"\\n\";\necho crc32(\"123456789\");\necho \"\\n\";\necho crc32($payload);\necho \"\\n\";\necho crc32(null);\necho \"\\n\";\n",
    )
    .expect("native string-int source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-int source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "0\n-1\n-1\n0\n65\n52\n3421780262\n382410329\n0\n"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_search_value_result_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "string_search_value_result",
        "<?php\n$payload = \"A\0B\";\n$repeated = \"A\0BA\0B\";\necho strpos($repeated, $payload);\necho \"\\n\";\necho strpos($repeated, $payload, 2);\necho \"\\n\";\necho strpos($repeated, \"missing\");\necho \"\\n\";\necho strpos($repeated, \"\", 3);\necho \"\\n\";\necho substr_count($repeated, $payload, 0, 6);\necho \"\\n\";\necho substr_count(42042, 42);\necho \"\\n\";\n",
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n3\n\n3\n2\n2\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_distance_operation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_distance_operation");
    let source_path = native_link_output_path("string_distance_operation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        "<?php\n$left = \"kitten\";\n$right = \"sitting\";\n$insert = 1;\n$replace = 2;\n$delete = 1;\necho levenshtein($left, $right);\necho \"\\n\";\necho levenshtein(\"A\0B\", \"A\0C\", $insert, $replace, $delete);\necho \"\\n\";\necho similar_text(42042, 42);\necho \"\\n\";\n",
    )
    .expect("native string-distance source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-distance source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "3\n2\n2\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_unary_string_result_operation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_result_operation");
    let source_path = native_link_output_path("string_result_operation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, NATIVE_STRING_RESULT_SOURCE)
        .expect("native string-result source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-result source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"B\0A|Nm-09|410042|mixed|MIXED|Word|word|24024");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_shell_escape_string_result_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("shell_escape_string_result_operation");
    let source_path = native_link_output_path("shell_escape_string_result_operation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, SHELL_ESCAPE_STRING_RESULT_SOURCE)
        .expect("native shell-escape source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native shell-escape source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"'X ;$'\\''Q\"'|X \\;\\$\\'Q\\\"|'42042'");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_offset_isset_empty_bool_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_offset_isset_empty_bool_boundary");
    let source_path = native_link_output_path("string_offset_isset_empty_bool_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, STRING_OFFSET_ISSET_EMPTY_SOURCE)
        .expect("native string-offset bool source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-offset bool source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|1|0|1|0");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_and_string_offset_presence_value_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_string_offset_presence_value_boundary");
    let source_path =
        native_link_output_path("array_string_offset_presence_value_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_PRESENCE_SOURCE)
        .expect("native value-offset presence source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native value-offset presence source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|1|0|1|1|0");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_offset_write_value_mutation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_offset_write_value_mutation");
    let source_path = native_link_output_path("array_offset_write_value_mutation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_MUTATION_ARRAY_WRITE_SOURCE)
        .expect("native array offset mutation source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array offset mutation source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A|B|C|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_nested_array_lvalue_write_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("nested_array_lvalue_write");
    let source_path = native_link_output_path("nested_array_lvalue_write_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_NESTED_WRITE_SOURCE)
        .expect("native nested array lvalue write source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native nested array lvalue write source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|0|1|R");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_nested_array_lvalue_append_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("nested_array_lvalue_append");
    let source_path = native_link_output_path("nested_array_lvalue_append_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_NESTED_APPEND_SOURCE)
        .expect("native nested array lvalue append source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native nested array lvalue append source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|0|1|R");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_nested_array_assignment_expression_value_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("nested_array_assignment_expr_value");
    let source_path = native_link_output_path("nested_array_assignment_expr_value_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_NESTED_ASSIGNMENT_EXPR_SOURCE)
        .expect("native nested array assignment-expression source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native nested array assignment-expression source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A|B|C|1|0|1|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_nested_array_lvalue_read_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("nested_array_lvalue_read");
    let source_path = native_link_output_path("nested_array_lvalue_read_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_NESTED_READ_SOURCE)
        .expect("native nested array lvalue read source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native nested array lvalue read source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"v|x|V|v");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_offset_append_value_mutation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_offset_append_value_mutation");
    let source_path = native_link_output_path("array_offset_append_value_mutation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_MUTATION_ARRAY_APPEND_SOURCE)
        .expect("native array offset append source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array offset append source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A|B|C|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_value_append_assignment_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("value_append_assignment_boundary");
    let source_path = native_link_output_path("value_append_assignment_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_MUTATION_VALUE_APPEND_SOURCE)
        .expect("native value append source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native value append source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A|BC|3");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("Automatic conversion of false to array is deprecated")
            && stderr.contains("cannot use a scalar value as an array"),
        "stderr:\n{stderr}"
    );

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_unassigned_value_offset_write_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("value_offset_write_unassigned_boundary");
    let source_path = native_link_output_path("value_offset_write_unassigned_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_MUTATION_VALUE_WRITE_SOURCE)
        .expect("native value write source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native value write source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"U|N|F|3");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("Automatic conversion of false to array is deprecated")
            && stderr.contains("cannot use a scalar value as an array"),
        "stderr:\n{stderr}"
    );

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_nested_value_path_write_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("nested_value_path_write_boundary");
    let source_path = native_link_output_path("nested_value_path_write_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_PATH_MUTATION_SOURCE)
        .expect("native nested value path source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native nested value path source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|1|3");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("Automatic conversion of false to array is deprecated")
            && stderr.contains("cannot use a scalar value as an array"),
        "stderr:\n{stderr}"
    );

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_nested_value_path_assignment_expression_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("nested_value_path_assignment_expression_boundary");
    let source_path =
        native_link_output_path("nested_value_path_assignment_expression_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_PATH_ASSIGNMENT_EXPR_SOURCE)
        .expect("native nested value path assignment-expression source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native nested assignment-expression source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A|1|B|1|C|1");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("Automatic conversion of false to array is deprecated"),
        "stderr:\n{stderr}"
    );

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_value_path_unset_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("value_path_unset_boundary");
    let source_path = native_link_output_path("value_path_unset_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_PATH_UNSET_SOURCE)
        .expect("native value path unset source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native value path unset source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"0|0|1|");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_assignment_expression_value_mutation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_assignment_expr_value_mutation");
    let source_path = native_link_output_path("array_assignment_expr_value_mutation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        VALUE_OFFSET_MUTATION_ARRAY_ASSIGNMENT_EXPR_SOURCE,
    )
    .expect("native array assignment-expression source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array assignment-expression source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"B|C|D|B|C|D|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_offset_read_value_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_offset_read_value_boundary");
    let source_path = native_link_output_path("array_offset_read_value_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_ARRAY_READ_SOURCE)
        .expect("native array offset read source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array offset read source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"q|B|q|Q");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

const ARRAYACCESS_READ_ISSET_SOURCE: &str = concat!(
    "<?php\n",
    "class TruthBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { if ($offset) { return \"T\"; } return \"F\"; }\n",
    "    public function offsetExists($offset) { return $offset; }\n",
    "    public function offsetSet($offset, $value) { return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class BaseBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { if ($offset) { return \"CT\"; } return \"CF\"; }\n",
    "    public function offsetExists($offset) { return $offset; }\n",
    "    public function offsetSet($offset, $value) { return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class ChildBag extends BaseBag {}\n",
    "$bag = new TruthBag();\n",
    "echo $bag[\"slot\"], \"|\", $bag[0], \"|\";\n",
    "echo isset($bag[\"slot\"]) ? \"Y\" : \"N\", \"|\", isset($bag[0]) ? \"Y\" : \"N\", \"|\";\n",
    "$alias = $bag;\n",
    "echo $alias[true], \"|\";\n",
    "$child = new ChildBag();\n",
    "echo $child[true], \"|\", isset($child[null]) ? \"Y\" : \"N\", \"|\";\n",
    "echo isset($child[1]) ? \"Y\" : \"N\";\n",
);

const ARRAYACCESS_EMPTY_NULLCOALESCE_SOURCE: &str = concat!(
    "<?php\n",
    "class ValueBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { return \"V\"; }\n",
    "    public function offsetExists($offset) { return $offset; }\n",
    "    public function offsetSet($offset, $value) { return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class ZeroBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { return 0; }\n",
    "    public function offsetExists($offset) { return $offset; }\n",
    "    public function offsetSet($offset, $value) { return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class StringZeroBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { return \"0\"; }\n",
    "    public function offsetExists($offset) { return $offset; }\n",
    "    public function offsetSet($offset, $value) { return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class NullBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { return null; }\n",
    "    public function offsetExists($offset) { return $offset; }\n",
    "    public function offsetSet($offset, $value) { return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "function fallback_value($value) { echo \"R\"; return $value; }\n",
    "$value = new ValueBag();\n",
    "$zero = new ZeroBag();\n",
    "$string_zero = new StringZeroBag();\n",
    "$nulls = new NullBag();\n",
    "echo empty($value[0]) ? \"EM\" : \"EX\";\n",
    "echo \"|\", empty($value[true]) ? \"ET\" : \"EF\";\n",
    "echo \"|\", empty($zero[true]) ? \"ZT\" : \"ZF\";\n",
    "echo \"|\", empty($string_zero[true]) ? \"ST\" : \"SF\";\n",
    "echo \"|\", $value[0] ?? fallback_value(\"M\");\n",
    "echo \"|\", $value[true] ?? fallback_value(\"B\");\n",
    "echo \"|\", $zero[true] ?? fallback_value(\"Z\");\n",
    "echo \"|\", $string_zero[true] ?? fallback_value(\"S\");\n",
    "echo \"|\", $nulls[true] ?? fallback_value(\"N\");\n",
);

const ARRAYACCESS_DYNAMIC_PRODUCER_FACT_SOURCE: &str = concat!(
    "<?php\n",
    "class DynamicTruthBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { if ($offset) { return \"T\"; } return \"F\"; }\n",
    "    public function offsetExists($offset) { return $offset; }\n",
    "    public function offsetSet($offset, $value) { echo \"T:set:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \"=\", $value, \";\"; return null; }\n",
    "    public function offsetUnset($offset) { echo \"T:unset:\", $offset, \";\"; return null; }\n",
    "}\n",
    "class DynamicBaseBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { if ($offset) { return \"B\"; } return \"b\"; }\n",
    "    public function offsetExists($offset) { return $offset; }\n",
    "    public function offsetSet($offset, $value) { echo \"B:set:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \"=\", $value, \";\"; return null; }\n",
    "    public function offsetUnset($offset) { echo \"B:unset:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \";\"; return null; }\n",
    "}\n",
    "class DynamicChildBag extends DynamicBaseBag {}\n",
    "$flag = 2 + \"1\";\n",
    "if ($flag) {\n",
    "    $class = \"dynamictruthbag\";\n",
    "} else {\n",
    "    $class = \"DynamicChildBag\";\n",
    "}\n",
    "$bag = new $class();\n",
    "echo $bag[\"slot\"], \"|\", isset($bag[\"slot\"]) ? \"Y\" : \"N\", \"|\";\n",
    "$bag[\"write\"] = \"W\";\n",
    "$bag[] = \"A\";\n",
    "unset($bag[\"write\"]);\n",
    "echo $bag[\"after\"], \"|\";\n",
    "$alias = $bag;\n",
    "echo $alias[0], \"|\";\n",
    "$alias[\"copy\"] = \"C\";\n",
    "$alias[] = 4 + 5;\n",
    "unset($alias[false]);\n",
    "echo $alias[true], \"|\";\n",
    "echo (new $class())[true], \"|\";\n",
    "if ($flag) {\n",
    "    $branch = new DynamicTruthBag();\n",
    "} else {\n",
    "    $branch = new DynamicChildBag();\n",
    "}\n",
    "echo $branch[true];\n",
);

const ARRAYACCESS_CALLABLE_RETURN_PRODUCER_FACT_SOURCE: &str = concat!(
    "<?php\n",
    "class ReturnTruthBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { if ($offset) { return \"T\"; } return \"F\"; }\n",
    "    public function offsetExists($offset) { return $offset; }\n",
    "    public function offsetSet($offset, $value) { return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class ReturnAltBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { return \"A\"; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "function make_return_bag() { return new ReturnTruthBag(); }\n",
    "class ReturnFactory {\n",
    "    public function fromLocal() { $local = new ReturnTruthBag(); return $local; }\n",
    "    public static function choose($flag) { if ($flag) { return new ReturnTruthBag(); } return new ReturnAltBag(); }\n",
    "}\n",
    "echo make_return_bag()[\"slot\"], \"|\";\n",
    "$assigned = make_return_bag();\n",
    "echo isset($assigned[true]) ? \"Y\" : \"N\", \"|\";\n",
    "$factory = new ReturnFactory();\n",
    "echo $factory->fromLocal()[0], \"|\";\n",
    "echo ReturnFactory::choose(true)[\"slot\"], \"|\";\n",
    "echo ReturnFactory::choose(false)[\"slot\"], \"|\";\n",
    "echo empty(make_return_bag()[0]) ? \"E\" : \"N\";\n",
    "echo \"|\", ReturnFactory::choose(false)[\"slot\"] ?? \"M\";\n",
);

const ARRAYACCESS_DYNAMIC_CALLABLE_RETURN_PRODUCER_FACT_SOURCE: &str = concat!(
    "<?php\n",
    "class DynamicCallableReturnBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { return \"D\"; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "function make_dynamic_callable_return_bag() { return new DynamicCallableReturnBag(); }\n",
    "class DynamicCallableReturnFactory {\n",
    "    public static function make() { return new DynamicCallableReturnBag(); }\n",
    "    public function __invoke() { return new DynamicCallableReturnBag(); }\n",
    "}\n",
    "$function = \"make_dynamic_callable_return_bag\";\n",
    "echo $function()[\"slot\"], \"|\";\n",
    "$static = \"DynamicCallableReturnFactory::make\";\n",
    "echo $static()[\"slot\"], \"|\";\n",
    "$object = new DynamicCallableReturnFactory();\n",
    "echo $object()[\"slot\"], \"|\";\n",
    "$closure = function () { return new DynamicCallableReturnBag(); };\n",
    "$copied = $closure;\n",
    "echo $copied()[\"slot\"];\n",
);

const ARRAYACCESS_CALLABLE_ARRAY_RETURN_PRODUCER_FACT_SOURCE: &str = concat!(
    "<?php\n",
    "class CallableArrayReturnBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { return \"C\"; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class CallableArrayReturnFactory {\n",
    "    public static function make() { return new CallableArrayReturnBag(); }\n",
    "    public function makeObject() { return new CallableArrayReturnBag(); }\n",
    "}\n",
    "$static = [\"CallableArrayReturnFactory\", \"make\"];\n",
    "echo $static()[\"slot\"], \"|\";\n",
    "$object = new CallableArrayReturnFactory();\n",
    "$method = [$object, \"makeObject\"];\n",
    "echo $method()[\"slot\"], \"|\";\n",
    "$keyed = [1 => \"makeObject\", 0 => $object];\n",
    "echo $keyed()[\"slot\"];\n",
);

#[test]
fn native_executable_c_source_routes_arrayaccess_read_isset_through_runtime_abi() {
    let program = parse(ARRAYACCESS_READ_ISSET_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_arrayaccess_offset_read_operation_with_diagnostic")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_GET")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_EXISTS")
            && source.contains(
                "phpc_native_value_new_declared_class_with_relationships_and_diagnostic"
            ),
        "ArrayAccess direct reads and isset should consume the runtime read/exists ABI and declared interface metadata:\n{source}"
    );
    assert!(
        source.contains("phpc_native_callable_table_register_visibility_staticness_frame_callback_and_free")
            && source.contains("phpc_native_callable_table_register_class_parent_and_free")
            && source.contains("_native_callable_frame"),
        "ArrayAccess methods, including inherited public methods, should resolve through callable-table method wrappers:\n{source}"
    );
    assert!(
        !source.contains("ArrayAccess lowering rejects")
            && !source.contains("phpc_native_value_dynamic_call_name_matches")
            && !source.contains("callable_array_matched")
            && !source.contains("callable_object_matched"),
        "ArrayAccess read/isset lowering should not revive the finite dynamic-callable ladder or rejection path:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_arrayaccess_callable_return_producer_facts() {
    let program = parse(ARRAYACCESS_CALLABLE_RETURN_PRODUCER_FACT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_arrayaccess_offset_read_operation_with_diagnostic")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_GET")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_EXISTS")
            && source.contains("user_function_result")
            && source.contains("method_dispatch_result")
            && source.contains("static_method_result"),
        "generated callable return facts should feed existing ArrayAccess read/exists consumers across function, instance method, and static method results:\n{source}"
    );
    assert!(
        !source.contains("ArrayAccess lowering rejects")
            && !source.contains("callable_array_matched")
            && !source.contains("callable_object_matched"),
        "callable return facts must not route through ArrayAccess rejection or finite callable-array/object ladders:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_arrayaccess_dynamic_callable_return_producer_facts() {
    let program = parse(ARRAYACCESS_DYNAMIC_CALLABLE_RETURN_PRODUCER_FACT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source
            .matches("phpc_native_value_arrayaccess_offset_read_operation_with_diagnostic")
            .count()
            >= 4
            && source.contains("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic")
            && source.contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free")
            && source.contains("phpc_native_value_is_descriptor_closure"),
        "dynamic callable return facts should feed ArrayAccess consumers through shared callable identity and runtime invocation boundaries:\n{source}"
    );
    assert!(
        !source.contains("ArrayAccess lowering rejects")
            && !source.contains("callable_array_matched")
            && !source.contains("callable_object_matched")
            && !source.contains("phpc_native_value_dynamic_call_name_matches"),
        "dynamic callable return facts must not use ArrayAccess-specific or legacy finite callable ladders:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_arrayaccess_callable_array_return_producer_facts() {
    let program = parse(ARRAYACCESS_CALLABLE_ARRAY_RETURN_PRODUCER_FACT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source
            .matches("phpc_native_value_arrayaccess_offset_read_operation_with_diagnostic")
            .count()
            >= 3
            && source.contains("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic")
            && source.contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free"),
        "callable-array return facts should feed ArrayAccess consumers through the shared callable identity and runtime invocation boundaries:\n{source}"
    );
    assert!(
        !source.contains("ArrayAccess lowering rejects")
            && !source.contains("phpc_native_value_dynamic_call_name_matches"),
        "callable-array return facts must not use ArrayAccess-specific lowering or the legacy dynamic-call name ladder:\n{source}"
    );
}

#[test]
fn native_executable_c_source_does_not_route_arrayaccess_callable_return_default_fallthrough_fact()
{
    let program = parse(concat!(
        "<?php\n",
        "class MaybeReturnBag implements ArrayAccess {\n",
        "    public function offsetGet($offset) { return \"M\"; }\n",
        "    public function offsetExists($offset) { return true; }\n",
        "    public function offsetSet($offset, $value) { return null; }\n",
        "    public function offsetUnset($offset) { return null; }\n",
        "}\n",
        "function maybe_return_bag($flag) { if ($flag) { return new MaybeReturnBag(); } }\n",
        "echo maybe_return_bag(false)[\"slot\"];\n",
    ))
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        !source.contains("phpc_native_value_arrayaccess_offset_read_operation_with_diagnostic")
            && source.contains("phpc_native_offset_read_source"),
        "callable return facts must be cleared when a generated function can fall through to default null:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_arrayaccess_callable_return_producer_fact_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "arrayaccess_callable_return_producer_fact",
        ARRAYACCESS_CALLABLE_RETURN_PRODUCER_FACT_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native callable-return ArrayAccess producer executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"T|Y|F|T|A|E|A");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_arrayaccess_dynamic_callable_return_producer_fact_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "arrayaccess_dynamic_callable_return_producer_fact",
        ARRAYACCESS_DYNAMIC_CALLABLE_RETURN_PRODUCER_FACT_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native dynamic callable-return ArrayAccess producer executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"D|D|D|D");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_arrayaccess_callable_array_return_producer_fact_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "arrayaccess_callable_array_return_producer_fact",
        ARRAYACCESS_CALLABLE_ARRAY_RETURN_PRODUCER_FACT_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native callable-array-return ArrayAccess producer executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"C|C|C");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_routes_arrayaccess_empty_nullcoalesce_through_runtime_abi() {
    let program = parse(ARRAYACCESS_EMPTY_NULLCOALESCE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_arrayaccess_offset_read_operation_with_diagnostic")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_EXISTS")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_GET")
            && source.contains("phpc_native_value_truthy_with_reference_slot_with_diagnostic")
            && source.contains("phpc_native_value_type_predicate"),
        "ArrayAccess empty/null-coalesce should use the shared exists/read ABI plus PHP truthiness and null checks:\n{source}"
    );
    assert!(
        !source.contains("ArrayAccess lowering rejects")
            && !source.contains("phpc_native_value_dynamic_call_name_matches")
            && !source.contains("callable_array_matched")
            && !source.contains("callable_object_matched"),
        "ArrayAccess empty/null-coalesce lowering should stay on declared-interface metadata and callable-table dispatch:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_arrayaccess_empty_nullcoalesce_runtime_consumer_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "arrayaccess_empty_nullcoalesce_runtime_consumer",
        ARRAYACCESS_EMPTY_NULLCOALESCE_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native ArrayAccess empty/null-coalesce executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"EM|EF|ZT|ST|RM|V|0|0|RN");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_routes_arrayaccess_dynamic_producer_facts() {
    let program = parse(ARRAYACCESS_DYNAMIC_PRODUCER_FACT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_dynamic_call_name_matches")
            && source.contains("phpc_native_value_arrayaccess_offset_read_operation_with_diagnostic")
            && source.contains("phpc_native_value_arrayaccess_offset_write_operation_with_diagnostic")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_GET")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_EXISTS")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_SET")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_APPEND")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_UNSET"),
        "known dynamic declared-object producers should feed the shared ArrayAccess interface fact query for read, isset, write, append, and unset:\n{source}"
    );
    assert!(
        !source.contains("ArrayAccess lowering rejects"),
        "dynamic ArrayAccess producer facts should not fall through the object-offset rejection path:\n{source}"
    );
}

#[test]
fn native_executable_c_source_rejects_arrayaccess_dynamic_unknown_class_name_fact() {
    let program = parse(concat!(
        "<?php\n",
        "class DynamicUnknownBag implements ArrayAccess {\n",
        "    public function offsetGet($offset) { return \"A\"; }\n",
        "    public function offsetExists($offset) { return true; }\n",
        "    public function offsetSet($offset, $value) { return null; }\n",
        "    public function offsetUnset($offset) { return null; }\n",
        "}\n",
        "$class = strtoupper(\"dynamicunknownbag\");\n",
        "echo (new $class())[\"slot\"];\n",
    ))
    .unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert!(
        error.message.contains("ArrayAccess lowering rejects"),
        "dynamic class-name new must not become ArrayAccess without a known generated-candidate interface fact: {error:?}"
    );
}

#[test]
fn emit_exe_links_and_runs_arrayaccess_dynamic_producer_fact_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "arrayaccess_dynamic_producer_fact",
        ARRAYACCESS_DYNAMIC_PRODUCER_FACT_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native dynamic ArrayAccess producer executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        run.stdout,
        b"T|Y|T:set:write=W;T:set:NULL=A;T:unset:write;T|F|T:set:copy=C;T:set:NULL=9;T:unset:;T|T|T"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_arrayaccess_read_isset_runtime_consumer_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "arrayaccess_read_isset_runtime_consumer",
        ARRAYACCESS_READ_ISSET_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native ArrayAccess read/isset executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"T|F|Y|N|T|CT|N|Y");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

const ARRAYACCESS_WRITE_UNSET_SOURCE: &str = concat!(
    "<?php\n",
    "class WriteBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"A:get;\"; return \"g\"; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"A:set:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \"=\", $value, \";\"; return \"ignored\"; }\n",
    "    public function offsetUnset($offset) { echo \"A:unset:\", $offset, \";\"; return \"ignored\"; }\n",
    "}\n",
    "class AuditBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"B:get;\"; return \"g\"; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"B:set:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \"=\", $value, \";\"; return \"ignored\"; }\n",
    "    public function offsetUnset($offset) { echo \"B:unset:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \";\"; return \"ignored\"; }\n",
    "}\n",
    "$first = new WriteBag();\n",
    "$first[\"slot\"] = \"one\";\n",
    "$first[] = \"tail\";\n",
    "unset($first[\"slot\"]);\n",
    "$first[\"again\"] = \"two\";\n",
    "$alias = $first;\n",
    "$alias[\"copy\"] = strtolower(\"COPY\");\n",
    "$alias[] = 4 + 5;\n",
    "unset($alias[0]);\n",
    "$second = new AuditBag();\n",
    "$second[7] = 3;\n",
    "$second[] = strtoupper(\"z\");\n",
    "unset($second[true]);\n",
    "$result = ($second[\"expr\"] = \"rv\");\n",
    "$flags = [\"pick\" => \"1\"];\n",
    "if ($flags[\"pick\"]) {\n",
    "    $joined = new WriteBag();\n",
    "} else {\n",
    "    $joined = new AuditBag();\n",
    "}\n",
    "$joined[\"branch\"] = \"j\";\n",
    "$joined[] = 1 + 1;\n",
    "unset($joined[false]);\n",
    "echo \"result=\", $result;\n",
);

#[test]
fn native_executable_c_source_routes_arrayaccess_write_unset_through_runtime_abi() {
    let program = parse(ARRAYACCESS_WRITE_UNSET_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_arrayaccess_offset_write_operation_with_diagnostic")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_SET")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_APPEND")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_UNSET")
            && source.contains(
                "phpc_native_value_new_declared_class_with_relationships_and_diagnostic"
            ),
        "ArrayAccess direct writes, appends, and unsets should consume the runtime write ABI and declared interface metadata:\n{source}"
    );
    assert!(
        source.contains("phpc_native_callable_table_register_visibility_staticness_frame_callback_and_free")
            && source.contains("_native_callable_frame")
            && !source.contains("ArrayAccess lowering rejects")
            && !source.contains("phpc_native_value_dynamic_call_name_matches"),
        "ArrayAccess write/unset lowering should dispatch through callable-table method wrappers without finite callable recognizers:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_arrayaccess_write_unset_runtime_consumer_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "arrayaccess_write_unset_runtime_consumer",
        ARRAYACCESS_WRITE_UNSET_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native ArrayAccess write/unset executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        run.stdout,
        b"A:set:slot=one;A:set:NULL=tail;A:unset:slot;A:set:again=two;A:set:copy=copy;A:set:NULL=9;A:unset:0;B:set:7=3;B:set:NULL=Z;B:unset:1;B:set:expr=rv;A:set:branch=j;A:set:NULL=2;A:unset:;result=rv"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

const ARRAYACCESS_RMW_NULLCOALESCE_ASSIGNMENT_SOURCE: &str = concat!(
    "<?php\n",
    "class NumberRmwBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"N:get:\", $offset, \";\"; return 4; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"N:set:\", $offset, \"=\", $value, \";\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class ChildNumberRmwBag extends NumberRmwBag {}\n",
    "class TextRmwBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"T:get:\", $offset, \";\"; return \"Hi\"; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"T:set:\", $offset, \"=\", $value, \";\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class MissingAssignBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"M:get;\"; return \"bad\"; }\n",
    "    public function offsetExists($offset) { return false; }\n",
    "    public function offsetSet($offset, $value) { echo \"M:set:\", $offset, \"=\", $value, \";\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class NullAssignBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"U:get:\", $offset, \";\"; return null; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"U:set:\", $offset, \"=\", $value, \";\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class ZeroAssignBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"Z:get:\", $offset, \";\"; return 0; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"Z:set-bad;\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class StringZeroAssignBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"S:get:\", $offset, \";\"; return \"0\"; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"S:set-bad;\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class FalseAssignBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"F:get:\", $offset, \";\"; return false; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"F:set-bad;\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class ValueAssignBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"V:get:\", $offset, \";\"; return \"keep\"; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"V:set-bad;\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "function aa_rhs($value) { echo \"R:\", $value, \";\"; return $value; }\n",
    "$number = new NumberRmwBag();\n",
    "$slot = strtolower(\"SLOT\");\n",
    "$number[$slot] += 2 + 1;\n",
    "$mul = ($number[\"expr\"] *= 2);\n",
    "$text = new TextRmwBag();\n",
    "$text[\"name\"] .= \"there\";\n",
    "$cat = ($text[\"bang\"] .= \"!\");\n",
    "echo \"rmw=\", $mul, \":\", $cat, \";\";\n",
    "$flag = 2 + \"1\";\n",
    "if ($flag) { $joined = new NumberRmwBag(); } else { $joined = new ChildNumberRmwBag(); }\n",
    "$joined[\"branch\"] += 6;\n",
    "$missing = new MissingAssignBag();\n",
    "$nulls = new NullAssignBag();\n",
    "$zero = new ZeroAssignBag();\n",
    "$stringZero = new StringZeroAssignBag();\n",
    "$false = new FalseAssignBag();\n",
    "$value = new ValueAssignBag();\n",
    "echo \"coalesce=\";\n",
    "echo ($missing[\"m\"] ??= aa_rhs(\"miss\")), \"|\";\n",
    "echo ($nulls[\"n\"] ??= aa_rhs(\"null\")), \"|\";\n",
    "echo ($zero[\"z\"] ??= aa_rhs(\"bad\")), \"|\";\n",
    "echo ($stringZero[\"s\"] ??= aa_rhs(\"bad\")), \"|\";\n",
    "echo ($false[\"f\"] ??= aa_rhs(\"bad\")), \"|\";\n",
    "echo ($value[\"v\"] ??= aa_rhs(\"bad\"));\n",
);

const ARRAYACCESS_REFERENCE_SLOT_OWNER_SOURCE: &str = concat!(
    "<?php\n",
    "class ReferenceSlotOwnerBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"get:\", $offset, \";\"; return 5; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"set:\", $offset, \"=\", $value, \";\"; }\n",
    "    public function offsetUnset($offset) { echo \"unset:\", $offset, \";\"; }\n",
    "}\n",
    "function reference_slot_capture_surface() {\n",
    "    $bag = new ReferenceSlotOwnerBag();\n",
    "    $touch = function () use (&$bag) { return 1; };\n",
    "    $bag[\"capture\"] = \"C\";\n",
    "    $bag[\"math\"] += 2;\n",
    "    return $touch();\n",
    "}\n",
    "function reference_slot_global_surface() {\n",
    "    global $globalBag;\n",
    "    $globalBag = new ReferenceSlotOwnerBag();\n",
    "    $globalBag[\"global\"] = \"G\";\n",
    "    $globalBag[\"sum\"] += 3;\n",
    "    return 2;\n",
    "}\n",
    "echo reference_slot_capture_surface(), \"|\", reference_slot_global_surface();\n",
);

const ARRAYACCESS_PROPERTY_HELD_OWNER_SOURCE: &str = concat!(
    "<?php\n",
    "class LiteralHeldBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"L:get:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \";\"; return 4; }\n",
    "    public function offsetExists($offset) { echo \"L:exists:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \";\"; return $offset; }\n",
    "    public function offsetSet($offset, $value) { echo \"L:set:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \"=\", $value, \";\"; }\n",
    "    public function offsetUnset($offset) { echo \"L:unset:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \";\"; }\n",
    "}\n",
    "class DynamicHeldBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"D:get:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \";\"; return 5; }\n",
    "    public function offsetExists($offset) { echo \"D:exists:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \";\"; return $offset; }\n",
    "    public function offsetSet($offset, $value) { echo \"D:set:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \"=\", $value, \";\"; }\n",
    "    public function offsetUnset($offset) { echo \"D:unset:\"; if ($offset) { echo $offset; } else { echo \"NULL\"; } echo \";\"; }\n",
    "}\n",
    "class AlphaHolder { public $first; public $other; public function __construct() { $this->first = new LiteralHeldBag(); } }\n",
    "class BetaHolder { public $dyn; public $extra; public function __construct() { $this->dyn = new DynamicHeldBag(); } }\n",
    "function held_rhs($value) { echo \"R:\", $value, \";\"; return $value; }\n",
    "$alpha = new AlphaHolder();\n",
    "$beta = new BetaHolder();\n",
    "$slot = \"dyn\";\n",
    "echo $alpha->first[\"slot\"], \"|\";\n",
    "echo isset($alpha->first[0]) ? \"Y\" : \"N\", \"|\";\n",
    "echo empty($alpha->first[true]) ? \"E\" : \"N\", \"|\";\n",
    "echo $alpha->first[true] ?? \"M\", \"|\";\n",
    "$alpha->first[\"write\"] = \"W\";\n",
    "unset($alpha->first[\"write\"]);\n",
    "$alpha->first[] = held_rhs(\"append\");\n",
    "echo \"inc=\";\n",
    "echo $alpha->first[\"post\"]++;\n",
    "echo \":\";\n",
    "echo ++$alpha->first[\"pre\"];\n",
    "echo \"|\";\n",
    "$alpha->first[\"rmw\"] += 3;\n",
    "echo ($alpha->first[0] ??= held_rhs(\"assign\")), \"|\";\n",
    "echo $beta->{$slot}[\"slot\"], \"|\";\n",
    "echo isset($beta->{$slot}[true]) ? \"Y\" : \"N\", \"|\";\n",
    "echo empty($beta->{$slot}[true]) ? \"E\" : \"N\", \"|\";\n",
    "echo $beta->{$slot}[0] ?? held_rhs(\"fallback\"), \"|\";\n",
    "$beta->{$slot}[\"write\"] = \"D\";\n",
    "unset($beta->{$slot}[\"write\"]);\n",
    "$beta->{$slot}[] = held_rhs(\"dynappend\");\n",
    "echo \"dinc=\";\n",
    "echo $beta->{$slot}[\"post\"]--;\n",
    "echo \":\";\n",
    "echo --$beta->{$slot}[\"pre\"];\n",
    "echo \"|\";\n",
    "$beta->{$slot}[\"rmw\"] += 2;\n",
);

const ARRAYACCESS_PROPERTY_HELD_INCREMENT_DIAGNOSTIC_SOURCE: &str = concat!(
    "<?php\n",
    "class BadHeldBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"get:\", $offset, \";\"; return \"az\"; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"set-bad;\"; }\n",
    "    public function offsetUnset($offset) { }\n",
    "}\n",
    "class BadHeldHolder { public $bag; public function __construct() { $this->bag = new BadHeldBag(); } }\n",
    "$holder = new BadHeldHolder();\n",
    "$holder->bag[\"bad\"]++;\n",
    "echo \"after\";\n",
);

const ARRAYACCESS_NESTED_OWNER_STACK_PRODUCTION_SOURCE: &str = concat!(
    "<?php\n",
    "class NestedProductionLeafBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"leaf-get:\", $offset, \";\"; return 0; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"leaf-set:\", $offset, \"=\", $value, \";\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class NestedProductionMiddleBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"middle-get:\", $offset, \";\"; return new NestedProductionLeafBag(); }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"middle-set:\", $offset, \";\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class NestedProductionRootBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"root-get:\", $offset, \";\"; return new NestedProductionMiddleBag(); }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"root-set:\", $offset, \";\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class NestedProductionHolder { public $bag; public function __construct() { $this->bag = new NestedProductionRootBag(); } }\n",
    "$direct = new NestedProductionRootBag();\n",
    "$direct[\"outer\"][\"middle\"][\"leaf\"] = \"D\";\n",
    "echo \"|\";\n",
    "$holder = new NestedProductionHolder();\n",
    "$holder->bag[\"pouter\"][\"pmiddle\"][\"pleaf\"] = \"P\";\n",
);

const ARRAYACCESS_NESTED_RMW_OWNER_STACK_SOURCE: &str = concat!(
    "<?php\n",
    "class NestedRmwLeafBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"leaf-get:\", $offset, \";\"; return 4; }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"leaf-set:\", $offset, \"=\", $value, \";\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class NestedRmwMiddleBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"middle-get:\", $offset, \";\"; return new NestedRmwLeafBag(); }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"middle-set:\", $offset, \";\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class NestedRmwRootBag implements ArrayAccess {\n",
    "    public function offsetGet($offset) { echo \"root-get:\", $offset, \";\"; return new NestedRmwMiddleBag(); }\n",
    "    public function offsetExists($offset) { return true; }\n",
    "    public function offsetSet($offset, $value) { echo \"root-set:\", $offset, \";\"; return null; }\n",
    "    public function offsetUnset($offset) { return null; }\n",
    "}\n",
    "class NestedRmwHolder { public $bag; public function __construct() { $this->bag = new NestedRmwRootBag(); } }\n",
    "$direct = new NestedRmwRootBag();\n",
    "$direct[\"outer\"][\"middle\"][\"leaf\"] += 1 + 2;\n",
    "echo \"|\";\n",
    "$holder = new NestedRmwHolder();\n",
    "echo ($holder->bag[\"pouter\"][\"pmiddle\"][\"pleaf\"] *= 1 + 1);\n",
);

#[test]
fn native_executable_c_source_routes_arrayaccess_reference_slot_owners_through_value_owner_boundary(
) {
    let program = parse(ARRAYACCESS_REFERENCE_SLOT_OWNER_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_reference_value_clone(")
            && source.contains("phpc_native_reference_set_value(")
            && source.contains("phpc_native_value_arrayaccess_offset_write_operation_with_diagnostic")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_SET")
            && source.contains("phpc_native_value_binary_result"),
        "closure-promoted and global-import reference slots should share native value owner clone/commit and ArrayAccess write/RMW ABIs:\n{source}"
    );
    assert!(
        source.matches("phpc_native_reference_value_clone(").count() >= 2
            && source.matches("phpc_native_reference_set_value(").count() >= 4,
        "reference-slot owner source/commit should cover both by-reference closure capture promotion and global-import roots:\n{source}"
    );
    assert!(
        !source.contains("ArrayAccess lowering rejects")
            && !source.contains("assembly mutation lowering rejects")
            && !source.contains("assembly global-declaration lowering rejects"),
        "reference-slot owner facts should not fall through exact-shape ArrayAccess or global blockers:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_arrayaccess_reference_slot_owner_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "arrayaccess_reference_slot_owner",
        ARRAYACCESS_REFERENCE_SLOT_OWNER_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native ArrayAccess reference-slot owner executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        run.stdout,
        b"set:capture=C;get:math;set:math=7;1|set:global=G;get:sum;set:sum=8;2"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_routes_property_held_arrayaccess_through_object_property_owner_boundary(
) {
    let program = parse(ARRAYACCESS_PROPERTY_HELD_OWNER_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_public_property_reference_with_diagnostic_and_free")
            && source.contains("phpc_native_reference_value_clone(")
            && source.contains("phpc_native_reference_set_value_with_diagnostic(")
            && source.contains("phpc_native_reference_free(")
            && source.contains("phpc_native_value_arrayaccess_offset_read_operation_with_diagnostic")
            && source.contains("phpc_native_value_arrayaccess_offset_write_operation_with_diagnostic")
            && source.contains("arrayaccess_offset_write_diagnostic")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_GET")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_EXISTS")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_SET")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_UNSET")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_APPEND")
            && source.contains("phpc_native_value_increment_decrement_result")
            && source.contains("PHPC_NATIVE_VALUE_INCREMENT")
            && source.contains("PHPC_NATIVE_VALUE_DECREMENT")
            && source.matches("arrayaccess_offset_unset").count() >= 2
            && source.contains("arrayaccess_offset_null_coalesce_assign"),
        "property-held ArrayAccess owners should use object-property reference owner materialization, read/write/append/unset ArrayAccess ABIs, value increment/decrement diagnostics, and owner commit:\n{source}"
    );
    assert!(
        source.matches("phpc_native_value_public_property_reference_with_diagnostic_and_free").count()
            >= 10
            && source
                .matches("phpc_native_reference_set_value_with_diagnostic(")
                .count()
                >= 6,
        "literal and dynamic property-held owners should repeatedly route through the shared reference owner path:\n{source}"
    );
    assert!(
        !source.contains("ArrayAccess lowering rejects")
            && !source.contains("assembly mutation lowering rejects")
            && !source.contains("non-local assignment lowering rejects"),
        "property-held ArrayAccess lowering must not fall back to exact-shape rejection paths:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_property_held_arrayaccess_owner_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "property_held_arrayaccess_owner",
        ARRAYACCESS_PROPERTY_HELD_OWNER_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native property-held ArrayAccess executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        run.stdout,
        b"L:get:slot;4|L:exists:NULL;N|L:exists:1;L:get:1;N|L:exists:1;L:get:1;4|L:set:write=W;L:unset:write;R:append;L:set:NULL=append;inc=L:get:post;L:set:post=5;4:L:get:pre;L:set:pre=5;5|L:get:rmw;L:set:rmw=7;L:exists:NULL;R:assign;L:set:NULL=assign;assign|D:get:slot;5|D:exists:1;Y|D:exists:1;D:get:1;N|D:exists:NULL;R:fallback;fallback|D:set:write=D;D:unset:write;R:dynappend;D:set:NULL=dynappend;dinc=D:get:post;D:set:post=4;5:D:get:pre;D:set:pre=4;4|D:get:rmw;D:set:rmw=7;"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_reports_property_held_arrayaccess_increment_conversion_diagnostic() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "property_held_arrayaccess_increment_conversion_diagnostic",
        ARRAYACCESS_PROPERTY_HELD_INCREMENT_DIAGNOSTIC_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native property-held ArrayAccess increment diagnostic executable {}: {error}",
            output_path.display()
        )
    });

    assert!(!run.status.success(), "native executable should fail");
    assert_eq!(run.stdout, b"get:bad;");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains(
            "native value increment/decrement supports int, float, and null values in the current native boundary, got string"
        ),
        "stderr:\n{stderr}"
    );
    assert!(
        !String::from_utf8_lossy(&run.stdout).contains("set-bad"),
        "offsetSet must not run after value conversion failure"
    );

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_routes_nested_arrayaccess_owner_stack_for_direct_and_property_roots()
{
    let program = parse(ARRAYACCESS_NESTED_OWNER_STACK_PRODUCTION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_GET")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_SET")
            && source
                .matches("phpc_native_value_arrayaccess_offset_read_operation_with_diagnostic")
                .count()
                >= 4
            && source
                .matches("phpc_native_value_arrayaccess_offset_write_operation_with_diagnostic")
                .count()
                >= 6
            && source.matches("nested_arrayaccess_leaf_write").count() >= 2
            && source.matches("nested_arrayaccess_parent_writeback").count() >= 4
            && source.matches("nested_arrayaccess_root_commit").count() >= 2
            && source.contains("phpc_native_value_public_property_reference_with_diagnostic_and_free")
            && source.contains("phpc_native_reference_set_value("),
        "nested ArrayAccess assignment should emit offsetGet descent, leaf writes, reverse parent writebacks, and direct/property root commits:\n{source}"
    );
    assert!(
        !source.contains("ArrayAccess lowering rejects")
            && !source.contains("assembly mutation lowering rejects")
            && !source.contains("non-local assignment lowering rejects"),
        "nested ArrayAccess production must not fall back to rejection paths:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_nested_arrayaccess_owner_stack_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "nested_arrayaccess_owner_stack_production",
        ARRAYACCESS_NESTED_OWNER_STACK_PRODUCTION_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native nested ArrayAccess executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        run.stdout,
        b"root-get:outer;middle-get:middle;leaf-set:leaf=D;middle-set:middle;root-set:outer;|root-get:pouter;middle-get:pmiddle;leaf-set:pleaf=P;middle-set:pmiddle;root-set:pouter;"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_routes_nested_arrayaccess_rmw_owner_stack_for_direct_and_property_roots(
) {
    let program = parse(ARRAYACCESS_NESTED_RMW_OWNER_STACK_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_GET")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_SET")
            && source.contains("PHPC_NATIVE_VALUE_BINARY_ADD")
            && source.contains("PHPC_NATIVE_VALUE_BINARY_MUL")
            && source.contains("phpc_native_value_binary_result")
            && source
                .matches("phpc_native_value_arrayaccess_offset_read_operation_with_diagnostic")
                .count()
                >= 6
            && source
                .matches("phpc_native_value_arrayaccess_offset_write_operation_with_diagnostic")
                .count()
                >= 6
            && source.matches("nested_arrayaccess_leaf_write").count() >= 2
            && source.matches("nested_arrayaccess_parent_writeback").count() >= 4
            && source.matches("nested_arrayaccess_root_commit").count() >= 2
            && source.contains("phpc_native_value_public_property_reference_with_diagnostic_and_free")
            && source.contains("phpc_native_reference_set_value("),
        "nested ArrayAccess RMW should emit owner-stack descent, leaf reads, native binary updates, reverse parent writebacks, and direct/property root commits:\n{source}"
    );
    assert!(
        !source.contains("ArrayAccess lowering rejects")
            && !source.contains("assembly mutation lowering rejects")
            && !source.contains("non-local assignment lowering rejects"),
        "nested ArrayAccess RMW production must not fall back to rejection paths:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_nested_arrayaccess_rmw_owner_stack_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "nested_arrayaccess_rmw_owner_stack_production",
        ARRAYACCESS_NESTED_RMW_OWNER_STACK_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native nested ArrayAccess RMW executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        run.stdout,
        b"root-get:outer;middle-get:middle;leaf-get:leaf;leaf-set:leaf=7;middle-set:middle;root-set:outer;|root-get:pouter;middle-get:pmiddle;leaf-get:pleaf;leaf-set:pleaf=8;middle-set:pmiddle;root-set:pouter;8"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_rejects_nested_arrayaccess_reference_returning_offsetget() {
    let program = parse(concat!(
        "<?php\n",
        "class RefNestedProductionBag implements ArrayAccess {\n",
        "    public function &offsetGet(&$offset) { return $offset; }\n",
        "    public function offsetExists($offset) { return true; }\n",
        "    public function offsetSet($offset, $value) { return null; }\n",
        "    public function offsetUnset($offset) { return null; }\n",
        "}\n",
        "$bag = new RefNestedProductionBag();\n",
        "$bag[\"outer\"][\"leaf\"] = \"blocked\";\n",
    ))
    .unwrap();
    let error = emit_native_executable_c_source(&program)
        .expect_err("reference-returning offsetGet must not be lowered by value");

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("ArrayAccess lowering rejects"),
        "reference-returning offsetGet should stay blocked at the nested owner boundary: {error:?}"
    );
}

#[test]
fn native_executable_c_source_rejects_nested_arrayaccess_non_assignment_mutations() {
    let sources = [
        (
            "nested ArrayAccess null coalesce assignment",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return new Bag(); }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "$bag = new Bag();\n",
                "$bag[\"outer\"][\"leaf\"] ??= 1;\n",
            ),
        ),
        (
            "nested ArrayAccess append assignment",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return new Bag(); }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "$bag = new Bag();\n",
                "$bag[\"outer\"][] = 1;\n",
            ),
        ),
        (
            "nested ArrayAccess increment",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return new Bag(); }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "$bag = new Bag();\n",
                "$bag[\"outer\"][\"leaf\"]++;\n",
            ),
        ),
        (
            "property-held nested ArrayAccess decrement",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return new Bag(); }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "class Box { public $bag; public function __construct() { $this->bag = new Bag(); } }\n",
                "$box = new Box();\n",
                "$box->bag[\"outer\"][\"leaf\"]--;\n",
            ),
        ),
    ];

    for (label, source) in sources {
        let program = parse(source).unwrap_or_else(|error| {
            panic!("{label} should parse before nested ArrayAccess blocker proof: {error:?}")
        });
        let error = emit_native_executable_c_source(&program)
            .expect_err(&format!("{label} unexpectedly emitted generated C"));

        assert_eq!(error.phase, Phase::Codegen, "{label}: {error:?}");
        assert!(
            error.message.contains("ArrayAccess lowering rejects")
                || error.message.contains("mutation lowering rejects")
                || error
                    .message
                    .contains("non-local assignment lowering rejects"),
            "{label} should remain behind the ArrayAccess/mutation owner boundary, got {error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_rejects_property_held_arrayaccess_increment_unsupported_owner_shapes()
{
    let sources = [
        (
            "unknown dynamic property-held ArrayAccess increment owner",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return 4; }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "class Box { public $bag; public function __construct() { $this->bag = new Bag(); } }\n",
                "$box = new Box();\n",
                "$slot = 1;\n",
                "$box->{$slot}[\"k\"]++;\n",
            ),
        ),
        (
            "nested property-held ArrayAccess increment owner",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return new Bag(); }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "class Box { public $bag; public function __construct() { $this->bag = new Bag(); } }\n",
                "$box = new Box();\n",
                "$box->bag[\"outer\"][\"leaf\"]++;\n",
            ),
        ),
        (
            "reference-returning offsetGet property-held ArrayAccess increment owner",
            concat!(
                "<?php\n",
                "class RefGetBag implements ArrayAccess {\n",
                "    public function &offsetGet(&$offset) { return $offset; }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "class Box { public $bag; public function __construct() { $this->bag = new RefGetBag(); } }\n",
                "$box = new Box();\n",
                "$box->bag[\"k\"]++;\n",
            ),
        ),
    ];

    for (label, source) in sources {
        let program = parse(source).unwrap_or_else(|error| {
            panic!("{label} should parse before codegen boundary proof: {error:?}")
        });
        let error = emit_native_executable_c_source(&program)
            .expect_err(&format!("{label} unexpectedly emitted generated C"));

        assert_eq!(error.phase, Phase::Codegen, "{label}: {error:?}");
        assert!(
            error.message.contains("ArrayAccess lowering rejects")
                || error.message.contains("mutation lowering rejects"),
            "{label} should remain behind the ArrayAccess/mutation owner boundary, got {error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_rejects_property_held_arrayaccess_unset_unsupported_owner_shapes() {
    let sources = [
        (
            "unknown dynamic property-held ArrayAccess unset owner",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return 4; }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "class Box { public $bag; public function __construct() { $this->bag = new Bag(); } }\n",
                "$box = new Box();\n",
                "$slot = 1;\n",
                "unset($box->{$slot}[\"k\"]);\n",
            ),
        ),
        (
            "nested property-held ArrayAccess unset owner",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return new Bag(); }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "class Box { public $bag; public function __construct() { $this->bag = new Bag(); } }\n",
                "$box = new Box();\n",
                "unset($box->bag[\"outer\"][\"leaf\"]);\n",
            ),
        ),
        (
            "non-direct property-held ArrayAccess unset owner",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return 4; }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "class Box { public $bag; public function __construct() { $this->bag = new Bag(); } }\n",
                "class Outer { public $child; public function __construct() { $this->child = new Box(); } }\n",
                "$outer = new Outer();\n",
                "unset($outer->child->bag[\"k\"]);\n",
            ),
        ),
    ];

    for (label, source) in sources {
        let program = parse(source).unwrap_or_else(|error| {
            panic!("{label} should parse before codegen boundary proof: {error:?}")
        });
        let error = emit_native_executable_c_source(&program)
            .expect_err(&format!("{label} unexpectedly emitted generated C"));

        assert_eq!(error.phase, Phase::Codegen, "{label}: {error:?}");
        assert!(
            error.message.contains("ArrayAccess lowering rejects"),
            "{label} should remain behind the ArrayAccess owner boundary, got {error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_arrayaccess_rmw_nullcoalesce_assignment_through_owner_boundary(
) {
    let program = parse(ARRAYACCESS_RMW_NULLCOALESCE_ASSIGNMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_arrayaccess_offset_read_operation_with_diagnostic")
            && source.contains("phpc_native_value_arrayaccess_offset_write_operation_with_diagnostic")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_GET")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_READ_EXISTS")
            && source.contains("PHPC_NATIVE_ARRAYACCESS_OFFSET_WRITE_SET")
            && source.contains("phpc_native_value_binary_result")
            && source.contains("arrayaccess_offset_null_coalesce_assign")
            && source.contains("arrayaccess_offset_assign_subject"),
        "ArrayAccess compound assignment and ??= should share read/write runtime ABIs, native value binary computation, and post-branch subject selection:\n{source}"
    );
    assert!(
        !source.contains("ArrayAccess lowering rejects")
            && !source.contains("phpc_native_value_dynamic_call_name_matches")
            && !source.contains("callable_array_matched")
            && !source.contains("callable_object_matched"),
        "ArrayAccess RMW/??= lowering should stay on declared-interface facts and callable-table dispatch:\n{source}"
    );
}

#[test]
fn native_executable_c_source_rejects_arrayaccess_rmw_unsupported_owner_shapes() {
    let sources = [
        (
            "unknown dynamic property-held ArrayAccess RMW owner",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return 4; }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "class Box { public $bag; }\n",
                "$box = new Box();\n",
                "$slot = 1;\n",
                "$box->{$slot}[\"k\"] += 1;\n",
            ),
        ),
        (
            "nested ArrayAccess RMW owner",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return new Bag(); }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "$bag = new Bag();\n",
                "$bag[\"outer\"][\"leaf\"] += 1;\n",
            ),
        ),
        (
            "append ArrayAccess RMW owner",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return 4; }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "$bag = new Bag();\n",
                "$bag[] += 1;\n",
            ),
        ),
        (
            "increment ArrayAccess RMW owner",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return 4; }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "$bag = new Bag();\n",
                "$bag[\"k\"]++;\n",
            ),
        ),
        (
            "unknown dynamic class-name ArrayAccess RMW owner",
            concat!(
                "<?php\n",
                "class Bag implements ArrayAccess {\n",
                "    public function offsetGet($offset) { return 4; }\n",
                "    public function offsetExists($offset) { return true; }\n",
                "    public function offsetSet($offset, $value) { return null; }\n",
                "    public function offsetUnset($offset) { return null; }\n",
                "}\n",
                "$class = strtoupper(\"bag\");\n",
                "$bag = new $class();\n",
                "$bag[\"k\"] += 1;\n",
            ),
        ),
    ];

    for (label, source) in sources {
        let program = match parse(source) {
            Ok(program) => program,
            Err(error) if label == "append ArrayAccess RMW owner" => {
                assert_eq!(error.phase, Phase::Parse, "{label}: {error:?}");
                assert!(
                    error.message.contains("append offsets")
                        && error.message.contains("unsupported compound assignment target"),
                    "{label} should remain parser-blocked before any exact-shape append RMW lowering, got {error:?}"
                );
                continue;
            }
            Err(error) => panic!("{label} should parse: {error:?}"),
        };
        let error = match emit_native_executable_c_source(&program) {
            Ok(source) => panic!("{label} unexpectedly emitted generated C:\n{source}"),
            Err(error) => error,
        };

        assert_eq!(error.phase, Phase::Codegen, "{label}: {error:?}");
        assert!(
            error.message.contains("ArrayAccess lowering rejects")
                || error.message.contains("mutation lowering rejects")
                || error.message.contains("non-local assignment lowering rejects"),
            "{label} should remain behind a centralized ArrayAccess/mutation/non-local-owner blocker, got {error:?}"
        );
    }
}

#[test]
fn emit_exe_links_and_runs_arrayaccess_rmw_nullcoalesce_assignment_runtime_consumer_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "arrayaccess_rmw_nullcoalesce_assignment_runtime_consumer",
        ARRAYACCESS_RMW_NULLCOALESCE_ASSIGNMENT_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native ArrayAccess RMW/??= executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        run.stdout,
        b"N:get:slot;N:set:slot=7;N:get:expr;N:set:expr=8;T:get:name;T:set:name=Hithere;T:get:bang;T:set:bang=Hi!;rmw=8:Hi!;N:get:branch;N:set:branch=10;coalesce=R:miss;M:set:m=miss;miss|U:get:n;R:null;U:set:n=null;null|Z:get:z;0|S:get:s;0|F:get:f;|V:get:v;keep"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_array_lvalue_compound_assignment_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_lvalue_compound_assignment");
    let source_path = native_link_output_path("array_lvalue_compound_assignment_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_COMPOUND_ASSIGNMENT_SOURCE)
        .expect("native array lvalue compound-assignment source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array lvalue compound source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"20|15|15|5x");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_direct_variable_compound_assignment_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "direct_variable_compound_assignment",
        DIRECT_VARIABLE_COMPOUND_ASSIGNMENT_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native direct variable compound executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"8:8|Ab|18|1:2|10:10:11");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_direct_variable_assignment_expression_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "direct_variable_assignment_expression",
        DIRECT_VARIABLE_ASSIGNMENT_EXPRESSION_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native direct variable assignment-expression executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"2:2|3:4:7|GO:GO|9:9|old|NEW:NEW");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_direct_variable_assignment_expression_native_result_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "direct_variable_assignment_expression_native_result",
        DIRECT_VARIABLE_NATIVE_RESULT_ASSIGNMENT_EXPRESSION_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native direct variable native-result assignment-expression executable: {error}"
        )
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"GO:GO|1:1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_array_compound_assignment_unions_through_value_result_boundary() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "array_compound_assignment_union_value_result_boundary",
        ARRAY_LVALUE_COMPOUND_ARRAY_UNION_SOURCE,
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native array-union executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        run.stdout,
        b"left-zero|right-one|left-name|right-role|nested-left|nested-add"
    );
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_array_to_string_cast_warnings_through_value_result_boundary() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "array_to_string_cast_value_result_boundary",
        ARRAY_CAST_VALUE_RESULT_SOURCE,
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native array-cast executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"Array|Array|Array|2");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(
        stderr
            .matches("Warning: Array to string conversion")
            .count(),
        3
    );
    assert!(
        !stderr.contains("native value cast rejects array-to-string diagnostics")
            && !stderr.contains("assembly cast lowering rejects"),
        "{stderr}"
    );

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_lvalue_increment_decrement_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_lvalue_increment_decrement");
    let source_path = native_link_output_path("array_lvalue_increment_decrement_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_INCREMENT_DECREMENT_SOURCE)
        .expect("native array lvalue increment/decrement source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array lvalue increment/decrement source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"6|6|5|1.5|0.5|6|5");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_lvalue_increment_decrement_missing_slot_recovery() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_lvalue_increment_decrement_missing");
    let source_path =
        native_link_output_path("array_lvalue_increment_decrement_missing_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        ARRAY_LVALUE_INCREMENT_DECREMENT_MISSING_SOURCE,
    )
    .expect("native array lvalue missing-slot increment/decrement source can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path.to_str().expect(
                "native array lvalue missing-slot increment/decrement source path is valid UTF-8",
            ),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"|1|1|1|1||1|1|1");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("undefined array key \"missing\""),
        "{stderr}"
    );
    assert!(stderr.contains("undefined array key \"leaf\""), "{stderr}");
    assert!(stderr.contains("undefined array key \"down\""), "{stderr}");
    assert!(!stderr.contains("nil"), "{stderr}");
    assert!(!stderr.contains("null_leaf"), "{stderr}");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_append_array_lvalue_increment_decrement_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("append_array_lvalue_increment_decrement");
    let source_path = native_link_output_path("append_array_lvalue_increment_decrement_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_APPEND_INCREMENT_DECREMENT_SOURCE)
        .expect("native append array lvalue increment/decrement source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path.to_str().expect(
                "native append array lvalue increment/decrement source path is valid UTF-8",
            ),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|1|1||1||1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_nested_array_lvalue_rmw_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("nested_array_lvalue_rmw");
    let source_path = native_link_output_path("nested_array_lvalue_rmw_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ARRAY_LVALUE_NESTED_RMW_SOURCE)
        .expect("native nested array lvalue RMW source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native nested array lvalue RMW source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"7|20|7|9|20|19|10");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_lvalue_rmw_owner_boundary_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "array_lvalue_rmw_owner_boundary",
        ARRAY_LVALUE_RMW_OWNER_BOUNDARY_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native array-lvalue RMW owner-boundary executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"6|L|5|9:G:3:4|9:G:4");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_read_recovery_result_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_read_recovery_result");
    let source_path = native_link_output_path("array_read_recovery_result_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_READ_RECOVERY_SOURCE)
        .expect("native array read recovery source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array read recovery source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"P||||1");
    let stderr = String::from_utf8_lossy(&run.stderr);
    for expected in [
        "undefined array key \"missing\"",
        "Warning: Trying to access array offset on value of type int",
        "undefined array key \"absent\"",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected:?} in stderr {stderr:?}"
        );
    }

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_offset_null_coalesce_value_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("offset_null_coalesce_value_boundary");
    let source_path = native_link_output_path("offset_null_coalesce_value_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_NULL_COALESCE_SOURCE)
        .expect("native offset null-coalesce source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native offset null-coalesce source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"L|fallback|fallback|b|fallback|N");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_offset_null_coalesce_assign_value_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_offset_null_coalesce_assign_value_boundary");
    let source_path =
        native_link_output_path("array_offset_null_coalesce_assign_value_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_NULL_COALESCE_ASSIGN_SOURCE)
        .expect("native offset null-coalesce assignment source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native offset null-coalesce assignment source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"M|N|K|two|7|M|M|0");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_nested_array_lvalue_null_coalesce_assign_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("nested_array_lvalue_null_coalesce_assign");
    let source_path =
        native_link_output_path("nested_array_lvalue_null_coalesce_assign_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        ARRAY_LVALUE_NESTED_NULL_COALESCE_ASSIGN_SOURCE,
    )
    .expect("native nested lvalue null-coalesce assignment source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native nested lvalue null-coalesce source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"M|N|K|F|NP|7|F|F|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_root_snapshot_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_root_snapshot");
    let source_path = native_link_output_path("request_root_snapshot_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, REQUEST_SUPERGLOBAL_ROOT_SOURCE)
        .expect("native request root source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request root source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|1|array|Array");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_globals_snapshot_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("globals_snapshot");
    let source_path = native_link_output_path("globals_snapshot_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, GLOBALS_SNAPSHOT_SOURCE)
        .expect("native globals snapshot source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native globals snapshot source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native globals snapshot executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A|2|array|array");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_globals_symbol_path_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("globals_symbol_path");
    let source_path = native_link_output_path("globals_symbol_path_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, GLOBALS_SYMBOL_PATH_SOURCE)
        .expect("native globals symbol path source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native globals symbol path source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native globals symbol path executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A|B|10|10");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_globals_symbol_path_write_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("globals_symbol_path_write");
    let source_path = native_link_output_path("globals_symbol_path_write_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, GLOBALS_SYMBOL_PATH_WRITE_SOURCE)
        .expect("native globals symbol path write source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native globals symbol path write source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native globals symbol path write executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A|B|C|11");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_globals_symbol_path_unset_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("globals_symbol_path_unset");
    let source_path = native_link_output_path("globals_symbol_path_unset_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, GLOBALS_SYMBOL_PATH_UNSET_SOURCE)
        .expect("native globals symbol path unset source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native globals symbol path unset source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native globals symbol path unset executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"010KC");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_globals_symbol_path_append_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("globals_symbol_path_append");
    let source_path = native_link_output_path("globals_symbol_path_append_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, GLOBALS_SYMBOL_PATH_APPEND_SOURCE)
        .expect("native globals symbol path append source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native globals symbol path append source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native globals symbol path append executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A|B|M|C|C");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_rejects_globals_direct_root_append_programs() {
    for (name, source) in [
        (
            "globals_direct_root_append_value_rejected",
            GLOBALS_DIRECT_ROOT_APPEND_VALUE_SOURCE,
        ),
        (
            "globals_direct_root_append_reference_rejected",
            GLOBALS_DIRECT_ROOT_APPEND_REFERENCE_TARGET_SOURCE,
        ),
    ] {
        let output_path = native_link_output_path(name);
        let source_path = native_link_output_path(name).with_extension("php");
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(&source_path);
        fs::write(&source_path, source)
            .expect("native globals direct root append source fixture can be written");

        let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
            .args([
                "compile",
                source_path
                    .to_str()
                    .expect("native globals direct root append source path is valid UTF-8"),
                "--emit-exe",
                output_path
                    .to_str()
                    .expect("native globals direct root append executable path is valid UTF-8"),
            ])
            .output()
            .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

        assert!(
            !compile.status.success(),
            "compile unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );
        assert!(
            String::from_utf8_lossy(&compile.stderr).contains(GLOBALS_ROOT_APPEND_REJECTION),
            "compile stderr should report the PHP $GLOBALS append fatal:\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        assert!(
            !output_path.exists(),
            "native executable should not be written"
        );

        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(&source_path);
    }
}

#[test]
fn emit_exe_links_and_runs_direct_root_undefined_read_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("root_symbol_undefined_read");
    let source_path = native_link_output_path("root_symbol_undefined_read_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ROOT_SYMBOL_UNDEFINED_READ_SOURCE)
        .expect("native root undefined read source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native root undefined read source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native root undefined read executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"NULL|ABC");
    let stderr = String::from_utf8_lossy(&run.stderr);
    for name in ["third", "missing", "other", "discarded"] {
        assert!(
            stderr.contains(&format!("undefined variable '${name}'")),
            "{stderr}"
        );
    }

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_direct_symbol_unset_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("direct_symbol_unset");
    let source_path = native_link_output_path("direct_symbol_unset_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, DIRECT_SYMBOL_UNSET_SOURCE)
        .expect("native direct symbol unset source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native direct symbol unset source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native direct symbol unset executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"|1|||C");
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("undefined variable '$first'"),
        "stderr:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_mixed_unset_targets_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("mixed_unset_targets");
    let source_path = native_link_output_path("mixed_unset_targets_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, MIXED_UNSET_TARGETS_SOURCE)
        .expect("native mixed unset targets source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native mixed unset targets source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native mixed unset targets executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"0|1|0|1|0|1|0|G|N");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_root_assignment_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_root_assignment");
    let source_path = native_link_output_path("request_root_assignment_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, REQUEST_SUPERGLOBAL_ROOT_ASSIGNMENT_SOURCE)
        .expect("native request root assignment source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request root assignment source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native request root assignment executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"alpha|42|1|array|SRV");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_unsets_and_reseeds_request_roots_through_shared_state_boundary() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_root_unset");
    let source_path = native_link_output_path("request_root_unset_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, REQUEST_SUPERGLOBAL_ROOT_UNSET_SOURCE)
        .expect("native request root unset source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request root unset source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native request root unset executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|0|1||Ada|array");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_reference_backed_request_root_assignment_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_reference_backed_root_assignment");
    let source_path =
        native_link_output_path("request_reference_backed_root_assignment_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        REQUEST_SUPERGLOBAL_REFERENCE_BACKED_ROOT_ASSIGNMENT_SOURCE,
    )
    .expect("native reference-backed request root assignment source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path.to_str().expect(
                "native reference-backed request root assignment source path is valid UTF-8",
            ),
            "--emit-exe",
            output_path.to_str().expect(
                "native reference-backed request root assignment executable path is valid UTF-8",
            ),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native reference-backed request root assignment executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"ALPHA|ALPHA|array|new|NULL|NULL");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_reference_backed_request_keyed_mutation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_reference_backed_keyed_mutation");
    let source_path = native_link_output_path("request_reference_backed_keyed_mutation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        REQUEST_SUPERGLOBAL_REFERENCE_BACKED_KEYED_MUTATION_SOURCE,
    )
    .expect("native reference-backed request keyed mutation source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path.to_str().expect(
                "native reference-backed request keyed mutation source path is valid UTF-8",
            ),
            "--emit-exe",
            output_path.to_str().expect(
                "native reference-backed request keyed mutation executable path is valid UTF-8",
            ),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native reference-backed request keyed mutation executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native reference-backed request keyed mutation executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"array|Ada|B|0|tail");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_reference_backed_request_keyed_reference_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_reference_backed_keyed_reference");
    let source_path =
        native_link_output_path("request_reference_backed_keyed_reference_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        REQUEST_SUPERGLOBAL_REFERENCE_BACKED_KEYED_REFERENCE_SOURCE,
    )
    .expect("native reference-backed request keyed reference source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path.to_str().expect(
                "native reference-backed request keyed reference source path is valid UTF-8",
            ),
            "--emit-exe",
            output_path.to_str().expect(
                "native reference-backed request keyed reference executable path is valid UTF-8",
            ),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native reference-backed request keyed reference executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native reference-backed request keyed reference executable: {error}")
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"B|S|D|D");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_keyed_storage_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_keyed_storage");
    let source_path = native_link_output_path("request_keyed_storage_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, REQUEST_SUPERGLOBAL_KEYED_STORAGE_SOURCE)
        .expect("native request keyed storage source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request keyed storage source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native request keyed storage executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"Ada|1|0|ANSWER|0");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_keyed_empty_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_keyed_empty");
    let source_path = native_link_output_path("request_keyed_empty_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, REQUEST_SUPERGLOBAL_KEYED_EMPTY_SOURCE)
        .expect("native request keyed empty source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request keyed empty source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native request keyed empty executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|0|1|1|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_path_mutation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_path_mutation");
    let source_path = native_link_output_path("request_path_mutation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, REQUEST_SUPERGLOBAL_PATH_MUTATION_SOURCE)
        .expect("native request path mutation source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request path mutation source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native request path mutation executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"0|0|1|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_path_append_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_path_append");
    let source_path = native_link_output_path("request_path_append_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, REQUEST_SUPERGLOBAL_PATH_APPEND_SOURCE)
        .expect("native request path append source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request path append source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native request path append executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A|B|C|D|D");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_append_suffix_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_append_suffix");
    let source_path = native_link_output_path("request_append_suffix_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, REQUEST_SUPERGLOBAL_APPEND_SUFFIX_SOURCE)
        .expect("native request append suffix source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request append suffix source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native request append suffix executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"G|P|C|N|xyz|xyz");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_path_read_probe_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_path_read_probe");
    let source_path = native_link_output_path("request_path_read_probe_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, REQUEST_SUPERGLOBAL_PATH_READ_PROBE_SOURCE)
        .expect("native request path read/probe source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request path read/probe source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native request path read/probe executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"G|P|1|0|1|1|0");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_assignment_expression_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_assignment_expression");
    let source_path = native_link_output_path("request_assignment_expression_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        REQUEST_SUPERGLOBAL_ASSIGNMENT_EXPRESSION_SOURCE,
    )
    .expect("native request assignment-expression source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request assignment-expression source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native request assignment-expression executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"ADA|42|42|array|ADA|C");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_null_coalesce_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("request_null_coalesce");
    let source_path = native_link_output_path("request_null_coalesce_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, REQUEST_SUPERGLOBAL_NULL_COALESCE_SOURCE)
        .expect("native request null-coalesce source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native request null-coalesce source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native request null-coalesce executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"Ada|fallback|null-fallback|P|xyz|array");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_globals_request_alias_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("globals_request_alias");
    let source_path = native_link_output_path("globals_request_alias_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, GLOBALS_REQUEST_ALIAS_SOURCE)
        .expect("native globals request alias source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native globals request alias source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native globals request alias executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"Ada|P|C|R|11|0|A|xyz|xyz");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_globals_self_request_alias_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("globals_self_request_alias");
    let source_path = native_link_output_path("globals_self_request_alias_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, GLOBALS_SELF_REQUEST_ALIAS_SOURCE)
        .expect("native globals self request alias source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native globals self request alias source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native globals self request alias executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"Ada|P|C|0|A|11|array");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_globals_dynamic_request_root_assignment_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("globals_dynamic_request_root_assignment");
    let source_path = native_link_output_path("globals_dynamic_request_root_assignment_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, GLOBALS_DYNAMIC_REQUEST_ROOT_ASSIGNMENT_SOURCE)
        .expect("native globals dynamic request root assignment source can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path.to_str().expect(
                "native globals dynamic request root assignment source path is valid UTF-8",
            ),
            "--emit-exe",
            output_path.to_str().expect(
                "native globals dynamic request root assignment executable path is valid UTF-8",
            ),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"Ada|P|P|S|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_globals_dynamic_request_root_read_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("globals_dynamic_request_root_read");
    let source_path = native_link_output_path("globals_dynamic_request_root_read_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, GLOBALS_DYNAMIC_REQUEST_ROOT_READ_SOURCE)
        .expect("native globals dynamic request root read source can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native globals dynamic request root read source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native globals dynamic request root read executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"Ada|1|1|1|S|1|0|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_native_value_variable_storage_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("native_value_variable_storage");
    let source_path = native_link_output_path("native_value_variable_storage_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, NATIVE_VALUE_VARIABLE_STORAGE_SOURCE)
        .expect("native value variable storage source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native value variable storage source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"q|q|Q|m|42|Q");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_active_symbol_root_offset_mutation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("active_symbol_root_offset_mutation");
    let source_path = native_link_output_path("active_symbol_root_offset_mutation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, ACTIVE_SYMBOL_ROOT_OFFSET_MUTATION_SOURCE)
        .expect("native active symbol root offset mutation source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native active symbol root offset mutation source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"m|Q|r|seed|B|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_array_offset_unset_lvalue_owner_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("array_offset_unset_lvalue_owner");
    let source_path = native_link_output_path("array_offset_unset_lvalue_owner_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_MUTATION_ARRAY_UNSET_SOURCE)
        .expect("native array offset unset source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native array offset unset source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|0|1|0|D");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_multi_operand_array_offset_unset_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("multi_operand_array_offset_unset");
    let source_path = native_link_output_path("multi_operand_array_offset_unset_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, VALUE_OFFSET_MUTATION_ARRAY_MULTI_UNSET_SOURCE)
        .expect("native multi-operand array unset source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native multi-operand array unset source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|0|1|1|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_offset_read_byte_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_offset_read_byte_boundary");
    let source_path = native_link_output_path("string_offset_read_byte_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, STRING_OFFSET_READ_SOURCE)
        .expect("native string-offset read source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-offset read source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A|\0|1|B");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_offset_write_byte_boundary_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_offset_write_byte_boundary");
    let source_path = native_link_output_path("string_offset_write_byte_boundary_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, STRING_OFFSET_WRITE_SOURCE)
        .expect("native string-offset write source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-offset write source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A\0CD|V\0|A\0C!");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_offset_write_warning_continuation_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_offset_write_warning_continuation");
    let source_path =
        native_link_output_path("string_offset_write_warning_continuation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        "<?php\n$flag = (1 + 2) === 3;\n$s = $flag ? \"ABC\" : \"WXY\";\n$rep = $flag ? \"XY\" : \"Z\";\n$s[1] = $rep;\n$a = [];\n$a[$s] = \"hit\";\necho $s, \"|\", strlen($s), \"|\", $a[$s];\n",
    )
    .expect("native string-offset warning source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native string-offset warning source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"AXC|3|hit");
    assert_eq!(
        String::from_utf8_lossy(&run.stderr),
        "Only the first byte will be assigned to the string offset"
    );

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_reports_shared_filesystem_path_blocker_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("filesystem_path_operation");
    let source_path = native_link_output_path("filesystem_path_operation_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(&source_path, FILESYSTEM_PATH_OPERATION_SOURCE)
        .expect("native filesystem path source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native filesystem path source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "done\n");
    let stderr = String::from_utf8_lossy(&run.stderr);
    for expected in [
        "file_get_contents() awaits the shared filesystem stream ABI",
        "realpath() awaits the shared filesystem canonicalization ABI",
        "file_exists() awaits the shared filesystem stat ABI",
        "is_writable() awaits the shared filesystem stat ABI",
        "filesize() awaits the shared filesystem stat-value ABI",
        "filemtime() awaits the shared filesystem stat-value ABI",
        "getcwd() awaits the shared process current-directory ABI",
        "clearstatcache() awaits the shared filesystem stat-cache ABI",
        "realpath_cache_get() awaits the shared filesystem realpath-cache ABI",
        "realpath_cache_size() awaits the shared filesystem realpath-cache ABI",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected:?} in {stderr:?}"
        );
    }

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_string_integer_argument_conversion_program() {
    if !has_cc() {
        return;
    }

    let output_path = native_link_output_path("string_integer_argument_conversion");
    let source_path = native_link_output_path("string_integer_argument_conversion_source.php");
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
    fs::write(
        &source_path,
        "<?php\n$offset = \"0\";\n$length = 4.0;\n$insert = true;\n$replace = \"1\";\n$delete = 1.0;\necho substr_count(\"aaaa\", \"aa\", $offset, $length);\necho \"\\n\";\necho levenshtein(\"kitten\", \"sitting\", $insert, $replace, $delete);\necho \"\\n\";\n",
    )
    .expect("native int conversion source fixture can be written");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native int conversion source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "2\n3\n");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_quarantines_dynamic_binary_string_comparison_until_conditional_lowering() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("dynamic_binary_string_comparison").with_extension("php");
    fs::write(&temp_php, DYNAMIC_BINARY_STRING_COMPARISON_SOURCE)
        .expect("write temporary dynamic binary string comparison source");
    let output_path = native_link_output_path("dynamic_binary_string_comparison");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile dynamic binary string comparison executable: {error}")
        });

    assert!(
        !compile.status.success(),
        "compile unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(
        String::from_utf8_lossy(&compile.stderr).contains(ASSEMBLY_CONDITIONAL_REJECTION),
        "compile stderr should report the shared conditional lowering blocker:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_runtime_comparison_program() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("runtime_comparison").with_extension("php");
    fs::write(
        &temp_php,
        r#"<?php
echo 1 == "1", "\n";
echo 1 != "2", "\n";
echo 2 < "10", "\n";
echo 2 <= "2", "\n";
echo "10" > 2, "\n";
echo "alpha" >= "alpha", "\n";
echo "10" < "zeta", "\n";
echo "8foo" > "2", "\n";
echo ".5m" < "5.", "\n";
echo "+foo" < "-word", "\n";
echo 2 === 2, "\n";
echo null == false, "\n";
echo 1 !== "1";
"#,
    )
    .expect("write temporary comparison source");
    let output_path = native_link_output_path("runtime_comparison");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native comparison executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native comparison executable: {error}"));

    assert!(run.status.success(), "native comparison executable failed");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_uses_runtime_comparison_results_as_branch_conditions() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("runtime_comparison_branch").with_extension("php");
    fs::write(
        &temp_php,
        r#"<?php
echo ("10" > 2) ? 1 : 0, "\n";
echo (1 != "2") ? 1 : 0, "\n";
echo (2 < "10") ? 1 : 0, "\n";
echo (2 <= "2") ? 1 : 0, "\n";
echo ("alpha" >= "alpha") ? 1 : 0, "\n";
echo (null == false) ? 1 : 0, "\n";
echo (1 !== "1") ? 1 : 0, "\n";
echo (2 === 2) ? 1 : 0;
"#,
    )
    .expect("write temporary comparison branch source");
    let output_path = native_link_output_path("runtime_comparison_branch");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile native comparison branch executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native comparison branch executable: {error}")
    });

    assert!(
        run.status.success(),
        "native comparison branch executable failed"
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "1\n1\n1\n1\n1\n1\n1\n1"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_runs_nested_runtime_comparison_decision_operands() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("nested_runtime_comparison_operands").with_extension("php");
    fs::write(
        &temp_php,
        r#"<?php
$payload = "2";
echo (($payload > 1) == true), "\n";
echo (((1 < 2) == (2 > 1)) ? 1 : 0), "\n";
echo ((null == false) != ("10" < 2));
"#,
    )
    .expect("write temporary nested comparison operand source");
    let output_path = native_link_output_path("nested_runtime_comparison_operands");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile nested comparison operand executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run nested comparison operand executable: {error}")
    });

    assert!(
        run.status.success(),
        "nested comparison operand executable failed"
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1\n1\n1");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

const GENERALIZED_ARRAY_KEY_SOURCE: &str = "<?php\n$slot = \"slot\";\n$two = 2;\n$numeric = \"3\";\n$nil = null;\n$binary = \"A\0B\";\n$a = [$slot => \"text\", $two => \"two\", $numeric => \"three\", $nil => \"null-key\", $binary => \"bin\0ary\", false => \"false-key\", true => \"true-key\", 4.0 => \"float-key\"];\necho $a[$slot], \"\\n\";\necho $a[2], \"\\n\";\necho $a[\"3\"], \"\\n\";\necho $a[$nil], \"\\n\";\necho $a[$binary], \"\\n\";\necho $a[false], \"\\n\";\necho $a[true], \"\\n\";\necho $a[4.0], \"\\n\";\n$a[$slot] = \"updated\";\n$a[$two] = \"two-updated\";\necho $a[\"slot\"], \"\\n\";\necho $a[2], \"\\n\";\n";

const NATIVE_ARRAY_APPEND_SOURCE: &str = "<?php\n$a = [1, \"two\", (string)(2 + 1), null];\necho $a[0], \"|\", $a[1], \"|\", $a[2], \"|\", $a[3];\n";

const NATIVE_VALUE_OPERATION_ARRAY_SOURCE: &str = "<?php\n$a = [];\n$a[\"s\" . \"lot\"] = (2 + 3) * (5 - 1);\n$a[(1 << 2) + 1] = \"fi\" . \"ve\";\n$a[\"neg\"] = -(\"6\" - 2);\necho $a[\"slot\"], \"|\", $a[5], \"|\", $a[\"neg\"];\n";

const NATIVE_VALUE_BITWISE_SOURCE: &str = "<?php\n$a = [];\n$a[\"and\"] = \"B\" & \"A\";\n$a[\"or\"] = \"A\" | \"\0\";\n$a[\"xor\"] = \"A\" ^ \"\0\";\n$a[\"not\"] = ~5;\n$a[\"left\"] = 8 << \"1\";\n$a[\"right\"] = 8 >> 1;\necho $a[\"and\"], \"|\", $a[\"or\"], \"|\", $a[\"xor\"], \"|\", $a[\"not\"], \"|\", $a[\"left\"], \"|\", $a[\"right\"];\n";

const NATIVE_VALUE_COMPARE_CAST_TYPE_NAME_SOURCE: &str = "<?php\n$a = [];\n$a[(int)\"5\"] = (string)((2 + 3) > 4);\n$a[(int)(3 <= 2)] = get_debug_type((string)123);\n$a[(float)\"3\"] = gettype((float)\"3.5\");\necho $a[5], \"|\", $a[0], \"|\", $a[3];\n";

const NATIVE_VALUE_RESULT_STRICT_IDENTITY_SOURCE: &str = "<?php\n$items = [\"word\" => \"go\"];\necho strtoupper($items[\"word\"]) === \"GO\";\necho \"|\";\necho array_sum([1]) !== \"1\";\necho \"|\";\necho strrev(\"ko\") === \"ok\";\necho \"|\";\n$store = [];\n$store[\"same\"] = strtoupper(\"x\") === \"X\";\necho $store[\"same\"];\n";

const NATIVE_VALUE_STRICT_IDENTITY_SOURCE: &str = "<?php\n$a = [\"i\" => 1, \"s\" => \"1\", \"n\" => null];\n$i = $a[\"i\"];\n$s = $a[\"s\"];\n$n = $a[\"n\"];\n$copy = $i;\n$GLOBALS[\"g\"] = 7;\n$_GET[\"id\"] = \"7\";\necho $i === 1;\necho \"|\";\necho $i !== 1;\necho \"|\";\necho $s !== 1;\necho \"|\";\necho $n === null;\necho \"|\";\necho $copy === 1;\necho \"|\";\necho $GLOBALS[\"g\"] === 7;\necho \"|\";\necho $_GET[\"id\"] === \"7\";\n";

const NATIVE_VALUE_TYPE_PREDICATE_SOURCE: &str = "<?php\necho is_int(\"6\" + 1), \"|\";\necho is_float(\"7\" / 2), \"|\";\necho is_string((string)(2 + 3)), \"|\";\necho is_bool((2 + 3) > 4), \"|\";\necho is_array((array)\"x\"), \"|\";\necho is_scalar(gettype((float)\"3.5\")), \"|\";\necho is_numeric((string)(2 + 3)), \"|\";\necho is_countable((array)null), \"|\";\necho is_iterable((array)\"x\"), \"|\";\necho is_null((array)null), \"|\";\necho is_object((array)\"x\");\n";

const NATIVE_STORED_VALUE_TYPE_INTROSPECTION_SOURCE: &str = concat!(
    "<?php\n",
    "$sum = \"6\" + 1;\n",
    "$word = strtoupper(\"go\");\n",
    "$array = (array)\"x\";\n",
    "$flag = $sum > 6;\n",
    "echo is_int($sum), \"|\";\n",
    "echo is_string($word), \"|\";\n",
    "echo is_array($array), \"|\";\n",
    "echo is_bool($flag), \"|\";\n",
    "echo is_numeric($sum), \"|\";\n",
    "echo (true ? gettype($word) : \"bad\"), \"|\";\n",
    "echo (true ? get_debug_type($array) : \"bad\");\n",
);

const NATIVE_STORED_VALUE_ISSET_EMPTY_SOURCE: &str = concat!(
    "<?php\n",
    "$sum = \"6\" + 1;\n",
    "$zero = strtoupper(\"0\");\n",
    "$word = strtoupper(\"go\");\n",
    "$array = (array)null;\n",
    "echo isset($sum) ? \"set\" : \"unset\", \"|\";\n",
    "echo empty($sum) ? \"empty\" : \"filled\", \"|\";\n",
    "echo isset($zero) ? \"set\" : \"unset\", \"|\";\n",
    "echo empty($zero) ? \"empty\" : \"filled\", \"|\";\n",
    "echo empty($word) ? \"empty\" : \"filled\", \"|\";\n",
    "echo isset($array) ? \"set\" : \"unset\", \"|\";\n",
    "echo empty($array) ? \"empty\" : \"filled\";\n",
);

const NATIVE_ARRAY_OWNER_TRUTHINESS_SOURCE: &str = concat!(
    "<?php\n",
    "$empty = [];\n",
    "$filled = [\"x\" => \"y\"];\n",
    "echo empty($empty) ? \"empty\" : \"filled\", \"|\";\n",
    "echo empty($filled) ? \"empty\" : \"filled\", \"|\";\n",
    "if ($empty) { echo \"bad\"; } else { echo \"falsey\"; }\n",
    "echo \"|\";\n",
    "if ($filled && !$empty) { echo \"truthy\"; } else { echo \"bad\"; }\n",
);

const NATIVE_ARRAY_OWNER_OUTPUT_SOURCE: &str = concat!(
    "<?php\n",
    "$empty = [];\n",
    "$filled = [\"x\" => \"y\"];\n",
    "echo $empty, \"|\";\n",
    "print $filled;\n",
);

const NATIVE_VALUE_CAST_ECHO_SOURCE: &str = "<?php\necho (int)\"5.9\", \"|\";\necho (float)\"3.5\", \"|\";\necho (string)(2 + 3), \"|\";\necho (bool)\"0\", \"|\";\necho gettype((string)123);\n";

const NATIVE_VALUE_OPERATION_ECHO_SOURCE: &str = "<?php\n$left = \"6\";\n$right = 2;\necho -$left, \"|\";\necho $left + $right, \"|\";\necho $left / $right, \"|\";\necho \"A\" . \"\0B\", \"|\";\necho \"B\" & \"A\", \"|\";\necho 8 << \"1\";\n";

const NATIVE_VALUE_OPERATION_PRINT_SOURCE: &str = "<?php\n$left = \"6\";\n$right = 2;\n$a = [];\n$a[\"sum\"] = $left + $right;\nprint -$left;\nprint \"|\";\nprint $left + $right;\nprint \"|\";\nprint $left / $right;\nprint \"|\";\nprint \"A\" . \"\0B\";\nprint \"|\";\nprint gettype((string)123);\nprint \"|\";\nprint $a[\"sum\"];\n";

const NATIVE_VALUE_CAST_BUILTIN_SOURCE: &str = "<?php\n$a = [];\n$a[strval(5)] = floatval(\"3.5\");\n$a[\"truth\"] = doubleval(\"2.5\");\necho strval(\"A\"), \"|\", boolval(\"0\"), \"|\", floatval(\" -12.8 \"), \"|\", doubleval(\"2.5\"), \"|\", $a[\"5\"], \"|\", $a[\"truth\"];\n";

const NATIVE_ARRAY_VALUE_OPERAND_SOURCE: &str = "<?php\n$a = [];\n$a[\"nested\"] = [1, 2];\necho (int)[1], \"|\", (int)[], \"|\", (float)[0], \"|\", boolval([0]), \"|\", gettype([1]);\n";

const NATIVE_VALUE_OPERATION_DIAGNOSTIC_SOURCE: &str = concat!(
    "<?php\n",
    "echo [1] + [2];\n",
    "echo (int)[1];\n",
    "echo strrev([1]);\n",
    "echo ~[1];\n",
    "echo gettype([1] + [2]);\n",
);

const NATIVE_VALUE_OPERATION_DIAGNOSTIC_FAILURE_SOURCE: &str =
    "<?php\necho \"before|\";\necho [1] + [2];\necho \"after\";\n";

#[test]
fn native_executable_c_source_routes_array_key_and_value_expressions_through_value_result_abi() {
    let program = parse(NATIVE_VALUE_OPERATION_ARRAY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_NativeValueOperationResult"),
        "{source}"
    );
    assert!(
        source.contains("extern phpc_NativeValueOperationResult phpc_native_value_binary_result"),
        "{source}"
    );
    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_bitwise_operation_with_diagnostic"
        ),
        "{source}"
    );
    assert!(
        source.contains("extern phpc_NativeValueOperationResult phpc_native_value_unary_result"),
        "{source}"
    );
    for op in [
        "PHPC_NATIVE_VALUE_BINARY_CONCAT",
        "PHPC_NATIVE_VALUE_BINARY_ADD",
        "PHPC_NATIVE_VALUE_BINARY_MUL",
        "PHPC_NATIVE_VALUE_BINARY_SUB",
        "PHPC_NATIVE_VALUE_BINARY_SHIFT_LEFT",
        "PHPC_NATIVE_VALUE_UNARY_NEGATE",
    ] {
        assert!(source.contains(op), "{op}\n\n{source}");
    }
    assert!(
        source.contains("PHPC_NATIVE_VALUE_BITWISE_SHIFT_LEFT"),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_binary_result(")
            .count()
            >= 6,
        "{source}"
    );
    assert!(
        source.contains(" = phpc_native_value_bitwise_operation_with_diagnostic("),
        "{source}"
    );
    assert!(
        source.contains(" = phpc_native_value_unary_result("),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_operation_result_free"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_to_array_key")
            && source.contains("phpc_native_array_insert_key_value_with_diagnostic"),
        "value operation results should feed the existing array key/value boundaries:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_bitwise_values_through_shared_value_boundary() {
    let program = parse(NATIVE_VALUE_BITWISE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_bitwise_operation_with_diagnostic"
        ),
        "{source}"
    );
    for op in [
        "PHPC_NATIVE_VALUE_BITWISE_AND",
        "PHPC_NATIVE_VALUE_BITWISE_OR",
        "PHPC_NATIVE_VALUE_BITWISE_XOR",
        "PHPC_NATIVE_VALUE_BITWISE_NOT",
        "PHPC_NATIVE_VALUE_BITWISE_SHIFT_LEFT",
        "PHPC_NATIVE_VALUE_BITWISE_SHIFT_RIGHT",
    ] {
        assert!(source.contains(op), "{op}\n\n{source}");
    }
    assert_eq!(
        source
            .matches(" = phpc_native_value_bitwise_operation_with_diagnostic(")
            .count(),
        6,
        "{source}"
    );
    assert!(
        !source.contains(" = phpc_native_value_binary_result("),
        "{source}"
    );
    assert!(
        !source.contains(" = phpc_native_value_unary_result("),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_insert_key_value_with_diagnostic")
            && source.contains("phpc_native_array_read_key_with_diagnostic"),
        "bitwise values should compose through array write/read boundaries:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_appends_through_diagnostic_boundary() {
    let program = parse(NATIVE_ARRAY_APPEND_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains(
            "extern bool phpc_native_array_append_value_with_diagnostic(phpc_NativeArrayHandle array, phpc_NativeValueHandle value, phpc_NativeDiagnosticHandle *diagnostic);"
        ),
        "{source}"
    );
    assert!(
        source
            .matches("phpc_native_array_append_value_with_diagnostic(")
            .count()
            >= 5,
        "declaration plus every appended value should use the diagnostic append ABI:\n{source}"
    );
    assert!(
        source.contains("array_append_diagnostic_")
            && source.contains("phpc_native_diagnostic_message_stderr(array_append_diagnostic_"),
        "append diagnostics should be reported through the shared diagnostic boundary:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_native_bitwise_value_boundary_program() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("native_bitwise_value_boundary").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_BITWISE_SOURCE)
        .expect("write native bitwise value boundary fixture");
    let output_path = native_link_output_path("native_bitwise_value_boundary");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"@|A|A|-6|16|4");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_native_array_append_diagnostic_program() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("native_array_append_diagnostic").with_extension("php");
    fs::write(&temp_php, NATIVE_ARRAY_APPEND_SOURCE)
        .expect("write native array append diagnostic fixture");
    let output_path = native_link_output_path("native_array_append_diagnostic");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|two|3|");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_native_value_result_array_key_and_value_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_value_result_array_key_value").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_OPERATION_ARRAY_SOURCE)
        .expect("write native value-result array key/value fixture");
    let output_path = native_link_output_path("native_value_result_array_key_value");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"20|five|-4");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn native_executable_c_source_routes_compare_cast_and_type_name_results_through_shared_abi() {
    let program = parse(NATIVE_VALUE_COMPARE_CAST_TYPE_NAME_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    for declaration in [
        "extern phpc_NativeValueOperationResult phpc_native_value_compare_result",
        "extern phpc_NativeValueOperationResult phpc_native_value_cast_result",
        "extern phpc_NativeValueOperationResult phpc_native_value_type_name_result",
    ] {
        assert!(source.contains(declaration), "{declaration}\n\n{source}");
    }

    for op in [
        "PHPC_NATIVE_VALUE_COMPARISON_GT",
        "PHPC_NATIVE_VALUE_COMPARISON_LE",
        "PHPC_NATIVE_VALUE_CAST_STRING",
        "PHPC_NATIVE_VALUE_CAST_INT",
        "PHPC_NATIVE_VALUE_CAST_FLOAT",
        "PHPC_NATIVE_VALUE_TYPE_NAME_GETTYPE",
        "PHPC_NATIVE_VALUE_TYPE_NAME_DEBUG",
    ] {
        assert!(source.contains(op), "{op}\n\n{source}");
    }

    assert!(
        source
            .matches(" = phpc_native_value_compare_result(")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        source.matches(" = phpc_native_value_cast_result(").count() >= 6,
        "{source}"
    );
    assert!(
        !source.contains(" = phpc_native_value_cast_operation_with_diagnostic("),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_type_name_result(")
            .count()
            >= 2,
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_to_array_key")
            && source.contains("phpc_native_array_insert_key_value_with_diagnostic"),
        "compare/cast/type-name results should feed the existing array key/value boundaries:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_value_result_strict_identity_through_compare_result() {
    let program = parse(NATIVE_VALUE_RESULT_STRICT_IDENTITY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("#define PHPC_NATIVE_VALUE_COMPARISON_STRICT_EQ 6")
            && source.contains("#define PHPC_NATIVE_VALUE_COMPARISON_STRICT_NE 7"),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_VALUE_COMPARISON_STRICT_EQ")
            && source.contains("PHPC_NATIVE_VALUE_COMPARISON_STRICT_NE"),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_compare_result(")
            .count()
            >= 4,
        "value-result strict identity should route through the shared value comparison result ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_string_result_operation_with_diagnostic")
            && source
                .contains("phpc_native_value_array_query_operation_with_operands_and_diagnostic"),
        "strict identity operands should reuse existing value-result producers:\n{source}"
    );
    assert!(
        !source.contains("assembly comparison lowering rejects"),
        "{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_native_compare_cast_type_name_result_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_compare_cast_type_name_result").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_COMPARE_CAST_TYPE_NAME_SOURCE)
        .expect("write native compare/cast/type-name value-result fixture");
    let output_path = native_link_output_path("native_compare_cast_type_name_result");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|string|double");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_native_value_result_strict_identity_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "native_value_result_strict_identity",
        NATIVE_VALUE_RESULT_STRICT_IDENTITY_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native value-result strict identity executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"1|1|1|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn native_executable_c_source_routes_native_value_strict_identity_through_comparison_operands() {
    let program = parse(NATIVE_VALUE_STRICT_IDENTITY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("extern phpc_NativeValueHandle phpc_native_value_clone"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_clone(")
            && source.contains("phpc_NativeComparisonOperand")
            && source.contains("phpc_native_comparison_operand_compare_operation_relation_and_free")
            && source
                .contains("phpc_native_comparison_relation_result_decision_or_report_stderr_and_free"),
        "native value handles should be cloned into the shared comparison operand boundary for strict identity:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_comparison_operation_from_opcode(6)")
            .count()
            >= 4
            && source
                .matches("phpc_native_comparison_operation_from_opcode(7)")
                .count()
                >= 2,
        "strict identity and non-identity should both use runtime comparison operations:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_native_value_strict_identity_program() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("native_value_strict_identity").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_STRICT_IDENTITY_SOURCE)
        .expect("write native value strict identity fixture");
    let output_path = native_link_output_path("native_value_strict_identity");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1||1|1|1|1|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn native_executable_c_source_routes_type_predicates_through_value_result_abi() {
    let program = parse(NATIVE_VALUE_TYPE_PREDICATE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains(
            "extern bool phpc_native_value_type_predicate(phpc_NativeValueHandle value, uint8_t predicate);"
        ),
        "{source}"
    );

    for tag in [
        "PHPC_NATIVE_VALUE_TYPE_IS_NULL",
        "PHPC_NATIVE_VALUE_TYPE_IS_BOOL",
        "PHPC_NATIVE_VALUE_TYPE_IS_INT",
        "PHPC_NATIVE_VALUE_TYPE_IS_FLOAT",
        "PHPC_NATIVE_VALUE_TYPE_IS_STRING",
        "PHPC_NATIVE_VALUE_TYPE_IS_ARRAY",
        "PHPC_NATIVE_VALUE_TYPE_IS_SCALAR",
        "PHPC_NATIVE_VALUE_TYPE_IS_NUMERIC",
        "PHPC_NATIVE_VALUE_TYPE_IS_COUNTABLE",
        "PHPC_NATIVE_VALUE_TYPE_IS_ITERABLE",
        "PHPC_NATIVE_VALUE_TYPE_IS_OBJECT",
    ] {
        assert!(source.contains(tag), "{tag}\n\n{source}");
    }

    assert_eq!(
        source
            .matches(" = phpc_native_value_type_predicate(")
            .count(),
        11,
        "{source}"
    );
    assert!(
        source.contains(" = phpc_native_value_binary_result(")
            && source.contains(" = phpc_native_value_compare_result(")
            && source.contains(" = phpc_native_value_cast_result(")
            && source.contains(" = phpc_native_value_type_name_result("),
        "type predicates should consume existing value-result operation, comparison, cast, and type-name materialization:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_stored_value_type_introspection_through_runtime_abi() {
    let program = parse(NATIVE_STORED_VALUE_TYPE_INTROSPECTION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("extern phpc_NativeValueHandle phpc_native_value_clone"),
        "stored native values should be cloned for type-introspection consumers:\n{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_type_predicate(")
            .count()
            >= 5,
        "stored native values should feed the shared type-predicate ABI:\n{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_type_name_result(")
            .count()
            >= 2,
        "stored native values should feed the shared type-name ABI:\n{source}"
    );
    assert!(
        source.contains(" = phpc_native_value_binary_result(")
            && source.contains(" = phpc_native_value_string_result_operation_with_diagnostic(")
            && source.contains(" = phpc_native_value_cast_result(")
            && source.contains(" = phpc_native_value_compare_result("),
        "stored type-introspection should compose with existing operation, string, cast, and comparison value owners:\n{source}"
    );
    assert!(
        !source
            .contains("native direct call lowering rejects this call until return value ownership"),
        "{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_native_type_predicate_value_result_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_type_predicate_value_result").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_TYPE_PREDICATE_SOURCE)
        .expect("write native type-predicate value-result fixture");
    let output_path = native_link_output_path("native_type_predicate_value_result");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|1|1|1|1|1|1|1|1||");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_stored_native_value_type_introspection_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "stored_native_value_type_introspection",
        NATIVE_STORED_VALUE_TYPE_INTROSPECTION_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run stored native value type-introspection executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"1|1|1|1|1|string|array");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn native_executable_c_source_routes_stored_value_isset_empty_through_runtime_abi() {
    let program = parse(NATIVE_STORED_VALUE_ISSET_EMPTY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("PHPC_NATIVE_VALUE_TYPE_IS_NULL")
            && source.contains(" = phpc_native_value_type_predicate("),
        "isset() should classify stored native values through the null type predicate:\n{source}"
    );
    assert!(
        source.contains("extern _Bool phpc_native_value_is_truthy(phpc_NativeValueHandle value);")
            && source.contains(" = phpc_native_value_is_truthy("),
        "empty() should classify stored native values through shared truthiness:\n{source}"
    );
    assert!(
        source.contains(" = phpc_native_value_string_result_operation_with_diagnostic(")
            && source.contains(" = phpc_native_value_binary_result(")
            && source.contains(" = phpc_native_value_cast_result("),
        "isset()/empty() should consume stored native owners from string, arithmetic, and cast value families:\n{source}"
    );
    assert!(
        !source.contains("assembly empty() lowering rejects")
            && !source.contains("assembly isset() lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_native_value_variable_isset_empty_through_null_and_truthiness_boundaries(
) {
    let program = parse(NATIVE_STORED_VALUE_ISSET_EMPTY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source
            .contains("extern _Bool phpc_native_value_truthy_with_reference_slot_with_diagnostic(")
            && source.contains("extern bool phpc_native_value_type_predicate("),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_truthy_with_reference_slot_with_diagnostic(")
            .count()
            >= 3
            && source
                .matches(" = phpc_native_value_type_predicate(")
                .count()
                >= 3
            && !source.contains(" = phpc_native_value_is_truthy(")
            && !source.contains(" = phpc_native_value_truthy_with_diagnostic("),
        "{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_stored_native_value_isset_empty_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "stored_native_value_isset_empty",
        NATIVE_STORED_VALUE_ISSET_EMPTY_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run stored native value isset/empty executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"set|filled|set|empty|filled|set|empty");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn native_executable_c_source_routes_array_owner_truthiness_through_runtime_abi() {
    let program = parse(NATIVE_ARRAY_OWNER_TRUTHINESS_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("extern _Bool phpc_native_value_is_truthy(phpc_NativeValueHandle value);"),
        "{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_is_truthy(")
            .count()
            >= 4,
        "empty(), if-condition, and unary-not array owner truthiness should share the runtime truthiness ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_from_array("),
        "array owners should be materialized as PHP-shaped native values before truthiness checks:\n{source}"
    );
    assert!(
        !source.contains("assembly empty() lowering rejects")
            && !source.contains("assembly unary lowering rejects")
            && !source.contains("assembly control-flow lowering rejects"),
        "{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_array_owner_truthiness_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "array_owner_truthiness",
        NATIVE_ARRAY_OWNER_TRUTHINESS_SOURCE,
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run array owner truthiness executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"empty|filled|falsey|truthy");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn native_executable_c_source_routes_array_owner_output_through_runtime_abi() {
    let program = parse(NATIVE_ARRAY_OWNER_OUTPUT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source
            .contains("extern size_t phpc_native_value_format_stdout_with_diagnostic(phpc_NativeValueHandle value, uint8_t formatter, phpc_NativeDiagnosticHandle *diagnostic);"),
        "{source}"
    );
    assert!(
        source.matches("phpc_native_value_from_array(").count() >= 2
            && source.matches("phpc_native_value_format_stdout_with_diagnostic(").count() >= 3,
        "echo and print should materialize array owners as PHP-shaped values and send them through runtime echo:\n{source}"
    );
    assert!(
        !source.contains("assembly array lowering rejects"),
        "{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_array_owner_output_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) =
        compile_native_link_fixture("array_owner_output", NATIVE_ARRAY_OWNER_OUTPUT_SOURCE);

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run array owner output executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"Array|Array");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn native_executable_c_source_routes_cast_echoes_through_value_cast_operation_abi() {
    let program = parse(NATIVE_VALUE_CAST_ECHO_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("extern phpc_NativeValueOperationResult phpc_native_value_cast_result"),
        "{source}"
    );
    assert!(
        source
            .contains("extern phpc_NativeValueOperationResult phpc_native_value_type_name_result"),
        "{source}"
    );
    assert!(
        source.matches(" = phpc_native_value_cast_result(").count() >= 5,
        "{source}"
    );
    assert!(
        !source.contains(" = phpc_native_value_cast_operation_with_diagnostic("),
        "{source}"
    );
    assert!(
        source.contains(" = phpc_native_value_type_name_result("),
        "{source}"
    );
    assert!(
        source
            .matches("phpc_native_value_format_stdout_with_diagnostic(")
            .count()
            >= 5,
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_VALUE_CAST_STRING")
            && source.contains("PHPC_NATIVE_VALUE_CAST_INT")
            && source.contains("PHPC_NATIVE_VALUE_CAST_BOOL")
            && source.contains("PHPC_NATIVE_VALUE_CAST_FLOAT")
            && source.contains("PHPC_NATIVE_VALUE_TYPE_NAME_GETTYPE"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_operation_result_free"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_operation_echoes_through_value_result_abi() {
    let program = parse(NATIVE_VALUE_OPERATION_ECHO_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    for declaration in [
        "extern phpc_NativeValueOperationResult phpc_native_value_unary_result",
        "extern phpc_NativeValueOperationResult phpc_native_value_binary_result",
        "extern phpc_NativeValueHandle phpc_native_value_bitwise_operation_with_diagnostic",
    ] {
        assert!(source.contains(declaration), "{declaration}\n\n{source}");
    }

    for op in [
        "PHPC_NATIVE_VALUE_UNARY_NEGATE",
        "PHPC_NATIVE_VALUE_BINARY_ADD",
        "PHPC_NATIVE_VALUE_BINARY_DIV",
        "PHPC_NATIVE_VALUE_BINARY_CONCAT",
        "PHPC_NATIVE_VALUE_BITWISE_AND",
        "PHPC_NATIVE_VALUE_BITWISE_SHIFT_LEFT",
    ] {
        assert!(source.contains(op), "{op}\n\n{source}");
    }

    assert!(
        source.contains(" = phpc_native_value_binary_result(")
            && source.contains(" = phpc_native_value_bitwise_operation_with_diagnostic("),
        "operation echoes should use runtime value-operation boundaries where dynamic PHP value semantics are required:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_value_format_stdout_with_diagnostic(")
            .count()
            >= 6,
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_operation_result_free"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_reports_value_operation_diagnostics_through_shared_consumer() {
    let program = parse(NATIVE_VALUE_OPERATION_DIAGNOSTIC_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    for report in [
        "phpc_native_diagnostic_report(native_value_binary_result_",
        "phpc_native_diagnostic_report(native_value_type_name_result_",
        "phpc_native_diagnostic_report(value_cast_diagnostic_",
        "phpc_native_diagnostic_report(string_result_diagnostic_",
        "phpc_native_diagnostic_report(value_bitwise_diagnostic_",
    ] {
        assert!(body.contains(report), "{report}\n\n{source}");
    }

    for old_consumer in [
        "phpc_native_diagnostic_message_stderr(native_value_binary_result_",
        "phpc_native_diagnostic_message_stderr(native_value_type_name_result_",
        "phpc_native_diagnostic_message_stderr(value_cast_diagnostic_",
        "phpc_native_diagnostic_message_stderr(string_result_diagnostic_",
        "phpc_native_diagnostic_message_stderr(value_bitwise_diagnostic_",
    ] {
        assert!(
            !body.contains(old_consumer),
            "value operation diagnostics should not keep the old message/free path:\n{source}"
        );
    }

    assert!(
        body.contains(".diagnostic.ptr = NULL")
            && body.contains("value_cast_diagnostic_")
            && body.contains("string_result_diagnostic_")
            && body.contains("value_bitwise_diagnostic_"),
        "reported diagnostics must be nulled before later cleanup:\n{source}"
    );
    assert_no_diagnostic_report_double_free(&source);
}

#[test]
fn native_executable_c_source_routes_print_values_through_value_result_and_array_read_abi() {
    let program = parse(NATIVE_VALUE_OPERATION_PRINT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    for declaration in [
        "extern phpc_NativeValueOperationResult phpc_native_value_unary_result",
        "extern phpc_NativeValueOperationResult phpc_native_value_binary_result",
        "extern phpc_NativeValueOperationResult phpc_native_value_type_name_result",
        "extern phpc_NativeValueOperationResult phpc_native_value_cast_result",
        "extern phpc_NativeValueHandle phpc_native_value_offset_operation_with_diagnostic",
    ] {
        assert!(source.contains(declaration), "{declaration}\n\n{source}");
    }

    for op in [
        "PHPC_NATIVE_VALUE_UNARY_NEGATE",
        "PHPC_NATIVE_VALUE_BINARY_ADD",
        "PHPC_NATIVE_VALUE_BINARY_DIV",
        "PHPC_NATIVE_VALUE_BINARY_CONCAT",
        "PHPC_NATIVE_VALUE_CAST_STRING",
        "PHPC_NATIVE_VALUE_TYPE_NAME_GETTYPE",
    ] {
        assert!(source.contains(op), "{op}\n\n{source}");
    }

    assert!(
        source.contains(" = phpc_native_value_binary_result(")
            && source.contains(" = phpc_native_value_type_name_result(")
            && source.contains(" = phpc_native_value_cast_result(")
            && source.contains(" = phpc_native_value_offset_operation_with_diagnostic("),
        "print should use the existing runtime value-result and value-offset boundaries:\n{source}"
    );
    assert!(
        source.matches("phpc_native_value_format_stdout_with_diagnostic(").count() >= 8,
        "print output should flow through the value stdout ABI for direct and materialized values:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_scalar_cast_builtins_through_value_cast_contract() {
    let program = parse(NATIVE_VALUE_CAST_BUILTIN_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("extern phpc_NativeValueOperationResult phpc_native_value_cast_result"),
        "{source}"
    );
    assert!(
        source.matches(" = phpc_native_value_cast_result(").count() >= 6,
        "{source}"
    );
    assert!(
        !source.contains(" = phpc_native_value_cast_operation_with_diagnostic("),
        "{source}"
    );
    assert!(
        source.contains("PHPC_NATIVE_VALUE_CAST_STRING")
            && source.contains("PHPC_NATIVE_VALUE_CAST_BOOL")
            && source.contains("PHPC_NATIVE_VALUE_CAST_FLOAT"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_to_array_key")
            && source.contains("phpc_native_array_insert_key_value_with_diagnostic")
            && source.contains("phpc_native_value_format_stdout_with_diagnostic("),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_routes_array_handles_through_value_operand_boundary() {
    let program = parse(NATIVE_ARRAY_VALUE_OPERAND_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains(
            "extern phpc_NativeValueHandle phpc_native_value_from_array(phpc_NativeArrayHandle array);"
        ),
        "{source}"
    );
    assert!(
        source.matches(" = phpc_native_value_from_array(").count() >= 6,
        "array handles used as array values, casts, cast builtins, and type-name operands should share one value materialization boundary:\n{source}"
    );
    let immediate_array_value_frees = source
        .lines()
        .collect::<Vec<_>>()
        .windows(2)
        .filter(|lines| {
            let Some(array_handle) = lines[0]
                .trim()
                .split(" = phpc_native_value_from_array(")
                .nth(1)
                .and_then(|suffix| suffix.strip_suffix(");"))
            else {
                return false;
            };
            let expected = format!("phpc_native_array_free({array_handle});");
            lines[1].trim() == expected.as_str()
        })
        .count();
    assert!(
        immediate_array_value_frees >= 6,
        "temporary array literals cloned into value handles should release the source array immediately:\n{source}"
    );
    assert!(
        source.matches(" = phpc_native_value_cast_result(").count() >= 4,
        "{source}"
    );
    assert!(
        source.contains(" = phpc_native_value_type_name_result("),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_insert_key_value_with_diagnostic(")
            && source.contains("phpc_native_array_append_value_with_diagnostic("),
        "nested array values should compose with keyed insert and append boundaries:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_native_cast_echo_value_result_program() {
    if !has_cc() {
        return;
    }

    let temp_php = native_link_output_path("native_cast_echo_value_result").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_CAST_ECHO_SOURCE)
        .expect("write native cast echo value-result fixture");
    let output_path = native_link_output_path("native_cast_echo_value_result");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"5|3.5|5||string");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_native_operation_echo_value_result_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_operation_echo_value_result").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_OPERATION_ECHO_SOURCE)
        .expect("write native operation echo value-result fixture");
    let output_path = native_link_output_path("native_operation_echo_value_result");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"-6|8|3|A\0B|@|16");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_reports_native_value_operation_diagnostics_once() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "native_value_operation_diagnostic",
        NATIVE_VALUE_OPERATION_DIAGNOSTIC_FAILURE_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native value diagnostic executable: {error}")
    });

    assert!(
        !run.status.success(),
        "native value diagnostic executable should fail"
    );
    assert_eq!(run.stdout, b"before|");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("native value binary operation rejects arrays"),
        "{stderr}"
    );
    assert!(!stderr.contains("after"), "{stderr}");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_native_operation_print_value_result_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_operation_print_value_result").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_OPERATION_PRINT_SOURCE)
        .expect("write native operation print value-result fixture");
    let output_path = native_link_output_path("native_operation_print_value_result");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"-6|8|3|A\0B|string|8");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_native_array_value_operand_boundary_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_array_value_operand_boundary").with_extension("php");
    fs::write(&temp_php, NATIVE_ARRAY_VALUE_OPERAND_SOURCE)
        .expect("write native array value operand fixture");
    let output_path = native_link_output_path("native_array_value_operand_boundary");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"1|0|1|1|array");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn emit_exe_links_and_runs_native_scalar_cast_builtin_boundary_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("native_scalar_cast_builtin_boundary").with_extension("php");
    fs::write(&temp_php, NATIVE_VALUE_CAST_BUILTIN_SOURCE)
        .expect("write native scalar-cast builtin fixture");
    let output_path = native_link_output_path("native_scalar_cast_builtin_boundary");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native executable: {error}"));

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"A||-12.8|2.5|3.5|2.5");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

#[test]
fn native_executable_c_source_routes_array_keys_through_runtime_materialization() {
    let program = parse(GENERALIZED_ARRAY_KEY_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_NativeArrayKeyMaterializationResult"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_to_array_key"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_insert_key_value_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_read_key_with_diagnostic"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_array_key_materialization_result_free"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_value_from_scalar")
            && source.contains("phpc_native_value_from_string_bytes_with_diagnostic"),
        "array keys should enter the same native value materialization boundary for scalar and string families:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_array_read_int("),
        "indexed reads should not bypass generalized key materialization:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_generalized_array_key_materialization_program() {
    if !has_cc() {
        return;
    }

    let temp_php =
        native_link_output_path("generalized_array_key_materialization").with_extension("php");
    fs::write(&temp_php, GENERALIZED_ARRAY_KEY_SOURCE)
        .expect("write generalized native array-key fixture");
    let output_path = native_link_output_path("generalized_array_key_materialization");
    let _ = fs::remove_file(&output_path);

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args([
            "compile",
            temp_php
                .to_str()
                .expect("temporary PHP path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to compile generalized array-key executable: {error}")
        });

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run generalized array-key executable: {error}"));

    assert!(run.status.success(), "native array-key executable failed");
    assert_eq!(
        run.stdout,
        b"text\ntwo\nthree\nnull-key\nbin\0ary\nfalse-key\ntrue-key\nfloat-key\nupdated\ntwo-updated\n"
    );
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&temp_php);
}

const NATIVE_USER_FUNCTION_FRAME_SOURCE: &str = concat!(
    "<?php\n",
    "function pick($value, $fallback = \"D\") {\n",
    "    $local = strtoupper($value);\n",
    "    if ($local) {\n",
    "        return $local;\n",
    "    }\n",
    "    return strtolower($fallback);\n",
    "}\n",
    "function relay($value) {\n",
    "    return pick($value, \"Relay\");\n",
    "}\n",
    "function side($value) {\n",
    "    echo \"side:\", $value, \"|\";\n",
    "}\n",
    "echo pick(\"go\"), \"|\", pick(\"\", \"ALT\"), \"|\", pick(\"\"), \"|\", relay(\"\"), \"|\";\n",
    "echo side(\"effect\");\n",
    "echo \"done\";\n",
);

const NATIVE_IMPORTED_USER_FUNCTION_ALIAS_SOURCE: &str = concat!(
    "<?php\n",
    "namespace Vendor\\Tools;\n",
    "use function Vendor\\Tools\\label as vendor_label, Vendor\\Tools\\set_slot as assign_slot;\n",
    "function label($value) {\n",
    "    return \"label:\" . $value;\n",
    "}\n",
    "function set_slot(&$slot, $value) {\n",
    "    $slot = $value;\n",
    "    return $slot;\n",
    "}\n",
    "$slot = \"old\";\n",
    "echo vendor_label(\"A\"), \"|\";\n",
    "echo assign_slot($slot, \"new\"), \":\", $slot, \"|\";\n",
    "echo label(\"direct\");\n",
);

const NATIVE_IMPORTED_RUNTIME_BUILTIN_ALIAS_SOURCE: &str = concat!(
    "<?php\n",
    "namespace App\\Demo;\n",
    "use function strlen as imported_strlen, str_contains as contains_text, ",
    "strtoupper as upper_text, gettype as imported_gettype;\n",
    "echo imported_strlen(\"abc\"), \"|\";\n",
    "echo contains_text(\"alphabet\", \"pha\"), \"|\";\n",
    "echo upper_text(\"mix\"), \"|\";\n",
    "echo imported_gettype([1]);\n",
);

const NATIVE_IMPORTED_UNSUPPORTED_BUILTIN_ALIAS_SOURCE: &str = concat!(
    "<?php\n",
    "namespace App\\Demo;\n",
    "use function count as imported_count;\n",
    "function count_arg() {\n",
    "    echo \"arg\";\n",
    "    return [1];\n",
    "}\n",
    "echo imported_count(count_arg()), \"after\";\n",
);

const NATIVE_USER_FUNCTION_INTROSPECTION_SOURCE: &str = concat!(
    "<?php\n",
    "function pick($value = \"ok\") {\n",
    "    return $value;\n",
    "}\n",
    "echo function_exists(\"pick\"), \"|\";\n",
    "echo function_exists(\"PICK\"), \"|\";\n",
    "echo is_callable(\"pick\"), \"|\";\n",
    "echo function_exists(\"strlen\"), \"|\";\n",
    "echo function_exists(\"missing_user\"), \"|\";\n",
    "echo pick();\n",
);

const NATIVE_CALLABLE_ARRAY_SYNTAX_SOURCE: &str = concat!(
    "<?php\n",
    "$class = \"CallableBox\";\n",
    "$method = \"run\";\n",
    "$pair = [$class, $method];\n",
    "$badTarget = [42, $method];\n",
    "$badMethod = [$class, 42];\n",
    "$extra = [$class, $method, \"tail\"];\n",
    "$query = array_change_key_case([$class, $method]);\n",
    "echo is_callable($pair, true) ? \"T\" : \"F\";\n",
    "echo \"|\";\n",
    "echo is_callable([\"Inline\", \"go\"], true) ? \"T\" : \"F\";\n",
    "echo \"|\";\n",
    "echo is_callable($query, true) ? \"T\" : \"F\";\n",
    "echo \"|\";\n",
    "echo is_callable($badTarget, true) ? \"T\" : \"F\";\n",
    "echo is_callable($badMethod, true) ? \"T\" : \"F\";\n",
    "echo is_callable($extra, true) ? \"T\" : \"F\";\n",
);

const NATIVE_DYNAMIC_USER_FUNCTION_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "function pick($value = \"ok\") {\n",
    "    return strtoupper($value);\n",
    "}\n",
    "function relay($value) {\n",
    "    return strtolower($value);\n",
    "}\n",
    "$call = \"pick\";\n",
    "echo $call(\"go\"), \"|\";\n",
    "$case = \"PICK\";\n",
    "echo $case(), \"|\";\n",
    "$nested = \"relay\";\n",
    "echo pick($nested(\"mix\")), \"|\";\n",
    "echo (true ? \"pick\" : \"PICK\")(\"tail\");\n",
);

const NATIVE_RUNTIME_DYNAMIC_USER_FUNCTION_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "function pick($value) {\n",
    "    return strtoupper($value);\n",
    "}\n",
    "function relay($value) {\n",
    "    return strtolower($value);\n",
    "}\n",
    "function with_default($value, $suffix = \"!\") {\n",
    "    return $value . $suffix;\n",
    "}\n",
    "function invoke($call, $value) {\n",
    "    return $call($value);\n",
    "}\n",
    "echo invoke(\"pick\", \"go\"), \"|\", invoke(\"relay\", \"MIX\"), \"|\", invoke(\"with_default\", \"D\");\n",
);

const NATIVE_DYNAMIC_STRING_CALLABLE_VALUE_SOURCE: &str = concat!(
    "<?php\n",
    "function upper($value) {\n",
    "    return strtoupper($value);\n",
    "}\n",
    "function lower($value) {\n",
    "    return strtolower($value);\n",
    "}\n",
    "function choose_upper() {\n",
    "    return \"upper\";\n",
    "}\n",
    "function choose_lower() {\n",
    "    return \"lower\";\n",
    "}\n",
    "function call_it($call, $value) {\n",
    "    return $call($value);\n",
    "}\n",
    "$direct = \"upper\";\n",
    "echo $direct(\"go\"), \"|\";\n",
    "$from_return = choose_lower();\n",
    "echo $from_return(\"MIX\"), \"|\";\n",
    "echo call_it(choose_upper(), \"ok\"), \"|\";\n",
    "$builtin = \"strtoupper\";\n",
    "echo $builtin(\"bi\"), \"|\";\n",
    "echo call_it(\"strtolower\", \"CAPS\");\n",
);

const NATIVE_RUNTIME_DYNAMIC_BUILTIN_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "$len = isset($_GET[\"len\"]) ? $_GET[\"len\"] : \"strlen\";\n",
    "$upper = isset($_GET[\"upper\"]) ? $_GET[\"upper\"] : \"strtoupper\";\n",
    "$contains = isset($_GET[\"contains\"]) ? $_GET[\"contains\"] : \"str_contains\";\n",
    "$cast = isset($_GET[\"cast\"]) ? $_GET[\"cast\"] : \"strval\";\n",
    "$type = isset($_GET[\"type\"]) ? $_GET[\"type\"] : \"gettype\";\n",
    "$numeric = isset($_GET[\"numeric\"]) ? $_GET[\"numeric\"] : \"is_numeric\";\n",
    "echo $len(\"A\0B\"), \"|\", $upper(\"go\"), \"|\", $contains(\"abc\", \"b\"), \"|\";\n",
    "echo $cast(7), \"|\", $type([1]), \"|\", $numeric(\"42\");\n",
);

const NATIVE_DYNAMIC_BUILTIN_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "$len = \"strlen\";\n",
    "$upper = \"STRTOUPPER\";\n",
    "$contains = \"str_contains\";\n",
    "$pos = \"strpos\";\n",
    "$type = \"gettype\";\n",
    "$numeric = \"is_numeric\";\n",
    "echo $len(\"A\0B\"), \"|\", $upper(\"go\"), \"|\", $contains(\"abc\", \"b\"), \"|\";\n",
    "echo $pos(\"abcabc\", \"ca\"), \"|\", $type([1]), \"|\", $numeric(\"42\");\n",
);

const NATIVE_MIXED_DYNAMIC_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "function pick($value) {\n",
    "    return \"user:\" . $value;\n",
    "}\n",
    "function wrap($value) {\n",
    "    return \"wrap:\" . $value;\n",
    "}\n",
    "$call = isset($_GET[\"user_alt\"]) ? \"wrap\" : \"pick\";\n",
    "echo $call(\"Go\"), \"|\";\n",
    "$_GET[\"user_alt\"] = \"1\";\n",
    "$call = isset($_GET[\"user_alt\"]) ? \"wrap\" : \"pick\";\n",
    "echo $call(\"Go\"), \"|\";\n",
    "$mixed = isset($_GET[\"builtin_alt\"]) ? \"strtoupper\" : \"pick\";\n",
    "echo $mixed(\"yo\"), \"|\";\n",
    "$_GET[\"builtin_alt\"] = \"1\";\n",
    "$mixed = isset($_GET[\"builtin_alt\"]) ? \"strtoupper\" : \"pick\";\n",
    "echo $mixed(\"yo\"), \"|\";\n",
    "$builtin = isset($_GET[\"type_alt\"]) ? \"gettype\" : \"strlen\";\n",
    "echo $builtin(\"abc\"), \"|\";\n",
    "$_GET[\"type_alt\"] = \"1\";\n",
    "$builtin = isset($_GET[\"type_alt\"]) ? \"gettype\" : \"strlen\";\n",
    "echo $builtin(\"abc\");\n",
);

const NATIVE_RECURSIVE_USER_FUNCTION_FRAME_SOURCE: &str = concat!(
    "<?php\n",
    "function countdown($n) {\n",
    "    if ($n <= 0) {\n",
    "        return \"done\";\n",
    "    }\n",
    "    echo $n, \":\";\n",
    "    return countdown($n - 1);\n",
    "}\n",
    "function even_label($n) {\n",
    "    if ($n <= 0) {\n",
    "        return \"even\";\n",
    "    }\n",
    "    return odd_label($n - 1);\n",
    "}\n",
    "function odd_label($n) {\n",
    "    if ($n <= 0) {\n",
    "        return \"odd\";\n",
    "    }\n",
    "    return even_label($n - 1);\n",
    "}\n",
    "function dynamic_step($n) {\n",
    "    if ($n <= 0) {\n",
    "        return \"dyn\";\n",
    "    }\n",
    "    $call = ($n <= 1) ? \"dynamic_step\" : \"DYNAMIC_STEP\";\n",
    "    return $call($n - 1);\n",
    "}\n",
    "echo countdown(3), \"|\", even_label(4), \":\", even_label(3), \"|\", dynamic_step(2);\n",
);

const NATIVE_TYPED_USER_FUNCTION_FRAME_SOURCE: &str = concat!(
    "<?php\n",
    "function typed_pack(int $value, int|string $tag): string {\n",
    "    return $tag . \":\" . $value;\n",
    "}\n",
    "function nullable(?int $value): ?int {\n",
    "    return $value;\n",
    "}\n",
    "function typed_array(array $items = [\"fallback\"]): string {\n",
    "    return $items[0];\n",
    "}\n",
    "function typed_return(int $value): string {\n",
    "    return $value + 1;\n",
    "}\n",
    "function passthrough(mixed $value): mixed {\n",
    "    return $value;\n",
    "}\n",
    "echo typed_pack(\"12\", 7), \"|\", \"[\", nullable(null), \"]:\", nullable(\"5\"), \"|\";\n",
    "echo typed_array(), \":\", typed_array([\"given\"]), \"|\", typed_return(\"4\"), \"|\", passthrough(\"ok\");\n",
);

const NATIVE_VARIADIC_USER_FUNCTION_FRAME_SOURCE: &str = concat!(
    "<?php\n",
    "function rest_state(...$tail) {\n",
    "    return $tail[0] ?? \"empty\";\n",
    "}\n",
    "function first_extra($head, ...$tail) {\n",
    "    return $tail[0];\n",
    "}\n",
    "function second_extra($head = \"base\", ...$tail) {\n",
    "    return $tail[1];\n",
    "}\n",
    "function typed_variadic(int ...$values): int {\n",
    "    return $values[1];\n",
    "}\n",
    "function call_dynamic($name, $value) {\n",
    "    return $name(\"prefix\", $value, \"last\");\n",
    "}\n",
    "echo rest_state(), \"|\", rest_state(\"filled\"), \"|\";\n",
    "echo first_extra(\"base\", \"A\", \"B\"), \"|\";\n",
    "echo second_extra(\"base\", \"A\", \"B\", \"C\"), \"|\";\n",
    "echo typed_variadic(\"4\", \"5\"), \"|\";\n",
    "$known = \"second_extra\";\n",
    "echo $known(\"base\", \"K\", \"L\"), \"|\";\n",
    "echo call_dynamic(\"first_extra\", \"dyn\");\n",
);

const NATIVE_NAMED_USER_FUNCTION_ARGUMENT_SOURCE: &str = concat!(
    "<?php\n",
    "function named_marker($label) { echo $label; return $label; }\n",
    "function named_join($first, &$slot, $third = \"D\", ...$tail) {\n",
    "    $slot = $slot . \"!\";\n",
    "    return $first . $slot . $third . $tail[\"extra\"];\n",
    "}\n",
    "$slot = \"S\";\n",
    "echo named_join(third: named_marker(\"T\"), extra: named_marker(\"E\"), first: named_marker(\"F\"), slot: $slot), \"|\", $slot;\n",
);

const NATIVE_NAMED_METHOD_SOURCE_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "function named_method_marker($label) { echo $label; return $label; }\n",
    "class NamedMethodCallBox {\n",
    "    public function mix($first, &$slot, $third = \"D\", ...$tail) {\n",
    "        $slot = $slot . \"?\";\n",
    "        return $first . $slot . $third . $tail[\"extra\"];\n",
    "    }\n",
    "}\n",
    "$box = new NamedMethodCallBox();\n",
    "$slot = \"S\";\n",
    "echo $box->mix(third: named_method_marker(\"T\"), extra: named_method_marker(\"E\"), first: named_method_marker(\"F\"), slot: $slot), \"|\", $slot;\n",
);

const NATIVE_NAMED_DYNAMIC_METHOD_SOURCE_CALL_SOURCE: &str = concat!(
    "<?php\n",
    "function named_dynamic_method_marker($label) { echo $label; return $label; }\n",
    "class NamedDynamicMethodCallBox {\n",
    "    public function mix($first, &$slot, $third = \"D\", ...$tail) {\n",
    "        $slot = $slot . \"~\";\n",
    "        return $first . $slot . $third . $tail[\"extra\"];\n",
    "    }\n",
    "}\n",
    "$box = new NamedDynamicMethodCallBox();\n",
    "$slot = \"S\";\n",
    "echo $box->{(true ? \"mix\" : \"mix\")}(third: named_dynamic_method_marker(\"T\"), extra: named_dynamic_method_marker(\"E\"), first: named_dynamic_method_marker(\"F\"), slot: $slot), \"|\", $slot;\n",
);

const NATIVE_BY_REFERENCE_USER_FUNCTION_FRAME_SOURCE: &str = concat!(
    "<?php\n",
    "function set_to(&$slot, $value) {\n",
    "    $slot = $value;\n",
    "    return $slot;\n",
    "}\n",
    "function swap(&$left, &$right) {\n",
    "    $tmp = $left;\n",
    "    $left = $right;\n",
    "    $right = $tmp;\n",
    "    return $left;\n",
    "}\n",
    "$known = \"set_to\";\n",
    "$dyn = \"seed\";\n",
    "echo $known($dyn, \"dynamic\"), \":\", $dyn, \"|\";\n",
    "$name = \"start\";\n",
    "echo set_to($name, \"changed\"), \":\", $name, \"|\";\n",
    "$items = [\"a\" => \"old\", \"b\" => [\"c\" => \"deep\"]];\n",
    "set_to($items[\"a\"], \"new\");\n",
    "echo $items[\"a\"], \"|\";\n",
    "set_to($items[\"b\"][\"c\"], \"deep-new\");\n",
    "echo $items[\"b\"][\"c\"], \"|\";\n",
    "$x = \"left\";\n",
    "$y = \"right\";\n",
    "echo swap($x, $y), \":\", $x, \":\", $y;\n",
);

const NATIVE_RUNTIME_DYNAMIC_BY_REFERENCE_USER_FUNCTION_FRAME_SOURCE: &str = concat!(
    "<?php\n",
    "function set_to(&$slot, $value) {\n",
    "    $slot = $value;\n",
    "    return $slot;\n",
    "}\n",
    "function swap(&$left, &$right) {\n",
    "    $tmp = $left;\n",
    "    $left = $right;\n",
    "    $right = $tmp;\n",
    "    return $left;\n",
    "}\n",
    "function wrap($value) {\n",
    "    return \"wrap:\" . $value;\n",
    "}\n",
    "$slot = \"old\";\n",
    "$set = isset($_GET[\"set\"]) ? $_GET[\"set\"] : \"set_to\";\n",
    "echo $set($slot, \"dynamic\"), \":\", $slot, \"|\";\n",
    "$items = [\"a\" => \"old\", \"b\" => [\"c\" => \"deep\"]];\n",
    "$set($items[\"a\"], \"array\");\n",
    "echo $items[\"a\"], \"|\";\n",
    "$set($items[\"b\"][\"c\"], \"nested\");\n",
    "echo $items[\"b\"][\"c\"], \"|\";\n",
    "$x = \"left\";\n",
    "$y = \"right\";\n",
    "$swap = isset($_GET[\"swap\"]) ? $_GET[\"swap\"] : \"swap\";\n",
    "echo $swap($x, $y), \":\", $x, \":\", $y, \"|\";\n",
    "$wrap = isset($_GET[\"wrap\"]) ? $_GET[\"wrap\"] : \"wrap\";\n",
    "echo $wrap(\"ok\");\n",
);

const NATIVE_SCOPED_CALLABLE_STRING_SIGNATURE_SOURCE: &str = concat!(
    "<?php\n",
    "class ScopedCallableSignature {\n",
    "    public static function mutate(&$slot, $value) {\n",
    "        $slot = $value;\n",
    "        return $slot;\n",
    "    }\n",
    "    public static function &borrow(&$slot) {\n",
    "        return $slot;\n",
    "    }\n",
    "}\n",
    "$slot = \"base\";\n",
    "$variable = \"ScopedCallableSignature::mutate\";\n",
    "echo $variable($slot, \"variable\"), \":\", $slot, \"|\";\n",
    "$concat = \"ScopedCallableSignature\" . \"::\" . \"mutate\";\n",
    "echo $concat($slot, \"concat\"), \":\", $slot, \"|\";\n",
    "$branch = true ? \"ScopedCallableSignature::mutate\" : \"ScopedCallableSignature::mutate\";\n",
    "echo $branch($slot, \"branch\"), \":\", $slot, \"|\";\n",
    "$borrow = \"ScopedCallableSignature::borrow\";\n",
    "$alias =& $borrow($slot);\n",
    "$alias = \"reference\";\n",
    "echo $slot;\n",
);

const NATIVE_GLOBAL_IMPORT_USER_FUNCTION_SOURCE: &str = concat!(
    "<?php\n",
    "$slot = \"root\";\n",
    "$other = \"A\";\n",
    "$bag = [\"k\" => \"root\"];\n",
    "function set_global($value) {\n",
    "    global $slot;\n",
    "    echo \"seen:\", $slot, \"|\";\n",
    "    $local = $value;\n",
    "    $slot = $local;\n",
    "    return $slot;\n",
    "}\n",
    "function swap_globals($left, $right) {\n",
    "    global $slot, $other;\n",
    "    $slot = $left;\n",
    "    $other = $right;\n",
    "    return $slot . $other;\n",
    "}\n",
    "function set_key($value) {\n",
    "    global $bag;\n",
    "    $bag[\"k\"] = $value;\n",
    "    return $bag[\"k\"];\n",
    "}\n",
    "echo set_global(\"direct\"), \":\", $slot, \"|\";\n",
    "echo set_global(\"again\"), \":\", $slot, \"|\";\n",
    "echo set_key(\"G\"), \":\", $bag[\"k\"], \"|\";\n",
    "echo swap_globals(\"S\", \"O\"), \":\", $slot, \":\", $other;\n",
);

const NATIVE_RUNTIME_DYNAMIC_GLOBAL_IMPORT_USER_FUNCTION_SOURCE: &str = concat!(
    "<?php\n",
    "$slot = \"root\";\n",
    "$bag = [\"k\" => \"root\"];\n",
    "function set_global($value) {\n",
    "    global $slot;\n",
    "    echo \"seen:\", $slot, \"|\";\n",
    "    $slot = $value;\n",
    "    return $slot;\n",
    "}\n",
    "function read_global($prefix) {\n",
    "    global $slot;\n",
    "    return $prefix . \":\" . $slot;\n",
    "}\n",
    "function set_key($value) {\n",
    "    global $bag;\n",
    "    $bag[\"k\"] = $value;\n",
    "    return $bag[\"k\"];\n",
    "}\n",
    "function wrap($value) {\n",
    "    return \"wrap:\" . $value;\n",
    "}\n",
    "$runtime = isset($_GET[\"call\"]) ? $_GET[\"call\"] : \"set_global\";\n",
    "echo $runtime(\"dynamic\"), \":\", $slot, \"|\";\n",
    "$runtime = isset($_GET[\"read\"]) ? $_GET[\"read\"] : \"read_global\";\n",
    "echo $runtime(\"seen\"), \"|\";\n",
    "$key_call = isset($_GET[\"key\"]) ? $_GET[\"key\"] : \"set_key\";\n",
    "echo $key_call(\"array\"), \":\", $bag[\"k\"], \"|\";\n",
    "$multi = isset($_GET[\"multi\"]) ? \"wrap\" : \"set_global\";\n",
    "echo $multi(\"finite\"), \":\", $slot, \"|\";\n",
    "$_GET[\"multi\"] = \"1\";\n",
    "$multi = isset($_GET[\"multi\"]) ? \"wrap\" : \"set_global\";\n",
    "echo $multi(\"finite\"), \":\", $slot, \"|\";\n",
    "$mixed = isset($_GET[\"mixed\"]) ? \"strtoupper\" : \"read_global\";\n",
    "echo $mixed(\"mix\"), \"|\";\n",
    "$_GET[\"mixed\"] = \"1\";\n",
    "$mixed = isset($_GET[\"mixed\"]) ? \"strtoupper\" : \"read_global\";\n",
    "echo $mixed(\"mix\");\n",
);

const NATIVE_TRANSITIVE_GLOBAL_IMPORT_USER_FUNCTION_SOURCE: &str = concat!(
    "<?php\n",
    "$slot = \"root\";\n",
    "$bag = [\"k\" => \"root\"];\n",
    "function set_global($value) {\n",
    "    global $slot;\n",
    "    $slot = $value;\n",
    "    return $slot;\n",
    "}\n",
    "function set_key($value) {\n",
    "    global $bag;\n",
    "    $bag[\"k\"] = $value;\n",
    "    return $bag[\"k\"];\n",
    "}\n",
    "function relay_global($value) {\n",
    "    return set_global($value);\n",
    "}\n",
    "function nested_relay($value) {\n",
    "    return relay_global($value);\n",
    "}\n",
    "function relay_key($value) {\n",
    "    return set_key($value);\n",
    "}\n",
    "echo nested_relay(\"wrapped\"), \":\", $slot, \"|\";\n",
    "echo relay_key(\"via\"), \":\", $bag[\"k\"];\n",
);

const NATIVE_GLOBALS_SELF_IMPORT_USER_FUNCTION_SOURCE: &str = concat!(
    "<?php\n",
    "$slot = \"root\";\n",
    "$bag = [\"k\" => \"root\"];\n",
    "function touch_globals($value) {\n",
    "    global $GLOBALS;\n",
    "    echo $GLOBALS[\"slot\"], \"|\";\n",
    "    $GLOBALS[\"slot\"] = $value;\n",
    "    $GLOBALS[\"bag\"][\"k\"] = strtoupper(\"deep\");\n",
    "    echo $GLOBALS[\"slot\"], \":\", $GLOBALS[\"bag\"][\"k\"];\n",
    "}\n",
    "function mix_globals($value) {\n",
    "    global $GLOBALS, $slot;\n",
    "    $GLOBALS[\"slot\"] = $value;\n",
    "    $slot = $slot . \"!\";\n",
    "    echo \"|\", $GLOBALS[\"slot\"], \":\", $slot;\n",
    "}\n",
    "touch_globals(\"changed\");\n",
    "echo \"|\", $slot, \":\", $bag[\"k\"];\n",
    "mix_globals(\"mixed\");\n",
    "echo \"|\", $GLOBALS[\"slot\"];\n",
);

const NATIVE_RUNTIME_DYNAMIC_GLOBALS_SELF_IMPORT_USER_FUNCTION_SOURCE: &str = concat!(
    "<?php\n",
    "$slot = \"root\";\n",
    "function touch_globals($value) {\n",
    "    global $GLOBALS;\n",
    "    $GLOBALS[\"slot\"] = $value;\n",
    "    echo $GLOBALS[\"slot\"];\n",
    "}\n",
    "$call = isset($_GET[\"call\"]) ? $_GET[\"call\"] : \"touch_globals\";\n",
    "$call(\"dynamic\");\n",
    "echo \"|\", $slot;\n",
);

const NATIVE_REQUEST_GLOBAL_FRAME_DIRECT_SOURCE: &str = concat!(
    "<?php\n",
    "$_GET[\"name\"] = \"Ada\";\n",
    "$_POST[\"id\"] = \"root\";\n",
    "function read_request() {\n",
    "    echo $_GET[\"name\"];\n",
    "    return $_POST[\"id\"];\n",
    "}\n",
    "echo read_request();\n",
);

const NATIVE_REQUEST_GLOBAL_FRAME_MUTATION_SOURCE: &str = concat!(
    "<?php\n",
    "function write_post() {\n",
    "    $_POST[\"id\"] = \"B\";\n",
    "}\n",
    "write_post();\n",
    "echo $_POST[\"id\"];\n",
);

const NATIVE_REQUEST_GLOBAL_FRAME_GLOBALS_ALIAS_SOURCE: &str = concat!(
    "<?php\n",
    "function update_request_alias() {\n",
    "    $GLOBALS[\"_GET\"][\"id\"] = \"fn\";\n",
    "}\n",
    "update_request_alias();\n",
    "echo $_GET[\"id\"];\n",
);

const NATIVE_REQUEST_GLOBAL_FRAME_MIXED_SOURCE: &str = concat!(
    "<?php\n",
    "$value = \"root\";\n",
    "function update_both() {\n",
    "    global $value;\n",
    "    $value = \"fn\";\n",
    "    $_GET[\"id\"] = \"G\";\n",
    "}\n",
    "update_both();\n",
    "echo $value, \":\", $_GET[\"id\"];\n",
);

#[test]
fn native_executable_c_source_lowers_direct_user_function_frames() {
    let program = parse(NATIVE_USER_FUNCTION_FRAME_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("static phpc_NativeValueHandle phpc_user_function_0_pick(")
            && source.contains("static phpc_NativeValueHandle phpc_user_function_1_relay(")
            && source.contains("static phpc_NativeValueHandle phpc_user_function_2_side("),
        "top-level functions should lower to reusable C frame entries:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_clone(arg_0)")
            && source.contains("phpc_native_diagnostic_result_terminal_kind_transfer_cleanup_and_free(1")
            && source.contains("phpc_native_diagnostic_result_return_take_value_and_free"),
        "callee frames should clone by-value parameters and hand off owned return handles through terminal results:\n{source}"
    );
    assert!(
        body.contains("phpc_native_call_arguments_new()")
            && body.contains(
                "phpc_native_callable_lookup_invoke_value_with_diagnostic_and_free_arguments"
            )
            && body.contains("phpc_native_diagnostic_result_from_value(user_function_result_"),
        "caller should route direct user functions through source-call arguments and value-result cleanup:\n{source}"
    );
    assert!(
        !body.contains("static phpc_NativeValueHandle"),
        "function definitions must stay outside main:\n{source}"
    );
    assert!(
        !source.contains("assembly user-function lowering rejects"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_exact_imported_user_function_aliases() {
    let program = parse(NATIVE_IMPORTED_USER_FUNCTION_ALIAS_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_user_function_0_vendor_tools_label_native_callable_frame")
            && source.contains("phpc_user_function_1_vendor_tools_set_slot_native_callable_frame"),
        "imported aliases should resolve to registered fully-qualified user-function frames:\n{source}"
    );
    assert!(
        body.contains("phpc_native_callable_lookup_invoke_value_with_diagnostic_and_free_arguments")
            && body.contains("phpc_native_call_arguments_push_value_and_free")
            && body.contains("phpc_native_call_arguments_push_reference_and_free"),
        "imported direct calls should use the shared source-call argument and lookup/invoke stack:\n{source}"
    );
    assert!(
        !source.contains("assembly function-call lowering rejects")
            && !source.contains("assembly user-function lowering rejects"),
        "supported imported user-function aliases should not hit call/frame blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_exact_imported_runtime_builtin_aliases() {
    let program = parse(NATIVE_IMPORTED_RUNTIME_BUILTIN_ALIAS_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);
    let lookup = body
        .find("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic(")
        .expect("imported runtime builtin should perform callable lookup");
    let arguments = body
        .find("phpc_native_call_arguments_new")
        .expect("imported runtime builtin should build source-call arguments");

    assert!(
        lookup < arguments,
        "imported runtime builtin lookup should precede argument construction:\n{source}"
    );
    assert!(
        body.matches("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic(")
            .count()
            >= 4
            && body.matches("phpc_native_callable_value_invoke_value_with_diagnostic_and_free")
                .count()
                >= 4
            && body.contains("phpc_native_call_arguments_push_value_and_free"),
        "imported runtime builtins should use runtime callable lookup/invoke with shared call arguments:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_dynamic_call_name_matches")
            && !source.contains("assembly function-call lowering rejects"),
        "imported runtime builtin aliases should avoid legacy ladders and call blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_preserves_unsupported_imported_builtin_lookup_boundary() {
    let program = parse(NATIVE_IMPORTED_UNSUPPORTED_BUILTIN_ALIAS_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);
    let lookup = body
        .find("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic(")
        .expect("unsupported imported builtin should perform runtime callable lookup");
    let arguments = body.find("phpc_native_call_arguments_new").expect(
        "unsupported imported builtin should still emit argument construction after lookup",
    );

    assert!(
        lookup < arguments,
        "unsupported imported builtin lookup must happen before argument construction:\n{source}"
    );
    assert!(
        body.contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free")
            && !source.contains("phpc_native_value_dynamic_call_name_matches")
            && !source.contains("assembly function-call lowering rejects"),
        "unsupported imported builtins should use the callable lookup boundary without legacy fallback:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_recursive_user_function_frames() {
    let program = parse(NATIVE_RECURSIVE_USER_FUNCTION_FRAME_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("#define PHPC_NATIVE_USER_FUNCTION_MAX_CALL_DEPTH 1024"),
        "recursive frames should emit the shared depth guard:\n{source}"
    );
    assert!(
        source.contains("int phpc_call_depth")
            && source.contains("((phpc_call_depth) + 1)")
            && source.contains("phpc native user-function call depth exceeded"),
        "recursive and in-frame calls should thread the generated call depth:\n{source}"
    );
    assert!(
        source.matches("phpc_user_function_0_countdown(").count() >= 3
            && source.matches("phpc_user_function_1_even_label(").count() >= 3
            && source.matches("phpc_user_function_2_odd_label(").count() >= 3
            && source.matches("phpc_user_function_3_dynamic_step(").count() >= 3,
        "direct recursion, mutual recursion, and known-string in-frame dynamic recursion should lower through frame entries:\n{source}"
    );
    assert!(
        !source.contains("assembly user-function lowering rejects")
            && !source.contains("assembly dynamic function-call lowering rejects"),
        "recursive frames and in-frame known dynamic calls should not hit call blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_enforces_user_function_type_metadata() {
    let program = parse(NATIVE_TYPED_USER_FUNCTION_FRAME_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_coerce_call_type_with_diagnostic"),
        "typed parameters and returns should route through the shared call-frame type ABI:\n{source}"
    );
    assert!(
        source
            .matches(" = phpc_native_value_coerce_call_type_with_diagnostic(")
            .count()
            >= 7,
        "parameters, defaults, and return values should all consume the type ABI:\n{source}"
    );
    assert!(
        source.contains("call_type_decl_bytes_")
            && source.contains("call_type_label_bytes_")
            && source.contains("call_type_callable_bytes_"),
        "type metadata should be passed as data, not source-shape recognition:\n{source}"
    );
    assert!(
        !source.contains("assembly user-function lowering rejects")
            && !source.contains("unsupported parameter or return type metadata"),
        "supported scalar/array type metadata should no longer hit frame blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_variadic_user_function_frames() {
    let program = parse(NATIVE_VARIADIC_USER_FUNCTION_FRAME_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_array_empty")
            && source.contains("phpc_native_array_append_value_with_diagnostic")
            && source.contains("phpc_native_value_from_array"),
        "variadic frame calls should pack surplus arguments through the shared native array/value ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_coerce_call_type_with_diagnostic"),
        "typed variadic arguments should consume the shared call-frame type ABI per supplied value:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_dynamic_call_name_matches")
            && source.contains("phpc_user_function_1_first_extra("),
        "runtime dynamic calls should dispatch into registered variadic frames:\n{source}"
    );
    assert!(
        !source.contains("assembly user-function lowering rejects")
            && !source.contains("assembly dynamic function-call lowering rejects")
            && !source.contains("bounded generated-C frame subset"),
        "supported by-value variadic frames should not hit declaration or call blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_named_user_function_arguments_through_shared_normalization() {
    let program = parse(NATIVE_NAMED_USER_FUNCTION_ARGUMENT_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("phpc_native_array_insert_key_value_with_diagnostic")
            && body.contains("phpc_native_call_arguments_push_reference_and_free")
            && body.contains("phpc_native_callable_lookup_invoke_value_with_diagnostic_and_free_arguments"),
        "named direct user-function calls should collect named variadics, propagate references, and bind through the runtime callable ABI:\n{source}"
    );
    assert!(
        !source.contains("named argument lowering is only implemented"),
        "{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_named_method_source_call_arguments_through_carriers() {
    let program = parse(NATIVE_NAMED_METHOD_SOURCE_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains(
            "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
        ) && body.contains("receiver_method_source_call_args_")
            && body.contains("phpc_native_array_insert_key_value_with_diagnostic")
            && body.contains("phpc_native_call_arguments_push_reference_and_free"),
        "named receiver-method calls should bind through shared source-call carriers with named variadic and by-reference slots:\n{source}"
    );
    assert!(
        !body.contains("method_dispatch_status")
            && !source.contains("named argument lowering is only implemented"),
        "named receiver-method source calls should not fall back to generated frame ladders:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_named_dynamic_method_source_call_arguments_through_carriers() {
    let program = parse(NATIVE_NAMED_DYNAMIC_METHOD_SOURCE_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains(
            "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
        ) && body.contains("dynamic_receiver_method_source_call_args_")
            && body.contains("phpc_native_array_insert_key_value_with_diagnostic")
            && body.contains("phpc_native_call_arguments_push_reference_and_free"),
        "named dynamic receiver-method calls should bind through shared source-call carriers with named variadic and by-reference slots:\n{source}"
    );
    assert!(
        !body.contains("dynamic_method_dispatch_status")
            && !body.contains("phpc_native_value_dynamic_method_name_matches")
            && !source.contains("named argument lowering is only implemented"),
        "named dynamic receiver-method source calls should not fall back to generated dynamic-dispatch ladders:\n{source}"
    );
}

#[test]
fn native_executable_c_source_blocks_named_dynamic_method_fallback_without_shared_contract() {
    let program = parse(concat!(
        "<?php\n",
        "class NamedDynamicMethodMagicBox {\n",
        "    public function __call($name, $args) { return \"magic\"; }\n",
        "    public function known($value) { return $value; }\n",
        "}\n",
        "$box = new NamedDynamicMethodMagicBox();\n",
        "$method = \"known\";\n",
        "echo $box->{$method}(value: \"x\");\n",
    ))
    .unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error
            .message
            .contains("named argument lowering is only implemented"),
        "{}",
        error.message
    );
}

#[test]
fn native_executable_c_source_lowers_by_reference_user_function_frames() {
    let program = parse(NATIVE_BY_REFERENCE_USER_FUNCTION_FRAME_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_NativeReferenceHandle arg_0")
            && source.contains("phpc_native_reference_clone(arg_0)")
            && source.contains("phpc_native_reference_set_value")
            && source.contains("phpc_native_symbol_table_reference_for_path"),
        "by-reference frame entries should bind reference handles through shared symbol/reference ABI:\n{source}"
    );
    assert!(
        source.matches("phpc_native_symbol_table_reference_for_path(").count() >= 4
            && source.contains("phpc_user_function_0_set_to(")
            && source.contains("phpc_user_function_1_swap("),
        "direct variables, nested array slots, and known dynamic calls should reuse the same reference path:\n{source}"
    );
    assert!(
        !source.contains("assembly user-function lowering rejects")
            && !source.contains("unsupported typed/default/variadic by-reference parameters"),
        "supported untyped by-reference frames should not hit declaration blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_function_scope_global_imports() {
    let program = parse(NATIVE_GLOBAL_IMPORT_USER_FUNCTION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_NativeSymbolTableHandle phpc_root_symbols")
            && source.contains("phpc_native_symbol_table_reference_for_path")
            && source.contains("global_import_ref_"),
        "function-scope global imports should bind local variables to root symbol references:\n{source}"
    );
    assert!(
        body.contains("phpc_native_symbol_table_new()")
            && body.contains("phpc_user_function_0_set_global(")
            && body.contains("phpc_user_function_1_swap_globals("),
        "callers should materialize one shared root symbol table for global-import frames:\n{source}"
    );
    assert!(
        source.matches("phpc_native_reference_set_value").count() >= 3
            && source.matches("phpc_native_reference_value_clone").count() >= 2,
        "global-import variables should read and write through the shared reference ABI:\n{source}"
    );
    assert!(
        source.contains("array_lvalue_symbol_write_result")
            && source.contains("phpc_native_array_lvalue_owner_reference_slot"),
        "global-import array paths should reuse symbol-table array lvalue owners:\n{source}"
    );
    assert!(
        !source.contains("assembly global-declaration lowering rejects")
            && !source.contains("assembly user-function lowering rejects"),
        "ordinary function-scope global imports should not hit global/frame blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_known_string_dynamic_user_function_calls() {
    let program = parse(NATIVE_DYNAMIC_USER_FUNCTION_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("static phpc_NativeValueHandle phpc_user_function_0_pick(")
            && source.contains("static phpc_NativeValueHandle phpc_user_function_1_relay("),
        "registered dynamic call targets should still lower to reusable frames:\n{source}"
    );
    assert!(
        body.matches("phpc_user_function_0_pick(").count() >= 3
            && body.contains("phpc_user_function_1_relay("),
        "known-string dynamic calls should dispatch through registered frame entries:\n{source}"
    );
    assert!(
        !source.contains("assembly dynamic function-call lowering rejects")
            && !source.contains("assembly function-call lowering rejects"),
        "registered known-string dynamic calls should not hit call blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_runtime_string_dynamic_user_function_calls() {
    let program = parse(NATIVE_RUNTIME_DYNAMIC_USER_FUNCTION_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_NativeCallableValueHandle")
            && source.contains(
                "phpc_native_callable_lookup_value_or_closure_with_context_diagnostic"
            )
            && source
                .contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free")
            && source.contains("phpc_native_call_arguments_push_value_and_free"),
        "runtime string-valued dynamic user-function calls should use the shared callable-value lookup/invoke ABI:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic(")
            .count()
            >= 1,
        "runtime dynamic calls should consume a runtime callable value at each dynamic call boundary:\n{source}"
    );
    assert!(
        source.contains("phpc_user_function_0_pick_native_callable_frame")
            && source.contains("phpc_user_function_1_relay_native_callable_frame")
            && source.contains("phpc_user_function_2_with_default_native_callable_frame"),
        "runtime dynamic dispatch should expose registered user functions through reusable callable frame callbacks:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_dynamic_call_name_matches")
            && !source.contains("dynamic_user_function_matched_")
            && !source.contains("dynamic_call_reason_bytes_"),
        "runtime string-valued dynamic calls should not use the legacy generated-C name-match ladder:\n{source}"
    );
    assert!(
        !source.contains("assembly dynamic function-call lowering rejects")
            && !source.contains("bounded generated-C finite known-string or runtime string-valued dispatch"),
        "runtime string-valued dynamic calls should no longer hit the finite-known-string blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_dynamic_callable_values_through_runtime_abi() {
    let program = parse(NATIVE_DYNAMIC_STRING_CALLABLE_VALUE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_NativeCallableValueHandle")
            && source.contains(
                "phpc_native_callable_lookup_value_or_closure_with_context_diagnostic"
            )
            && source
                .contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free")
            && source.contains("phpc_native_call_arguments_push_reference_and_free")
            && source.contains("phpc_native_call_arguments_push_value_and_free"),
        "dynamic callable values should route through the callable-value lookup/invoke and shared call-argument ABI:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic(")
            .count()
            >= 3,
        "each dynamic callable value call site should consume the runtime lookup boundary:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_dynamic_call_name_matches")
            && !source.contains("dynamic_user_function_matched_")
            && !source.contains("dynamic_call_reason_bytes_"),
        "dynamic callable values should not use the legacy generated-C name-match ladder:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_runtime_dynamic_global_import_user_function_calls() {
    let program = parse(NATIVE_RUNTIME_DYNAMIC_GLOBAL_IMPORT_USER_FUNCTION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains("phpc_NativeSymbolTableHandle phpc_root_symbols")
            && source.contains("phpc_native_value_dynamic_call_name_matches")
            && source.contains("dynamic_user_function_matched_"),
        "runtime dynamic global-import dispatch should reuse the frame root-symbol and dynamic lookup ABIs:\n{source}"
    );
    assert!(
        body.contains("phpc_native_symbol_table_new()")
            && body.contains("phpc_user_function_0_set_global(")
            && body.contains("phpc_user_function_1_read_global(")
            && body.contains("phpc_user_function_2_set_key(")
            && body.contains("phpc_user_function_3_wrap(")
            && body.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "runtime dispatch should cover global-import frames, ordinary frames, and mixed builtin branches through one lookup table:\n{source}"
    );
    assert!(
        body.matches("phpc_native_value_dynamic_call_name_matches(").count() >= 5,
        "dynamic global-import dispatch should stay table-driven across callable families:\n{source}"
    );
    assert!(
        !source.contains("assembly dynamic function-call lowering rejects")
            && !source.contains("assembly global-declaration lowering rejects"),
        "ordinary global-import frames should no longer be excluded from runtime dynamic dispatch:\n{source}"
    );
}

#[test]
fn native_executable_c_source_threads_global_import_roots_through_wrapper_frames() {
    let program = parse(NATIVE_TRANSITIVE_GLOBAL_IMPORT_USER_FUNCTION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "phpc_user_function_2_relay_global(int phpc_call_depth, phpc_NativeSymbolTableHandle phpc_root_symbols"
        ) && source.contains(
            "phpc_user_function_3_nested_relay(int phpc_call_depth, phpc_NativeSymbolTableHandle phpc_root_symbols"
        ) && source.contains(
            "phpc_user_function_4_relay_key(int phpc_call_depth, phpc_NativeSymbolTableHandle phpc_root_symbols"
        ),
        "wrapper frames that can reach global-import callees should receive the caller root symbol table:\n{source}"
    );
    assert!(
        body.contains("phpc_native_symbol_table_new()")
            && body.contains("phpc_user_function_3_nested_relay(")
            && body.contains("phpc_user_function_4_relay_key("),
        "top-level calls to transitive global-import wrappers should share one caller root symbol table:\n{source}"
    );
    assert!(
        source.matches("phpc_native_symbol_table_new()").count() == 1,
        "wrapper frames should pass the borrowed root table instead of creating nested symbol tables:\n{source}"
    );
    assert!(
        !source.contains("assembly user-function lowering rejects")
            && !source.contains("assembly global-declaration lowering rejects"),
        "transitive global-import frame calls should not hit frame/global blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_globals_self_import_user_function_frames() {
    let program = parse(NATIVE_GLOBALS_SELF_IMPORT_USER_FUNCTION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "phpc_user_function_0_touch_globals(int phpc_call_depth, phpc_NativeSymbolTableHandle phpc_root_symbols"
        ) && source.contains(
            "phpc_user_function_1_mix_globals(int phpc_call_depth, phpc_NativeSymbolTableHandle phpc_root_symbols"
        ),
        "$GLOBALS self-import frames should receive the shared caller root symbol table:\n{source}"
    );
    assert!(
        source.matches("phpc_native_symbol_table_reference_for_path(").count() >= 1
            && source.contains("global_import_ref_")
            && body.contains("phpc_native_symbol_table_new()"),
        "ordinary globals mixed with $GLOBALS self-imports should still bind through reference paths and one caller root table:\n{source}"
    );
    assert!(
        source.contains("phpc_native_symbol_table_set_value_by_path_with_diagnostic")
            && source.contains("phpc_native_symbol_table_read_value_by_path_with_diagnostic"),
        "$GLOBALS self-import function bodies should read and write through shared symbol paths:\n{source}"
    );
    assert!(
        !source.contains("assembly global-declaration lowering rejects")
            && !source.contains("assembly user-function lowering rejects"),
        "$GLOBALS self-imports should not hit global/frame blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_runtime_dynamic_globals_self_import_calls() {
    let program = parse(NATIVE_RUNTIME_DYNAMIC_GLOBALS_SELF_IMPORT_USER_FUNCTION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        source.contains(
            "phpc_user_function_0_touch_globals(int phpc_call_depth, phpc_NativeSymbolTableHandle phpc_root_symbols"
        ) && source.contains("phpc_native_value_dynamic_call_name_matches"),
        "runtime dynamic calls to $GLOBALS self-import frames should reuse dynamic lookup and root-symbol frame ABI:\n{source}"
    );
    assert!(
        body.contains("phpc_native_symbol_table_new()")
            && body.contains("phpc_user_function_0_touch_globals(")
            && body.contains("dynamic_user_function_matched_"),
        "runtime dynamic $GLOBALS self-import dispatch should materialize and pass one caller root table:\n{source}"
    );
    assert!(
        !source.contains("assembly dynamic function-call lowering rejects")
            && !source.contains("assembly global-declaration lowering rejects"),
        "runtime dynamic $GLOBALS self-import calls should not hit dynamic/global blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_known_string_dynamic_builtin_calls() {
    let program = parse(NATIVE_DYNAMIC_BUILTIN_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_to_string_bytes")
            && source.contains("phpc_native_value_string_result_operation_with_diagnostic")
            && source.contains("phpc_native_value_string_predicate_with_diagnostic")
            && source.contains("phpc_native_value_string_search_result_with_diagnostic")
            && source.contains("phpc_native_value_type_name_result")
            && source.contains("phpc_native_value_type_predicate"),
        "known-string dynamic builtins should reuse the existing native builtin semantic families:\n{source}"
    );
    assert!(
        !source.contains("assembly dynamic function-call lowering rejects")
            && !source.contains("runtime callable builtins"),
        "supported finite known-string dynamic builtins should not hit the dynamic-call blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_runtime_string_dynamic_builtin_calls() {
    let program = parse(NATIVE_RUNTIME_DYNAMIC_BUILTIN_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic")
            && source.contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free")
            && source.contains("phpc_native_call_arguments_new")
            && source.contains("phpc_native_call_arguments_push_value_and_free"),
        "runtime dynamic builtins should use the shared callable-value source-call ABI:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic(")
            .count()
            >= 6
            && source
                .matches("phpc_native_callable_value_invoke_value_with_diagnostic_and_free(")
                .count()
                >= 6
            && source
                .matches("phpc_native_call_arguments_push_value_and_free")
                .count()
                >= 7,
        "runtime dynamic builtins should build source-call arguments for each selected builtin family:\n{source}"
    );
    assert!(
        !source.contains("assembly dynamic function-call lowering rejects")
            && !source.contains("unsupported runtime callable builtin families"),
        "supported runtime string-valued dynamic builtins should not hit the dynamic-call blocker:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_dynamic_call_name_matches")
            && !source.contains("dynamic_user_function_matched_"),
        "runtime dynamic builtins should not fall back to the legacy generated-C name-match ladder:\n{source}"
    );
}

#[test]
fn native_executable_c_source_lowers_finite_mixed_dynamic_calls() {
    let program = parse(NATIVE_MIXED_DYNAMIC_CALL_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_value_dynamic_call_name_matches")
            && source.contains("phpc_native_value_dynamic_call_failure_with_diagnostic"),
        "finite mixed callable sets should reuse the shared runtime lookup/failure ABI:\n{source}"
    );
    assert!(
        source.contains("phpc_user_function_0_pick(")
            && source.contains("phpc_user_function_1_wrap(")
            && source.contains("phpc_native_value_string_result_operation_with_diagnostic")
            && source.contains("phpc_native_value_type_name_result"),
        "finite mixed dispatch should cover multiple user frames and builtin semantic families:\n{source}"
    );
    assert!(
        source.matches("phpc_native_value_dynamic_call_name_matches(").count() >= 6
            && source.contains("dynamic_user_function_matched_"),
        "finite mixed dispatch should be generated as a callable table, not one recognized spelling:\n{source}"
    );
    assert!(
        !source.contains("assembly dynamic function-call lowering rejects")
            && !source.contains("unsupported finite target sets"),
        "supported finite mixed callable sets should no longer hit the dynamic-call blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_invokes_callable_arrays_through_method_frames() {
    let source = concat!(
        "<?php\n",
        "class CallableArrayTarget {\n",
        "    public static function stat($value) { return \"S\" . $value; }\n",
        "    public function inst($value) { return \"I\" . $value; }\n",
        "    public function __invoke($value) { return \"V\" . $value; }\n",
        "    public function mutate(&$slot, $value) { $slot = $value; return $slot; }\n",
        "}\n",
        "class CallableArrayChild extends CallableArrayTarget {}\n",
        "function apply_callable($callback, $value) { return $callback($value); }\n",
        "function apply_ref_callable($callback, &$slot, $value) { return $callback($slot, $value); }\n",
        "$static = [\"CallableArrayTarget\", \"stat\"];\n",
        "echo $static(\"A\"), \"|\", apply_callable($static, \"B\"), \"|\";\n",
        "$childStatic = [\"CallableArrayChild\", \"stat\"];\n",
        "echo $childStatic(\"C\"), \"|\";\n",
        "$object = new CallableArrayTarget();\n",
        "$instance = [$object, \"inst\"];\n",
        "echo $instance(\"D\"), \"|\", apply_callable($instance, \"E\"), \"|\";\n",
        "$child = new CallableArrayChild();\n",
        "$childInstance = [$child, \"inst\"];\n",
        "echo $childInstance(\"F\"), \"|\";\n",
        "echo $object(\"G\"), \"|\", $child(\"H\"), \"|\";\n",
        "$slot = \"old\";\n",
        "$mutator = [$object, \"mutate\"];\n",
        "echo $mutator($slot, \"direct\"), \":\", $slot, \"|\";\n",
        "echo apply_ref_callable($mutator, $slot, \"relay\"), \":\", $slot;\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("PHPC_NATIVE_CALLABLE_KIND_METHOD")
            && source.contains("phpc_native_callable_table_register_visibility_staticness_frame_callback_and_free")
            && source.contains("phpc_native_callable_table_register_class_parent_and_free")
            && source.contains("_native_callable_frame"),
        "declared public methods should be published as runtime callable-table method descriptors with generated frame wrappers:\n{source}"
    );
    assert!(
        source.contains("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic")
            && source
                .contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free")
            && source.contains("phpc_native_call_frame_read_receiver")
            && source.contains("phpc_native_call_arguments_push_reference_and_free")
            && source.contains("phpc_native_call_arguments_push_value_and_free"),
        "callable arrays and callable objects should use the shared callable-value lookup/invoke ABI and generated method frames:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic(")
            .count()
            >= 6,
        "static callable arrays, object callable arrays, inherited method arrays, and object __invoke should share the runtime lookup boundary:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION)
            && !source.contains("callable must be a string for generated-C runtime dispatch")
            && !source.contains("phpc_native_value_dynamic_call_name_matches")
            && !source.contains("callable_array_matched")
            && !source.contains("callable_object_matched"),
        "supported method callables should not use the legacy generated-C dynamic callable ladder:\n{source}"
    );
}

#[test]
fn native_executable_c_source_invokes_class_method_strings_through_runtime_callable_value_abi() {
    let source = concat!(
        "<?php\n",
        "class ClassMethodCallableBase {\n",
        "    public static function stat($value) { return \"B\" . $value; }\n",
        "}\n",
        "class ClassMethodCallableChild extends ClassMethodCallableBase {}\n",
        "class ClassMethodCallableRelay {\n",
        "    public static function apply($callback, $value) { return $callback($value); }\n",
        "}\n",
        "function apply_class_method_callable($callback, $value) { return $callback($value); }\n",
        "$direct = \"ClassMethodCallableBase::stat\";\n",
        "echo $direct(\"A\"), \"|\", apply_class_method_callable($direct, \"B\"), \"|\";\n",
        "$inherited = \"ClassMethodCallableChild::stat\";\n",
        "echo $inherited(\"C\"), \"|\", ClassMethodCallableRelay::apply($inherited, \"D\"), \"|\";\n",
        "$upper = \"CLASSMETHODCALLABLECHILD::STAT\";\n",
        "echo $upper(\"E\");\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("PHPC_NATIVE_CALLABLE_KIND_METHOD")
            && source.contains("phpc_native_callable_table_register_visibility_staticness_frame_callback_and_free")
            && source.contains("phpc_native_callable_table_register_class_parent_and_free")
            && source.contains("_native_callable_frame"),
        "declared static methods should be published as runtime callable-table descriptors consumed by class-method string values:\n{source}"
    );
    assert!(
        source.contains("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic")
            && source
                .contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free")
            && source.contains("phpc_native_call_arguments_push_value_and_free"),
        "class-method string callables should use callable-value lookup/invoke and shared argument ABI:\n{source}"
    );
    assert!(
        source
            .matches("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic(")
            .count()
            >= 4,
        "direct, relayed, inherited, and case-varied class-method strings should share the runtime lookup boundary:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION)
            && !source.contains("callable must be a string for generated-C runtime dispatch")
            && !source.contains("phpc_native_value_dynamic_call_name_matches")
            && !source.contains("callable_array_matched")
            && !source.contains("callable_object_matched"),
        "class-method string callables should not use the legacy generated-C dynamic callable ladder:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_class_method_string_callable_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class ClassMethodCallableBase {\n",
        "    public static function stat($value) { return \"B\" . $value; }\n",
        "}\n",
        "class ClassMethodCallableChild extends ClassMethodCallableBase {}\n",
        "class ClassMethodCallableRelay {\n",
        "    public static function apply($callback, $value) { return $callback($value); }\n",
        "}\n",
        "function apply_class_method_callable($callback, $value) { return $callback($value); }\n",
        "$direct = \"ClassMethodCallableBase::stat\";\n",
        "echo $direct(\"A\"), \"|\", apply_class_method_callable($direct, \"B\"), \"|\";\n",
        "$inherited = \"ClassMethodCallableChild::stat\";\n",
        "echo $inherited(\"C\"), \"|\", ClassMethodCallableRelay::apply($inherited, \"D\"), \"|\";\n",
        "$upper = \"CLASSMETHODCALLABLECHILD::STAT\";\n",
        "echo $upper(\"E\");\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("class_method_string_callable", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native class-method string callable executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"BA|BB|BC|BD|BE");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn emit_exe_links_and_runs_callable_array_invocation_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class CallableArrayTarget {\n",
        "    public static function stat($value) { return \"S\" . $value; }\n",
        "    public function inst($value) { return \"I\" . $value; }\n",
        "    public function __invoke($value) { return \"V\" . $value; }\n",
        "    public function mutate(&$slot, $value) { $slot = $value; return $slot; }\n",
        "}\n",
        "class CallableArrayChild extends CallableArrayTarget {}\n",
        "function apply_callable($callback, $value) { return $callback($value); }\n",
        "function apply_ref_callable($callback, &$slot, $value) { return $callback($slot, $value); }\n",
        "$static = [\"CallableArrayTarget\", \"stat\"];\n",
        "echo $static(\"A\"), \"|\", apply_callable($static, \"B\"), \"|\";\n",
        "$childStatic = [\"CallableArrayChild\", \"stat\"];\n",
        "echo $childStatic(\"C\"), \"|\";\n",
        "$object = new CallableArrayTarget();\n",
        "$instance = [$object, \"inst\"];\n",
        "echo $instance(\"D\"), \"|\", apply_callable($instance, \"E\"), \"|\";\n",
        "$child = new CallableArrayChild();\n",
        "$childInstance = [$child, \"inst\"];\n",
        "echo $childInstance(\"F\"), \"|\";\n",
        "echo $object(\"G\"), \"|\", $child(\"H\"), \"|\";\n",
        "$slot = \"old\";\n",
        "$mutator = [$object, \"mutate\"];\n",
        "echo $mutator($slot, \"direct\"), \":\", $slot, \"|\";\n",
        "echo apply_ref_callable($mutator, $slot, \"relay\"), \":\", $slot;\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("callable_array_invocation", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native callable-array executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(
        run.stdout,
        b"SA|SB|SC|ID|IE|IF|VG|VH|direct:direct|relay:relay"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_invokes_callable_objects_through_invoke_frames() {
    let source = concat!(
        "<?php\n",
        "class CallableObjectTarget {\n",
        "    public function __invoke($value) { return \"O\" . $value; }\n",
        "    public function selfCall($value) { return $this($value); }\n",
        "}\n",
        "class CallableObjectChild extends CallableObjectTarget {}\n",
        "class CallableObjectRelay {\n",
        "    public static function apply($callback, $value) { return $callback($value); }\n",
        "}\n",
        "class CallableObjectMutator {\n",
        "    public function __invoke(&$slot, $value) { $slot = $value; return $slot; }\n",
        "}\n",
        "function apply_callable($callback, $value) { return $callback($value); }\n",
        "function apply_ref_callable($callback, &$slot, $value) { return $callback($slot, $value); }\n",
        "$object = new CallableObjectTarget();\n",
        "echo $object(\"A\"), \"|\", apply_callable($object, \"B\"), \"|\";\n",
        "echo CallableObjectRelay::apply($object, \"C\"), \"|\", $object->selfCall(\"D\"), \"|\";\n",
        "$child = new CallableObjectChild();\n",
        "echo $child(\"E\"), \"|\";\n",
        "$slot = \"old\";\n",
        "$mutator = new CallableObjectMutator();\n",
        "echo $mutator($slot, \"direct\"), \":\", $slot, \"|\";\n",
        "echo apply_ref_callable($mutator, $slot, \"relay\"), \":\", $slot;\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("PHPC_NATIVE_CALLABLE_KIND_METHOD")
            && source.contains("phpc_native_callable_table_register_visibility_staticness_frame_callback_and_free")
            && source.contains("phpc_native_callable_table_register_class_parent_and_free"),
        "callable object __invoke methods should be published through the runtime method callable table:\n{source}"
    );
    assert!(
        source.contains("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic")
            && source
                .contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free")
            && source.contains("phpc_native_call_frame_read_receiver")
            && source.contains("phpc_declared_method_")
            && source.contains("__invoke"),
        "callable object dispatch should use runtime callable-value lookup with generated __invoke method frame wrappers:\n{source}"
    );
    assert!(
        source.contains("phpc_user_function_0_apply_callable(")
            && source.contains("phpc_user_function_1_apply_ref_callable(")
            && source.contains("phpc_native_reference_set_value"),
        "callable objects should flow through direct calls, user-function relay, static-method relay, method self-calls, and by-reference argument relay:\n{source}"
    );
    assert!(
        !source.contains(ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION)
            && !source.contains("callable must be a string for generated-C runtime dispatch")
            && !source.contains("phpc_native_value_dynamic_call_name_matches")
            && !source.contains("callable_object_matched"),
        "supported callable objects should not use the legacy generated-C dynamic callable ladder:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_callable_object_invocation_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class CallableObjectTarget {\n",
        "    public function __invoke($value) { return \"O\" . $value; }\n",
        "    public function selfCall($value) { return $this($value); }\n",
        "}\n",
        "class CallableObjectChild extends CallableObjectTarget {}\n",
        "class CallableObjectRelay {\n",
        "    public static function apply($callback, $value) { return $callback($value); }\n",
        "}\n",
        "class CallableObjectMutator {\n",
        "    public function __invoke(&$slot, $value) { $slot = $value; return $slot; }\n",
        "}\n",
        "function apply_callable($callback, $value) { return $callback($value); }\n",
        "function apply_ref_callable($callback, &$slot, $value) { return $callback($slot, $value); }\n",
        "$object = new CallableObjectTarget();\n",
        "echo $object(\"A\"), \"|\", apply_callable($object, \"B\"), \"|\";\n",
        "echo CallableObjectRelay::apply($object, \"C\"), \"|\", $object->selfCall(\"D\"), \"|\";\n",
        "$child = new CallableObjectChild();\n",
        "echo $child(\"E\"), \"|\";\n",
        "$slot = \"old\";\n",
        "$mutator = new CallableObjectMutator();\n",
        "echo $mutator($slot, \"direct\"), \":\", $slot, \"|\";\n",
        "echo apply_ref_callable($mutator, $slot, \"relay\"), \":\", $slot;\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("callable_object_invocation", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!(
            "failed to run native callable-object executable {}: {error}",
            output_path.display()
        )
    });

    assert!(run.status.success(), "native executable failed");
    assert_eq!(run.stdout, b"OA|OB|OC|OD|OE|direct:direct|relay:relay");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
}

#[test]
fn native_executable_c_source_preserves_unsupported_dynamic_builtin_lookup_boundary() {
    for source in [
        "<?php\n$call = \"count\";\necho $call([1]);\n",
        "<?php\n$flag = isset($_GET[\"x\"]);\n$call = $flag ? \"strlen\" : \"count\";\necho $call(\"abc\");\n",
    ] {
        let program = parse(source).unwrap();
        let generated = emit_native_executable_c_source(&program).unwrap();
        let lookup = generated
            .find("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic(")
            .expect("unsupported runtime builtin should lower through callable lookup");
        let arguments = generated
            .find("phpc_native_call_arguments_new")
            .expect("dynamic callable source call should still construct arguments after lookup");

        assert!(
            lookup < arguments,
            "unsupported runtime builtin lookup must happen before argument construction:\n{generated}"
        );
        assert!(
            generated.contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free")
                && !generated.contains("phpc_native_value_dynamic_call_name_matches"),
            "unsupported runtime builtins should use callable lookup/invoke boundaries without the legacy generated-C name ladder:\n{generated}"
        );
    }
}

#[test]
fn native_executable_c_source_routes_user_function_introspection_through_registered_frames() {
    let program = parse(NATIVE_USER_FUNCTION_INTROSPECTION_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("static phpc_NativeValueHandle phpc_user_function_0_pick("),
        "registered user function should still lower to a reusable frame:\n{source}"
    );
    assert!(
        source.contains("phpc_native_text_membership_with_reference_slot_with_diagnostic(")
            && source.contains("(const uint8_t *)\"pick\""),
        "function_exists runtime membership should include registered user functions:\n{source}"
    );
    assert!(
        !source.contains("assembly function-call lowering rejects")
            && !source.contains("assembly user-function lowering rejects"),
        "function introspection over registered direct frames should not hit call blockers:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_callable_array_syntax_only_through_runtime_abi() {
    let program = parse(NATIVE_CALLABLE_ARRAY_SYNTAX_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        source.contains("phpc_native_array_is_callable_syntax_only"),
        "direct array callable syntax should route through the shared runtime array helper:\n{source}"
    );
    assert!(
        source.contains("phpc_native_value_is_callable_syntax_only"),
        "array values loaded from storage should route through the shared runtime value helper:\n{source}"
    );
    assert!(
        !source.contains("assembly function-call lowering rejects"),
        "syntax-only callable arrays should not hit callable invocation blockers:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_callable_array_syntax_only_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "callable_array_syntax_only",
        NATIVE_CALLABLE_ARRAY_SYNTAX_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native callable-array syntax executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"T|T|T|FFF");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_direct_user_function_frame_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "direct_user_function_frame",
        NATIVE_USER_FUNCTION_FRAME_SOURCE,
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native user-function executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"GO|alt|d|relay|side:effect|done");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_exact_imported_user_function_alias_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "exact_imported_user_function_alias",
        NATIVE_IMPORTED_USER_FUNCTION_ALIAS_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native imported user-function executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"label:A|new:new|label:direct");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_exact_imported_runtime_builtin_alias_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "exact_imported_runtime_builtin_alias",
        NATIVE_IMPORTED_RUNTIME_BUILTIN_ALIAS_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native imported runtime-builtin executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"3|1|MIX|array");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_reports_unsupported_imported_builtin_before_arguments() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "exact_imported_unsupported_builtin_alias",
        NATIVE_IMPORTED_UNSUPPORTED_BUILTIN_ALIAS_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native unsupported imported-builtin executable: {error}")
    });

    assert!(
        !run.status.success(),
        "unsupported imported builtin should fail"
    );
    assert!(
        run.stdout.is_empty(),
        "unsupported imported builtin should stop before argument side effects, stdout:\n{}",
        String::from_utf8_lossy(&run.stdout)
    );
    assert!(
        String::from_utf8_lossy(&run.stderr).contains(
            "unsupported call count(): runtime dynamic generated-C lookup did not find a registered user-function frame or supported native builtin family"
        ),
        "stderr should contain imported count() runtime builtin failure, got:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_typed_user_function_frame_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "typed_user_function_frame",
        NATIVE_TYPED_USER_FUNCTION_FRAME_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native typed user-function executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"7:12|[]:5|fallback:given|5|ok");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_variadic_user_function_frame_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "variadic_user_function_frame",
        NATIVE_VARIADIC_USER_FUNCTION_FRAME_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native variadic user-function executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"empty|filled|A|B|5|L|dyn");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_named_user_function_argument_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "named_user_function_arguments",
        NATIVE_NAMED_USER_FUNCTION_ARGUMENT_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native named user-function executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"TEFFS!TE|S!");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_named_method_source_call_argument_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "named_method_source_call_arguments",
        NATIVE_NAMED_METHOD_SOURCE_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native named method source-call executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"TEFFS?TE|S?");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_named_dynamic_method_source_call_argument_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "named_dynamic_method_source_call_arguments",
        NATIVE_NAMED_DYNAMIC_METHOD_SOURCE_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native named dynamic method source-call executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"TEFFS~TE|S~");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_by_reference_user_function_frame_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "by_reference_user_function_frame",
        NATIVE_BY_REFERENCE_USER_FUNCTION_FRAME_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native by-reference user-function executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        run.stdout,
        b"dynamic:dynamic|changed:changed|new|deep-new|right:right:left"
    );
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_global_import_user_function_frame_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "global_import_user_function_frame",
        NATIVE_GLOBAL_IMPORT_USER_FUNCTION_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native global-import user-function executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        run.stdout,
        b"seen:root|direct:direct|seen:direct|again:again|G:G|SO:S:O"
    );
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_transitive_global_import_user_function_frame_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "transitive_global_import_user_function_frame",
        NATIVE_TRANSITIVE_GLOBAL_IMPORT_USER_FUNCTION_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native transitive global-import executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"wrapped:wrapped|via:via");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_globals_self_import_user_function_frame_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "globals_self_import_user_function_frame",
        NATIVE_GLOBALS_SELF_IMPORT_USER_FUNCTION_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native $GLOBALS self-import executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        run.stdout,
        b"root|changed:DEEP|changed:DEEP|mixed!:mixed!|mixed!"
    );
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_global_frame_direct_user_function_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "request_global_frame_direct_user_function",
        NATIVE_REQUEST_GLOBAL_FRAME_DIRECT_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native request/global direct executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"Adaroot");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_global_frame_mutation_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "request_global_frame_mutation",
        NATIVE_REQUEST_GLOBAL_FRAME_MUTATION_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native request/global mutation executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"B");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_global_frame_globals_alias_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "request_global_frame_globals_alias",
        NATIVE_REQUEST_GLOBAL_FRAME_GLOBALS_ALIAS_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native request/global $GLOBALS alias executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"fn");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_request_global_frame_mixed_environment_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "request_global_frame_mixed_environment",
        NATIVE_REQUEST_GLOBAL_FRAME_MIXED_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native request/global mixed environment executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"fn:G");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_runtime_dynamic_by_reference_user_function_frame_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "runtime_dynamic_by_reference_user_function_frame",
        NATIVE_RUNTIME_DYNAMIC_BY_REFERENCE_USER_FUNCTION_FRAME_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native runtime dynamic by-reference executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        run.stdout,
        b"dynamic:dynamic|array|nested|right:right:left|wrap:ok"
    );
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_reports_typed_user_function_frame_mismatches() {
    if !has_cc() {
        return;
    }

    for (name, source, expected) in [
        (
            "typed_user_function_parameter_mismatch",
            "<?php\nfunction needs_int(int $value): string { return $value; }\necho needs_int([]), \"after\";\n",
            "unsupported call needs_int(): parameter $value expects int, got array",
        ),
        (
            "typed_user_function_return_mismatch",
            "<?php\nfunction returns_int(): int { return \"not numeric\"; }\necho returns_int(), \"after\";\n",
            "unsupported call returns_int(): return value expects int, got string",
        ),
        (
            "typed_user_function_variadic_mismatch",
            "<?php\nfunction spread_int(int ...$values): int { return $values[0]; }\necho spread_int([]), \"after\";\n",
            "unsupported call spread_int(): parameter $values expects int, got array",
        ),
    ] {
        let (source_path, output_path) = compile_native_link_fixture(name, source);
        let run = Command::new(&output_path).output().unwrap_or_else(|error| {
            panic!("failed to run native typed user-function mismatch executable: {error}")
        });

        assert!(
            !run.status.success(),
            "{name} should fail through the native diagnostic path"
        );
        assert!(
            run.stdout.is_empty(),
            "{name} should stop before later side effects, stdout:\n{}",
            String::from_utf8_lossy(&run.stdout)
        );
        assert!(
            String::from_utf8_lossy(&run.stderr).contains(expected),
            "{name} stderr should contain {expected:?}, got:\n{}",
            String::from_utf8_lossy(&run.stderr)
        );

        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(&source_path);
    }
}

#[test]
fn emit_exe_links_and_runs_dynamic_user_function_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "dynamic_user_function_call",
        NATIVE_DYNAMIC_USER_FUNCTION_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native dynamic user-function executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"GO|OK|MIX|TAIL");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_runtime_dynamic_user_function_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "runtime_dynamic_user_function_call",
        NATIVE_RUNTIME_DYNAMIC_USER_FUNCTION_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native runtime dynamic user-function executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"GO|mix|D!");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_dynamic_string_callable_value_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "dynamic_string_callable_value",
        NATIVE_DYNAMIC_STRING_CALLABLE_VALUE_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native dynamic string callable-value executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"GO|mix|OK|BI|caps");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_runtime_dynamic_global_import_user_function_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "runtime_dynamic_global_import_user_function_call",
        NATIVE_RUNTIME_DYNAMIC_GLOBAL_IMPORT_USER_FUNCTION_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native runtime dynamic global-import executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        run.stdout,
        b"seen:root|dynamic:dynamic|seen:dynamic|array:array|seen:dynamic|finite:finite|wrap:finite:finite|mix:finite|MIX"
    );
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_runtime_dynamic_globals_self_import_user_function_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "runtime_dynamic_globals_self_import_user_function_call",
        NATIVE_RUNTIME_DYNAMIC_GLOBALS_SELF_IMPORT_USER_FUNCTION_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native runtime dynamic $GLOBALS self-import executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"dynamic|dynamic");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_known_string_dynamic_builtin_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "known_string_dynamic_builtin_call",
        NATIVE_DYNAMIC_BUILTIN_CALL_SOURCE,
    );

    let run = Command::new(&output_path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run native dynamic builtin executable: {error}"));

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"3|GO|1|2|array|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_runtime_dynamic_builtin_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "runtime_dynamic_builtin_call",
        NATIVE_RUNTIME_DYNAMIC_BUILTIN_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native runtime dynamic builtin executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"3|GO|1|7|array|1");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_finite_mixed_dynamic_call_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "finite_mixed_dynamic_call",
        NATIVE_MIXED_DYNAMIC_CALL_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native finite mixed dynamic executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"user:Go|wrap:Go|user:yo|YO|3|string");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_reports_runtime_dynamic_user_function_call_failures() {
    if !has_cc() {
        return;
    }

    for (name, source, expected) in [
        (
            "runtime_dynamic_user_function_unsupported_builtin_miss",
            "<?php\nfunction invoke($call, $value) { return $call($value); }\nfunction pick($value) { return $value; }\necho invoke(\"count\", [1]), \"after\";\n",
            "unsupported call count(): runtime dynamic generated-C lookup did not find a registered user-function frame or supported native builtin family",
        ),
        (
            "runtime_dynamic_user_function_arity_mismatch",
            "<?php\nfunction needs_two($a, $b) { return $a . $b; }\nfunction invoke($call, $value) { return $call($value); }\necho invoke(\"needs_two\", \"a\"), \"after\";\n",
            "unsupported call needs_two(): argument count is outside the generated frame arity/default/variadic subset",
        ),
        (
            "runtime_dynamic_user_function_non_string",
            "<?php\nfunction invoke($call, $value) { return $call($value); }\nfunction pick($value) { return $value; }\necho invoke(7, \"abc\"), \"after\";\n",
            "unsupported call dynamic function call: callable must be a string for generated-C runtime dispatch, got int",
        ),
        (
            "runtime_dynamic_user_function_array_callable_blocked",
            "<?php\nfunction invoke($call, $value) { return $call($value); }\nfunction pick($value) { return $value; }\necho invoke([\"Box\", \"run\"], \"abc\"), \"after\";\n",
            "unsupported call dynamic function call: callable must be a string for generated-C runtime dispatch, got array",
        ),
        (
            "runtime_dynamic_user_function_by_reference_arity_mismatch",
            "<?php\nfunction set_to(&$slot, $value) { $slot = $value; }\n$slot = \"old\";\n$call = isset($_GET[\"call\"]) ? $_GET[\"call\"] : \"set_to\";\necho $call($slot), \"after\";\n",
            "unsupported call set_to(): argument count is outside the generated frame arity/default/variadic subset",
        ),
        (
            "runtime_dynamic_user_function_by_reference_literal_argument",
            "<?php\nfunction set_to(&$slot, $value) { $slot = $value; }\nfunction later() { echo \"arg\"; return \"x\"; }\n$call = isset($_GET[\"call\"]) ? $_GET[\"call\"] : \"set_to\";\necho $call(\"literal\", later()), \"after\";\n",
            "unsupported call set_to(): by-reference parameter binding requires a supported lvalue argument in the generated-C runtime dynamic frame subset",
        ),
    ] {
        let (source_path, output_path) = compile_native_link_fixture(name, source);
        let run = Command::new(&output_path).output().unwrap_or_else(|error| {
            panic!("failed to run native runtime dynamic user-function failure executable: {error}")
        });

        assert!(!run.status.success(), "{name} should fail at runtime");
        assert!(
            run.stdout.is_empty(),
            "{name} should stop before later side effects, stdout:\n{}",
            String::from_utf8_lossy(&run.stdout)
        );
        assert!(
            String::from_utf8_lossy(&run.stderr).contains(expected),
            "{name} stderr should contain {expected:?}, got:\n{}",
            String::from_utf8_lossy(&run.stderr)
        );

        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(&source_path);
    }
}

#[test]
fn emit_exe_reports_runtime_dynamic_builtin_call_failures() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "function count_arg() { echo \"arg\"; return [1]; }\n",
        "$call = isset($_GET[\"call\"]) ? $_GET[\"call\"] : \"count\";\n",
        "echo $call(count_arg()), \"after\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("runtime_dynamic_builtin_unsupported", source);
    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native runtime dynamic builtin failure executable: {error}")
    });

    assert!(
        !run.status.success(),
        "unsupported runtime builtin should fail"
    );
    assert!(
        run.stdout.is_empty(),
        "unsupported runtime builtin should stop before later side effects, stdout:\n{}",
        String::from_utf8_lossy(&run.stdout)
    );
    assert!(
        String::from_utf8_lossy(&run.stderr).contains(
            "unsupported call count(): runtime dynamic generated-C lookup did not find a registered user-function frame or supported native builtin family"
        ),
        "stderr should contain runtime dynamic builtin failure, got:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_recursive_user_function_frame_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "recursive_user_function_frame",
        NATIVE_RECURSIVE_USER_FUNCTION_FRAME_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native recursive user-function executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"3:2:1:done|even:odd|dyn");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_user_function_introspection_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "direct_user_function_introspection",
        NATIVE_USER_FUNCTION_INTROSPECTION_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native user-function introspection executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"1|1|1|1||ok");
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn native_executable_c_source_rejects_unsupported_user_function_frame_shapes() {
    for source in [
        "<?php\nfunction byref_variadic(&...$values) { return 1; }\n",
        "<?php\nfunction typed_byref(int &$value) { return $value; }\n",
        "<?php\nfunction default_byref(&$value = null) { return $value; }\n",
        "<?php\nfunction typed(callable $value) { return 1; }\n",
        "<?php\nfunction typed_object(object $value) { return 1; }\n",
        "<?php\nfunction ret(): void { return; }\n",
        "<?php\nfunction bad_exit() { exit(\"bad\"); }\necho bad_exit();\n",
        "<?php\nfunction outer() { function inner() { return 1; } return inner(); }\necho outer();\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error
                .message
                .contains("bounded generated-C frame subset"),
            "{source}\n{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_rejects_unsupported_global_import_roots() {
    for source in ["<?php\nfunction bad() { global $_GET; return 1; }\necho bad();\n"] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error
                .message
                .contains("assembly global-declaration lowering rejects"),
            "{source}\n{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_rejects_unsupported_by_reference_call_bindings() {
    for source in [
        "<?php\nfunction set_to(&$slot, $value) { $slot = $value; }\nset_to(\"literal\", \"x\");\n",
        "<?php\nfunction set_to(&$slot, $value) { $slot = $value; }\nset_to(strrev(\"x\"), \"y\");\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error.message.contains("by-reference argument binding")
                || error.message.contains("dynamic string-valued calls")
                || error
                    .message
                    .contains("assembly dynamic function-call lowering rejects"),
            "{source}\n{error:?}"
        );
    }
}

#[test]
fn native_executable_c_source_lowers_runtime_dynamic_by_reference_user_function_dispatch() {
    let program = parse(NATIVE_RUNTIME_DYNAMIC_BY_REFERENCE_USER_FUNCTION_FRAME_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("phpc_native_value_dynamic_call_name_matches")
            && body.contains("phpc_user_function_0_set_to(")
            && body.contains("phpc_user_function_1_swap(")
            && body.contains("phpc_user_function_2_wrap("),
        "runtime string-valued dispatch should route by-reference and by-value frames through the same lookup table:\n{source}"
    );
    assert!(
        body.matches("phpc_native_symbol_table_reference_for_path(").count() >= 4
            && body.contains("phpc_NativeReferenceHandle"),
        "runtime dynamic by-reference branches should bind direct and nested lvalue arguments through the shared symbol/reference path:\n{source}"
    );
    assert!(
        !body.contains("by-reference parameter binding requires a statically known reference argument set"),
        "runtime string-valued by-reference frames should no longer stop on the old by-reference dynamic-call failure:\n{source}"
    );
}

#[test]
fn native_executable_c_source_plans_scoped_callable_string_signature_arguments() {
    let program = parse(NATIVE_SCOPED_CALLABLE_STRING_SIGNATURE_SOURCE).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches("phpc_native_call_arguments_push_reference_and_free")
            .count()
            >= 4
            && body.contains("phpc_native_callable_lookup_value_or_closure_with_context_diagnostic")
            && body.contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free")
            && body.contains("phpc_native_callable_value_invoke_reference_with_diagnostic_and_free"),
        "variable-held, concatenated, branch-selected, and reference-return scoped callable strings should share callable-value lookup and reference carriers:\n{source}"
    );
    assert!(
        !body.contains("phpc_native_value_dynamic_call_name_matches"),
        "scoped callable-string signatures should not use the legacy finite function-string ladder:\n{source}"
    );
}

#[test]
fn native_executable_c_source_transfers_reference_return_source_calls_into_byref_arguments() {
    let source = concat!(
        "<?php\n",
        "class SourceCallAliasTransfer {\n",
        "    public function &borrowInstance(&$slot) { return $slot; }\n",
        "    public static function &borrow(&$slot) { return $slot; }\n",
        "    public static function &borrowOther(&$slot) { return $slot; }\n",
        "}\n",
        "function source_call_consume(&$slot, $value) { $slot = $value; return $slot; }\n",
        "function source_call_pair($label, &$left, &$right) { $left = $label . \"-left\"; $right = $label . \"-right\"; return $left . \":\" . $right; }\n",
        "$slot = \"old\";\n",
        "$box = new SourceCallAliasTransfer();\n",
        "echo source_call_consume($box->borrowInstance($slot), \"method\"), \":\", $slot, \"|\";\n",
        "echo source_call_consume(SourceCallAliasTransfer::borrow($slot), \"static\"), \":\", $slot, \"|\";\n",
        "$borrow = \"SourceCallAliasTransfer::borrow\";\n",
        "$other = \"SourceCallAliasTransfer::borrowOther\";\n",
        "echo source_call_consume($borrow($slot), \"direct\"), \":\", $slot, \"|\";\n",
        "$consume = \"source_call_consume\";\n",
        "echo $consume($borrow($slot), \"dynamic\"), \":\", $slot;\n",
        "$left = \"L0\";\n",
        "$right = \"R0\";\n",
        "echo \"|\", source_call_pair(\"pair\", $borrow($left), $other($right)), \":\", $left, \":\", $right;\n",
        "$pair = \"source_call_pair\";\n",
        "echo \"|\", $pair(\"dynpair\", $borrow($left), $other($right)), \":\", $left, \":\", $right;\n",
    );
    let program = parse(source).unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.contains("source_reference_result")
            && body.contains("source_reference_receiver_result")
            && body.contains("source_reference_static_result")
            && body.contains("phpc_native_method_invoke_reference_with_access_context_diagnostic_and_free_receiver_method_arguments")
            && body.contains("phpc_native_static_method_invoke_reference_with_access_context_diagnostic_and_free_scope_method_arguments")
            && body.contains("phpc_native_callable_value_invoke_reference_with_diagnostic_and_free")
            && body.contains("phpc_native_callable_lookup_invoke_value_with_diagnostic_and_free_arguments")
            && body.contains("phpc_native_callable_value_invoke_value_with_diagnostic_and_free"),
        "source-call reference results should use reference carriers across receiver-method, static-method, and dynamic consumers:\n{source}"
    );
    assert!(
        body.matches("phpc_native_call_arguments_push_reference_and_free")
            .count()
            >= 12,
        "source-call aliases and their direct/dynamic consumers should move multiple reference positions through the call-argument handle ABI:\n{source}"
    );
    assert!(
        !body.contains("alias_transfer_missing")
            && !body.contains(
                "by-reference parameter alias transfer from produced arguments"
            ),
        "reference-return source calls should not fall back to the produced-argument alias-transfer blocker:\n{source}"
    );
}

#[test]
fn native_executable_c_source_keeps_non_reference_scoped_callable_strings_on_return_ownership_blocker(
) {
    let program = parse(concat!(
        "<?php\n",
        "class ScopedCallableValueReturn {\n",
        "    public static function value($slot) { return $slot; }\n",
        "}\n",
        "$slot = \"value\";\n",
        "$call = \"ScopedCallableValueReturn::value\";\n",
        "$alias =& $call($slot);\n",
    ))
    .unwrap();
    let error = emit_native_executable_c_source(&program).unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, ASSEMBLY_DYNAMIC_FUNCTION_CALL_REJECTION);
}

#[test]
fn native_executable_c_source_keeps_by_value_produced_calls_on_alias_transfer_blocker() {
    let program = parse(concat!(
        "<?php\n",
        "class SourceCallByValueProducedReject {\n",
        "    public function value($slot) { return $slot; }\n",
        "    public static function stat($slot) { return $slot; }\n",
        "}\n",
        "function source_call_value_produced($slot) { return $slot; }\n",
        "function source_call_consume_reject(&$slot) { $slot = \"changed\"; }\n",
        "$slot = \"value\";\n",
        "$box = new SourceCallByValueProducedReject();\n",
        "source_call_consume_reject(source_call_value_produced($slot));\n",
        "source_call_consume_reject($box->value($slot));\n",
        "source_call_consume_reject(SourceCallByValueProducedReject::stat($slot));\n",
    ))
    .unwrap();
    let source = emit_native_executable_c_source(&program).unwrap();
    let body = main_body(&source);

    assert!(
        body.matches(
            "phpc_native_call_frame_reference_parameter_alias_transfer_result_from_results_with_diagnostic",
        )
        .count()
            >= 3,
        "by-value produced calls should stay on the alias-transfer blocker path:\n{source}"
    );
    assert!(
        !body.contains("source_reference_direct_result")
            && !body.contains("user_function_reference_result")
            && !body.contains("source_reference_receiver_result")
            && !body.contains("source_reference_static_result")
            && !body.contains("phpc_native_callable_lookup_invoke_reference_with_diagnostic_and_free_arguments")
            && !body.contains("phpc_native_method_invoke_reference_with_access_context_diagnostic_and_free_receiver_method_arguments")
            && !body.contains("phpc_native_static_method_invoke_reference_with_access_context_diagnostic_and_free_scope_method_arguments"),
        "by-value produced calls must not be coerced into source-call reference carriers:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_source_call_reference_alias_byref_argument_program() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class SourceCallAliasTransferRun {\n",
        "    public function &borrowInstance(&$slot) { return $slot; }\n",
        "    public static function &borrow(&$slot) { return $slot; }\n",
        "    public static function &borrowOther(&$slot) { return $slot; }\n",
        "}\n",
        "function source_call_consume_run(&$slot, $value) { $slot = $value; return $slot; }\n",
        "function source_call_pair_run($label, &$left, &$right) { $left = $label . \"-left\"; $right = $label . \"-right\"; return $left . \":\" . $right; }\n",
        "$slot = \"old\";\n",
        "$box = new SourceCallAliasTransferRun();\n",
        "echo source_call_consume_run($box->borrowInstance($slot), \"method\"), \":\", $slot, \"|\";\n",
        "echo source_call_consume_run(SourceCallAliasTransferRun::borrow($slot), \"static\"), \":\", $slot, \"|\";\n",
        "$borrow = \"SourceCallAliasTransferRun::borrow\";\n",
        "$other = \"SourceCallAliasTransferRun::borrowOther\";\n",
        "echo source_call_consume_run($borrow($slot), \"direct\"), \":\", $slot, \"|\";\n",
        "$consume = \"source_call_consume_run\";\n",
        "echo $consume($borrow($slot), \"dynamic\"), \":\", $slot;\n",
        "$left = \"L0\";\n",
        "$right = \"R0\";\n",
        "echo \"|\", source_call_pair_run(\"pair\", $borrow($left), $other($right)), \":\", $left, \":\", $right;\n",
        "$pair = \"source_call_pair_run\";\n",
        "echo \"|\", $pair(\"dynpair\", $borrow($left), $other($right)), \":\", $left, \":\", $right;\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("source_call_reference_alias_byref_argument", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native source-call reference alias executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        run.stdout,
        b"method:method|static:static|direct:direct|dynamic:dynamic|pair-left:pair-right:pair-left:pair-right|dynpair-left:dynpair-right:dynpair-left:dynpair-right"
    );
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_reports_source_call_reference_alias_argument_cleanup_failure() {
    if !has_cc() {
        return;
    }

    let source = concat!(
        "<?php\n",
        "class SourceCallAliasTransferFailure {\n",
        "    public static function &borrow(&$slot) { return $slot; }\n",
        "}\n",
        "function source_call_consume_failure(&$slot, $value) { $slot = $value; return $slot; }\n",
        "$slot = \"old\";\n",
        "$consume = \"source_call_consume_failure\";\n",
        "$missing = \"missing_source_call_alias_transfer\";\n",
        "$consume(SourceCallAliasTransferFailure::borrow($slot), $missing());\n",
        "echo \"after\";\n",
    );
    let (source_path, output_path) =
        compile_native_link_fixture("source_call_reference_alias_cleanup_failure", source);

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native source-call reference alias failure executable: {error}")
    });

    assert!(
        !run.status.success(),
        "runtime lookup failure should stop execution"
    );
    assert_eq!(run.stdout, b"");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("missing_source_call_alias_transfer") && stderr.contains("not registered"),
        "stderr should report the failing second argument lookup, got:\n{stderr}"
    );

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn emit_exe_links_and_runs_scoped_callable_string_signature_program() {
    if !has_cc() {
        return;
    }

    let (source_path, output_path) = compile_native_link_fixture(
        "scoped_callable_string_signature",
        NATIVE_SCOPED_CALLABLE_STRING_SIGNATURE_SOURCE,
    );

    let run = Command::new(&output_path).output().unwrap_or_else(|error| {
        panic!("failed to run native scoped callable-string executable: {error}")
    });

    assert!(
        run.status.success(),
        "run stdout:\n{}\nrun stderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        run.stdout,
        b"variable:variable|concat:concat|branch:branch|reference"
    );
    assert_eq!(run.stderr, b"");

    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_file(&source_path);
}

#[test]
fn native_executable_c_source_rejects_unsupported_dynamic_calls_inside_user_function_frames() {
    for source in [
        "<?php\nfunction bad() { $call = \"count\"; return $call([1]); }\necho bad();\n",
        "<?php\nfunction bad() { $call = \"missing\"; return $call(); }\nfunction known() { return 1; }\necho bad();\n",
    ] {
        let program = parse(source).unwrap();
        let error = emit_native_executable_c_source(&program).unwrap_err();

        assert!(
            error
                .message
                .contains("supported native builtin families"),
            "{source}\n{error:?}"
        );
    }
}

fn native_link_output_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("phpc-native-link-{name}-{}", std::process::id()));
    path
}

fn compile_native_link_fixture(name: &str, source: &str) -> (PathBuf, PathBuf) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler crate has workspace parent");
    let source_path = native_link_output_path(name).with_extension("php");
    let output_path = native_link_output_path(name);
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);
    fs::write(&source_path, source).expect("write native link fixture source");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            source_path
                .to_str()
                .expect("native link fixture source path is valid UTF-8"),
            "--emit-exe",
            output_path
                .to_str()
                .expect("native executable path is valid UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile native executable: {error}"));

    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(output_path.exists(), "native executable was not written");

    (source_path, output_path)
}

fn has_cc() -> bool {
    Command::new("cc").arg("--version").output().is_ok()
}

fn main_body(source: &str) -> &str {
    source
        .split_once("int main(void)")
        .map(|(_, body)| body)
        .unwrap_or(source)
}

fn assert_reference_handle_typedef_precedes_uses(label: &str, source: &str) {
    let typedef = "typedef struct { void *ptr; } phpc_NativeReferenceHandle;";
    let typedef_offset = source
        .find(typedef)
        .unwrap_or_else(|| panic!("{label} should declare phpc_NativeReferenceHandle:\n{source}"));
    let mut offset = 0;
    for line in source.lines() {
        if line.contains("phpc_NativeReferenceHandle") && !line.contains(typedef) {
            assert!(
                typedef_offset < offset,
                "{label} should declare phpc_NativeReferenceHandle before use:\n{source}"
            );
        }
        offset += line.len() + 1;
    }
}

fn assert_request_key_results_use_accessors(source: &str) {
    assert!(
        source.contains("phpc_native_request_state_key_result_buffer"),
        "{source}"
    );
    assert!(
        source.contains("phpc_native_request_state_key_result_status"),
        "{source}"
    );

    for line in source.lines() {
        assert!(
            !(line_has_request_key_result_field(line, "request_superglobal_key_", ".buffer")
                || line_has_request_key_result_field(line, "request_superglobal_key_", ".status")
                || line_has_request_key_result_field(
                    line,
                    "globals_dynamic_request_key_",
                    ".buffer"
                )
                || line_has_request_key_result_field(
                    line,
                    "globals_dynamic_request_key_",
                    ".status"
                )
                || line_has_request_key_result_field(
                    line,
                    "globals_dynamic_reference_path_key_",
                    ".buffer"
                )
                || line_has_request_key_result_field(
                    line,
                    "globals_dynamic_reference_path_key_",
                    ".status"
                )),
            "generated request-key code must consume key-result data through runtime accessors, not struct fields:\n{line}\n\n{source}"
        );
    }
}

fn line_has_request_key_result_field(line: &str, stem: &str, field: &str) -> bool {
    let mut remaining = line;
    while let Some(position) = remaining.find(stem) {
        let after_stem = &remaining[position + stem.len()..];
        let digit_count = after_stem
            .chars()
            .take_while(|value| value.is_ascii_digit())
            .count();
        if digit_count > 0 && after_stem[digit_count..].starts_with(field) {
            return true;
        }
        remaining = &after_stem[digit_count..];
    }
    false
}

fn assert_no_diagnostic_report_double_free(source: &str) {
    for line in source.lines() {
        assert!(
            !(line.contains("phpc_native_diagnostic_report(")
                && line.contains("phpc_native_diagnostic_free(")),
            "generated C must not free diagnostics after the report consumer already owns them:\n{line}\n\n{source}"
        );
    }
}

fn strip_fixture_editor_newline(mut value: String) -> String {
    if value.ends_with('\n') {
        value.pop();
        if value.ends_with('\r') {
            value.pop();
        }
    }
    value
}
