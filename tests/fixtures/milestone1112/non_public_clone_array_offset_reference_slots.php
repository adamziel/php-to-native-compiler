<?php
class Base {
    protected $items = ["slot" => "base"];

    public function readItem($key) {
        return $this->items[$key];
    }
}

class Box extends Base {
    private $privateItems = ["slot" => "private"];
    private $privateAppends = [];

    public function exercisePrivate($key) {
        $alias =& $this->privateItems[$key];
        $copy = clone $this;
        $copy->privateItems[$key] = "copy-private";
        echo $alias, "|", $this->privateItems[$key], "|", $copy->privateItems[$key], "\n";
        $alias = "alias-private";
        echo $alias, "|", $this->privateItems[$key], "|", $copy->privateItems[$key], "\n";
    }

    public function exercisePrivateAppend() {
        $alias =& $this->privateAppends[];
        $copy = clone $this;
        $copy->privateAppends[0] = "copy-append";
        echo $alias, "|", $this->privateAppends[0], "|", $copy->privateAppends[0], "\n";
    }

    public function exerciseProtectedPeer($other, $key) {
        $alias =& $other->items[$key];
        $copy = clone $other;
        $copy->items[$key] = "copy-protected";
        echo $alias, "|", $other->readItem($key), "|", $copy->readItem($key), "\n";
        $alias = "alias-protected";
        echo $alias, "|", $other->readItem($key), "|", $copy->readItem($key);
    }
}

$box = new Box();
$box->exercisePrivate("slot");
$box->exercisePrivateAppend();
$box->exerciseProtectedPeer(new Box(), "slot");
