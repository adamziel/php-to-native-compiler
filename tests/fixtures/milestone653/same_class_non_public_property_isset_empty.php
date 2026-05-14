<?php
class Box {
    private $secret;
    protected $label;

    public function set($secret, $label) {
        $this->secret = $secret;
        $this->label = $label;
    }

    public function checks($other) {
        echo isset($this->secret) ? "this-secret:set\n" : "this-secret:unset\n";
        echo empty($this->secret) ? "this-secret:empty\n" : "this-secret:not-empty\n";
        echo isset($other->label) ? "peer-label:set\n" : "peer-label:unset\n";
        echo empty($other->label) ? "peer-label:empty" : "peer-label:not-empty";
    }
}

$first = new Box();
$second = new Box();
$first->set("0", "main");
$second->set(null, "");
$first->checks($second);
