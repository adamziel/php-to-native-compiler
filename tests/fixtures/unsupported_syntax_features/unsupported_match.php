<?php
$status = 200;
$label = match ($status) {
    200 => "ok",
    default => "other",
};
echo $label;
