<?php
$label = "print:";
$count = 222;
$truthy = true;
$falsey = false;
$nothing = null;

print $label;
echo $count, "\n";
print "line";
echo "-", $truthy, $falsey, $nothing, "\n";
print "done";
