<?php
class Box {
    private $store = [];
    public $log = [];

    public function seed($key, $value) {
        $this->store[$key] = $value;
    }

    public function read($key) {
        return $this->store[$key];
    }

    public function &__get(string $name): mixed {
        $this->log[] = "get:" . $name;
        return $this->store[$name];
    }

    public function snapshotTry() {
        $this->log[] = "try";
        try {
            return $this->store;
        } finally {
            $this->log[] = "finally";
        }
    }
}

$box = new Box();
$box->seed("x", "old");
$alias =& $box->x;
$copy = $box->snapshotTry();
$alias = "new";
$copy["x"] = "copy";
echo $box->read("x"), "|", $alias, "|", $copy["x"], "|", implode(",", $box->log);
