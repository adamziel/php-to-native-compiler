<?php
function describe_handlers($label, $handlers) {
    echo $label . "=" . count($handlers);
    foreach ($handlers as $handler) {
        echo ":" . $handler;
    }
}

describe_handlers("initial", ob_list_handlers());
ob_start();
$outer = ob_list_handlers();
echo "outer";
ob_start();
$inner = ob_list_handlers();
echo "|inner";
$inner_capture = ob_get_clean();
echo "|after-inner=" . count(ob_list_handlers());
$outer_capture = ob_get_clean();
echo "|";
describe_handlers("outer", $outer);
echo "|";
describe_handlers("inner", $inner);
echo "|outer-capture=[" . $outer_capture . "]";
echo "|inner-capture=[" . $inner_capture . "]";
echo "|";
describe_handlers("final", ob_list_handlers());
