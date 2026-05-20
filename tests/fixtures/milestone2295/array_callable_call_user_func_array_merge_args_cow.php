<?php
class Milestone2295Callable {
    public function pass($arr) {
        return $arr;
    }
}

class BoxCallable {
    public $store;
    public $callback;

    public function __get($name) {
        $copy = $this->store;
        return call_user_func_array(
            $this->callback,
            array_merge(array($copy))
        );
    }
}

$box = new BoxCallable();
$box->store = array("plain" => "old", "ref" => array("v" => "leaf"));
$box->callback = array(new Milestone2295Callable(), "pass");
$alias =& $box->store["ref"]["v"];
$tmp = $box->missing;
$tmp["ref"]["v"] = "updated";
echo $box->store["plain"], "\n";
echo $box->store["ref"]["v"];
