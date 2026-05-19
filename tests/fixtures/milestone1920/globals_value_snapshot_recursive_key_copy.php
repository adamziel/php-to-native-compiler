<?php
$GLOBALS["GLOBALS"] = "recursive";
$snapshot = $GLOBALS;
$snapshot["GLOBALS"] = "copy";

echo $GLOBALS["GLOBALS"], "|", $snapshot["GLOBALS"];
