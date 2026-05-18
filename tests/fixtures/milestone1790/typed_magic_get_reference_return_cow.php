<?php
class Milestone1790_MagicBox {
    public $store = [];
    public $log = [];

    public function &__get(string $name): mixed {
        $this->log[] = "typed:" . $name;
        if (!isset($this->store[$name])) {
            $this->store[$name] = [];
        }
        return $this->store[$name];
    }
}

$source = "typed-magic-seed";
$node = ["value" => &$source, "plain" => ["value" => "typed-magic-copy"]];

$box = new Milestone1790_MagicBox();
$box->missing["node"] = $node;
$box->missing["node"]["value"] = "typed-magic";
$box->missing["node"]["plain"]["value"] = "typed-magic-plain";

echo $source,
    "|",
    $box->store["missing"]["node"]["plain"]["value"],
    "|",
    $box->log[0];
