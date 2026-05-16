<?php
class OptionFilter {
    public function update(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

$filter = new OptionFilter();
$option = "autoload";
echo call_user_func_array(array($filter, "update"), array(&$option, "object-cache")), "\n";
echo $option;
