<?php
function cast_value($mixedVar) {
    $mixedVar = (float) $mixedVar;
    return $mixedVar;
}

echo cast_value("2.5"), "|", cast_value(4), "|", cast_value(false);
