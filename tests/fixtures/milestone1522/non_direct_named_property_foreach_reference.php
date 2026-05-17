<?php
class WP_RefCow_NonDirect_Named_Property_Bag {
    public $items = ["outer" => ["a" => "one", "b" => "two"]];
}

class WP_RefCow_NonDirect_Named_Private_Bag {
    private $items = ["x" => "ex", "y" => "why"];

    private function holder() {
        return $this;
    }

    public function mutate() {
        foreach ($this->holder()->items as $key => &$value) {
            $value = "private:" . $key;
            if ($key === "x") {
                $this->items["z"] = "zed";
            }
        }
        echo $this->items["x"], "|", $this->items["y"], "|", $this->items["z"], "|", $value, "\n";
        $this->items["z"] = "direct-private";
        echo $value, "|";
        $value = "tail-private";
        echo $this->items["z"], "|", $value;
    }
}

$holders = ["bag" => new WP_RefCow_NonDirect_Named_Property_Bag()];
foreach ($holders["bag"]->items["outer"] as $key => &$value) {
    $value = "public:" . $key;
}
echo $holders["bag"]->items["outer"]["a"], "|", $holders["bag"]->items["outer"]["b"], "|", $value, "\n";
$bag = $holders["bag"];
$bag->items["outer"]["b"] = "direct-public";
echo $value, "|";
$value = "tail-public";
echo $holders["bag"]->items["outer"]["b"], "|", $value, "\n";
unset($value);

$private = new WP_RefCow_NonDirect_Named_Private_Bag();
$private->mutate();
