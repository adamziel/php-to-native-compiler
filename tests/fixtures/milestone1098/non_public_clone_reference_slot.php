<?php
class Box {
    private $secret = "initial";

    public function run() {
        $alias =& $this->secret;
        $copy = clone $this;
        $copy->secret = "copy-secret";
        echo $alias, "|", $this->secret, "|", $copy->secret, "\n";
        $alias = "alias-secret";
        echo $alias, "|", $this->secret, "|", $copy->secret;
    }
}

$box = new Box();
$box->run();
