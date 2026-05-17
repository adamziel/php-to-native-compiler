<?php
class WP_RefCow_Dynamic_Property_Bag {
    public $items = ["outer" => ["a" => "one", "b" => "two"]];
    private $privateItems = ["x" => "ex", "y" => "why"];

    public function mutatePrivate($property) {
        foreach ($this->{$property} as $key => &$value) {
            $value = "private:" . $key;
            if ($key === "x") {
                $this->privateItems["z"] = "zed";
            }
        }
        echo $this->{$property}["x"], "|", $this->{$property}["y"], "|", $this->{$property}["z"], "|", $value, "\n";
        $this->privateItems["z"] = "direct-private";
        echo $value, "|";
        $value = "tail-private";
        echo $this->{$property}["z"], "|", $value;
    }
}

$bag = new WP_RefCow_Dynamic_Property_Bag();
$property = "items";
foreach ($bag->{$property}["outer"] as $key => &$value) {
    $value = "public:" . $key;
    if ($key === "a") {
        $bag->items["outer"]["c"] = "three";
    }
}
echo $bag->{$property}["outer"]["a"], "|", $bag->{$property}["outer"]["b"], "|", $bag->{$property}["outer"]["c"], "|", $value, "\n";
$bag->items["outer"]["c"] = "direct-public";
echo $value, "|";
$value = "tail-public";
echo $bag->{$property}["outer"]["c"], "|", $value, "\n";
unset($value);

$bag->mutatePrivate("privateItems");
