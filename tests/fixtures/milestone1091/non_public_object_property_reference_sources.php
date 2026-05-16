<?php
class Box {
    private $secret = "initial";
    protected $label = "start";

    public function run() {
        $secret =& $this->secret;
        $secret = "secret-alias";
        echo $this->secret, "|";
        $this->secret = "secret-property";
        echo $secret, "|";

        $label =& $this->label;
        $label = "label-alias";
        echo $this->label, "|";
        $this->label = "label-property";
        echo $label;
    }
}

$box = new Box();
$box->run();

echo "|";

class Base {
    protected $shared = "base";

    public function readShared() {
        return $this->shared;
    }
}

class Child extends Base {
    public function aliasOwn() {
        $alias =& $this->shared;
        $alias = "own-alias";
        echo $this->shared, "|";
        $this->shared = "own-property";
        echo $alias;
    }

    public function aliasPeer($other) {
        $alias =& $other->shared;
        $alias = "peer-alias";
        echo $other->readShared(), "|";
        $other->shared = "peer-property";
        echo $alias;
    }
}

$child = new Child();
$child->aliasOwn();
echo "|";
$peer = new Child();
$child->aliasPeer($peer);
