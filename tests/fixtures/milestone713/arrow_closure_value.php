<?php
$fn = fn ($value) => $value;
if ($fn) {
    echo "truthy\n";
}
echo "after\n";
