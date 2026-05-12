<?php
$items = ["Ada"];
echo array_reduce($items, "strlen", "start");
