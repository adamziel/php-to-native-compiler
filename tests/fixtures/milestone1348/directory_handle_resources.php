<?php
$dir = opendir(__DIR__ . "/directory_handle_fixture");
echo gettype($dir);

$entries = array();
while (($entry = readdir($dir)) !== false) {
    $entries[] = $entry;
}

echo "|";
echo in_array(".", $entries, true) ? "dot" : "missing-dot";
echo ":";
echo in_array("..", $entries, true) ? "dotdot" : "missing-dotdot";
echo ":";
echo in_array("alpha.txt", $entries, true) ? "alpha" : "missing-alpha";
echo ":";
echo in_array("beta.inc", $entries, true) ? "beta" : "missing-beta";
echo ":";
echo in_array("nested", $entries, true) ? "nested" : "missing-nested";
echo ":";
echo count($entries);

rewinddir($dir);
$rewound = array();
while (($entry = readdir($dir)) !== false) {
    $rewound[] = $entry;
}
echo "|rewound:";
echo count($rewound);
closedir($dir);
