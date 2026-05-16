<?php
$raw = file_get_contents("php://input");
echo $raw === "" ? "empty" : "non-empty";
