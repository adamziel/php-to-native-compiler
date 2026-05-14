<?php
class Box {
    public $name;
    public $count;

    public function label($prefix = "box") {
        return $prefix . ":" . $this->name;
    }

    public function rename($name) {
        $this->name = $name;
        $this->count++;
        return $this->label("renamed");
    }

    public function touch() {
        $this->count = $this->count + 1;
    }
}

$box = new Box();
$box->name = "Ada";
$box->count = 0;
$alias = $box;

echo $box->label("user"), "\n";
echo $box->LABEL(), "\n";
echo $box->rename("Grace"), "\n";
echo $alias->name, "\n";
$box->touch();
echo $box->count, "\n";
