<?php
class Milestone1786_MagicBox {
    public $store = [];
    public $log = [];

    public function &__get($name) {
        try {
            throw new Exception();
        } catch (Exception $e) {
            $this->log[] = "catch:" . get_class($e);
            if (!isset($this->store[$name])) {
                $this->store[$name] = [];
            }
            return $this->store[$name];
        } finally {
            $this->log[] = "finally";
        }
    }
}

$source = "magic-seed";
$node = ["value" => &$source, "plain" => ["value" => "magic-copy"]];

$box = new Milestone1786_MagicBox();
$box->missing["node"] = $node;
$box->missing["node"]["value"] = "magic-caught";
$box->missing["node"]["plain"]["value"] = "magic-plain-caught";

echo $source,
    "|",
    $box->store["missing"]["node"]["plain"]["value"],
    "|",
    $box->log[0],
    "|",
    $box->log[1];
