<?php
$required_php_extensions = array("json", "hash");
$missing_extensions = array();

foreach ($required_php_extensions as $extension) {
    if (extension_loaded($extension)) {
        continue;
    }

    $missing_extensions[] = $extension;
}

echo count($missing_extensions) === 0 ? "ok" : implode(",", $missing_extensions);

