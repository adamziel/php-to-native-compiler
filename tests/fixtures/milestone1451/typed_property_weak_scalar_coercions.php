<?php
class Box {
    public int $id;
    public float $ratio;
    public bool $enabled;
    public string $label;
    public static int $count;
    public static float $total;
    public static bool $flag;
    public static string $code;
}

function yn($value) {
    return $value ? "1" : "0";
}

$box = new Box();
$box->id = "42";
$box->ratio = "4.5";
$box->enabled = "0";
$box->label = 123;

Box::$count = true;
Box::$total = "8.25";
Box::$flag = "plugin";
Box::$code = false;

echo "instance|", gettype($box->id), ":", $box->id, "|", gettype($box->ratio), ":", $box->ratio, "|", gettype($box->enabled), ":", yn($box->enabled), "|", gettype($box->label), ":", $box->label, "\n";
echo "static|", gettype(Box::$count), ":", Box::$count, "|", gettype(Box::$total), ":", Box::$total, "|", gettype(Box::$flag), ":", yn(Box::$flag), "|", gettype(Box::$code), ":", (Box::$code === "" ? "empty" : Box::$code);
