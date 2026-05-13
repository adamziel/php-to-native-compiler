<?php
function join_label($prefix, $name = "Ada") {
    if ($name === "stop") {
        return $prefix . ":stopped";
    }
    return $prefix . ":" . $name;
}

echo join_label("hello"), "\n";
echo join_label("bye", "Grace");
return "finished";
echo "after";
