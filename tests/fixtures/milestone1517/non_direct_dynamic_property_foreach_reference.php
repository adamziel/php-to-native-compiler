<?php
class WP_RefCow_NonDirect_Dynamic_Property_Bag {
    public $items = ["outer" => ["a" => "one", "b" => "two"]];
}

class WP_RefCow_NonDirect_Private_Bag {
    private $privateItems = ["x" => "ex", "y" => "why"];

    private function holder() {
        return $this;
    }

    public function mutate($property) {
        foreach ($this->holder()->{$property} as $key => &$value) {
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

$holders = ["bag" => new WP_RefCow_NonDirect_Dynamic_Property_Bag()];
$property = "items";
foreach ($holders["bag"]->{$property}["outer"] as $key => &$value) {
    $value = "public:" . $key;
}
echo $holders["bag"]->{$property}["outer"]["a"], "|", $holders["bag"]->{$property}["outer"]["b"], "|", $value, "\n";
$bag = $holders["bag"];
$bag->items["outer"]["b"] = "direct-public";
echo $value, "|";
$value = "tail-public";
echo $holders["bag"]->{$property}["outer"]["b"], "|", $value, "\n";
unset($value);

$private = new WP_RefCow_NonDirect_Private_Bag();
$private->mutate("privateItems");
