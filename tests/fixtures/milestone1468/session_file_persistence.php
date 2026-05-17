<?php
ini_set("session.save_path", "/tmp");
session_id("phpcmilestone1468");
session_start(["use_cookies" => false]);
$_SESSION["token"] = "saved";
$_SESSION["nested"]["role"] = "admin";
session_write_close();
$_SESSION["token"] = "closed-edit";
$_SESSION["nested"]["role"] = "guest";
session_start(["use_cookies" => false]);
echo $_SESSION["token"], "|", $_SESSION["nested"]["role"];
