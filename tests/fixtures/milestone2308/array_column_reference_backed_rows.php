<?php
function rewriteRows(&$rows) {
    $next = 10;
    foreach ($rows as &$row) {
        $row["id"] = $next;
        $row["superhero"] = "robin" . $next;
        $next = $next + 1;
    }
}

$rows = [
    ["id" => "before-a", "superhero" => "superman"],
    ["id" => "before-b", "superhero" => "acuaman"],
];

echo implode(",", array_column($rows, "superhero")), "\n";
rewriteRows($rows);
$names = array_column($rows, "superhero");
echo implode(",", $names), "\n";
$indexed = array_column($rows, "superhero", "id");
echo implode(",", array_keys($indexed)), "|", implode(",", $indexed), "\n";
$whole = array_column($rows, null, "id");
echo $whole[10]["superhero"], "|", $whole[11]["superhero"], "\n";
$call = "array_column";
echo implode(",", $call($rows, "id"));
