<?php
$items = ["a" => 1, "b" => 0];
$result = array_filter($items, "strlen", ARRAY_FILTER_USE_BOTH);
