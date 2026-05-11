<?php
function loop($n) {
    return loop($n + 1);
}
echo loop(0);
