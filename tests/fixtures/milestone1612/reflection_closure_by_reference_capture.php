<?php
$counter = 10;
$callback = function ($hook) use (&$counter) {
    $counter = $counter + 1;
    return $hook . ":" . $counter;
};
$counter = 20;

$function = new ReflectionFunction($callback);
echo $function->invoke("init"), "|", $counter, "\n";
$counter = 30;
echo $function->invokeArgs(array("save_post")), "|", $counter, "\n";

function make_reflected_counter() {
    $local = "start";
    $callback = function ($next) use (&$local) {
        $local = $next;
        return $local;
    };
    return new ReflectionFunction($callback);
}

$reflected = make_reflected_counter();
echo $reflected->invoke("kept"), "\n";
echo $reflected->invokeArgs(array("updated"));
