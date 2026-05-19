<?php
class Milestone1892_Box {
    public $store = [];
    public $log = [];

    public function seed(&$leaf) {
        $this->store["outer"] = [
            "group" => ["leaf" => &$leaf, "plain" => "old"],
        ];
    }

    private function choose($name) {
        if ($name === "outer") {
            return $this->store[$name];
        }
        return [];
    }

    public function __get($name) {
        $this->log[] = "get:" . $name;
        $bucket = $this->choose($name);
        switch ($name) {
            case "outer":
                return $bucket;
            default:
                return [];
        }
    }
}

$leaf = "seed";
$box = new Milestone1892_Box();
$box->seed($leaf);
$copy = $box->outer["group"];
$copy["leaf"] = "copy";

echo $leaf, "|", $box->store["outer"]["group"]["leaf"], "|", $copy["leaf"], "|", $box->log[0];
