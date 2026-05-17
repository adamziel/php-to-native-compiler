<?php
class WP_RefCow_Static_Reference_Touch {
    public static function &touch(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class WP_RefCow_Static_Reference_Parent {
    public static function &touch(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class WP_RefCow_Static_Reference_Child extends WP_RefCow_Static_Reference_Parent {
    public function run(&$value) {
        self::touch($value, "self");
        parent::touch($value, "parent");
        static::touch($value, "static");
    }

    public static function &touch(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

$magicStorage = ["slot" => "magic"];

class WP_RefCow_Static_Reference_Magic_Box {
    public function &__get($name) {
        global $magicStorage;
        return $magicStorage;
    }
}

$items = [
    "named" => "named",
    "class_string" => "class",
    "object" => "object",
    "context" => "context",
];

WP_RefCow_Static_Reference_Touch::touch($items["named"], "direct");

$class = "WP_RefCow_Static_Reference_Touch";
$class::touch($items["class_string"], "dynamic");

$magicBox = new WP_RefCow_Static_Reference_Magic_Box();
WP_RefCow_Static_Reference_Touch::touch($magicBox->missing["slot"], "magic");
$class::touch($magicBox->missing["slot"], "dynamic_magic");

$object = new WP_RefCow_Static_Reference_Touch();
$object::touch($items["object"], "object");

$child = new WP_RefCow_Static_Reference_Child();
$child->run($items["context"]);

echo $items["named"], "\n";
echo $items["class_string"], "\n";
echo $items["object"], "\n";
echo $items["context"], "\n";
echo $magicStorage["slot"];
