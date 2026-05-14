<?php
function mark($label) {
    echo $label;
    return true;
}

assert(mark("A"), mark("B"));
echo "C";
