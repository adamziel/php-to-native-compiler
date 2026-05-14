<?php
class Base {
    private $token;
    protected $shared;

    public function seedBase($token, $shared) {
        $this->token = $token;
        $this->shared = $shared;
    }

    public function describeBase() {
        return $this->token . ":" . $this->shared;
    }
}

class Child extends Base {
    private $childToken;
    protected $childShared;

    public function seedChild($token, $shared) {
        $this->childToken = $token;
        $this->childShared = $shared;
    }

    public function describeChild() {
        return $this->childToken . ":" . $this->childShared;
    }
}

$child = new Child();
$child->seedBase("base-token", "base-shared");
$child->seedChild("child-token", "child-shared");
echo $child->describeBase(), "\n";
echo $child->describeChild(), "\n";
print_r($child);
echo "done";
