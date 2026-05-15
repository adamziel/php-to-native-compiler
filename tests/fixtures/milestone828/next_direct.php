<?php
$items = array("first", "second");
echo current($items), "|";
echo next($items), "|";
echo current($items), "|";
var_dump(next($items));
class HookLike {
    public $iterations = array();

    public function run() {
        $level = 0;
        $this->iterations[$level] = array(10, 20);
        echo current($this->iterations[$level]), "|";
        echo next($this->iterations[$level]), "|";
        echo current($this->iterations[$level]);
    }
}
$hook = new HookLike();
$hook->run();
