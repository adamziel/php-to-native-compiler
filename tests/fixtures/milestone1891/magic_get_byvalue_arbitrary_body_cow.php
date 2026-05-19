<?php
class Milestone1891_Box {
    public $store = [];
    public $log = [];

    public function seed(&$leaf) {
        $this->store["outer"] = ["leaf" => &$leaf, "plain" => "old"];
    }

    private function choose($name) {
        while (true) {
            return $this->store[$name];
        }
        return [];
    }

    public function __get($name) {
        $this->log[] = "get:" . $name;
        $bucket = $this->choose($name);
        if ($name === "outer") {
            return $bucket;
        }
        return [];
    }
}

$leaf = "seed";
$box = new Milestone1891_Box();
$box->seed($leaf);
$copy = $box->outer;
$copy["leaf"] = "copy";

echo $leaf, "|", $box->store["outer"]["leaf"], "|", $copy["leaf"], "|", $box->log[0];
