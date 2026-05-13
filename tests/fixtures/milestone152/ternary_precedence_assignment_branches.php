<?php
function trace($name, $value) {
    echo "call:", $name, "\n";
    return $value;
}

echo "coalesce precedence\n";
echo (($missing ?? false) ? trace("bad-condition-true", "bad") : trace("condition-false", "F")), "\n";
echo (false ? trace("bad-true", "bad") : $missing ?? trace("false-branch-coalesce", "C")), "\n";
echo (true ? $missing ?? trace("true-branch-coalesce", "T") : trace("bad-false", "bad")), "\n";

echo "assignment branches\n";
$left = "left-start";
$right = "right-start";
$picked = true ? ($left = trace("assign-left", "L")) : ($right = trace("assign-right", "R"));
echo $picked, ":", $left, ":", $right, "\n";

$count = 1;
$fallback = null;
$picked = false ? ($count += trace("bad-compound", 10)) : ($fallback ??= trace("coalesce-assign", "ready"));
echo $picked, ":", $count, ":", $fallback, "\n";

echo "short ternary coalesce\n";
echo (($missing ?? "kept") ?: trace("bad-short-fallback", "bad")), "\n";
echo (($empty ?? "") ?: trace("short-fallback", "short"));
