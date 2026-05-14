<?php
function local_name() {}

echo is_callable("missing", true) ? "1" : "0";
echo is_callable("not valid", true) ? "1" : "0";
echo is_callable("local_name", false) ? "1" : "0";
echo is_callable("missing", false) ? "1" : "0";
echo is_callable(42, true) ? "1" : "0";
echo is_callable(null, true) ? "1" : "0";
