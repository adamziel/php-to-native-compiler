<?php

$path = __DIR__ . "/../hello.php";
$summary = "PTN:" . strlen("native");

if (str_contains($path, "examples")) {
    echo "path-ok\n";
}

for ($i = 1; $i <= 3; $i++) {
    echo "for:", $i, "\n";
}

$i = 0;
while ($i < 2) {
    echo "while:", $i, "\n";
    $i++;
}

switch (dirname($path)) {
    case __DIR__ . "/..":
        echo "dir-parent\n";
        break;
    default:
        echo "dir-other\n";
}

echo bin2hex("PTN"), "\n";
echo $summary, "\n";
