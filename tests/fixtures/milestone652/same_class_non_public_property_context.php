<?php
class Box {
    private $secret;
    protected $label;

    public function set($secret, $label) {
        $this->secret = $secret;
        $this->label = $label;
    }

    public function describe() {
        return $this->secret . ":" . $this->label;
    }

    public function copyTo($other) {
        $other->secret = $this->secret;
        $other->label = "copy";
    }
}

$first = new Box();
$second = new Box();
$first->set("one", "main");
echo $first->describe(), "\n";
$first->copyTo($second);
echo $second->describe();
