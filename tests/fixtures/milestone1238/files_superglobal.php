<?php
echo is_array($_FILES) ? "files-array" : "files-missing";
echo "|";
echo isset($_FILES["async-upload"]) ? "upload" : "files-empty";

function wp_upload_probe() {
    $_FILES["async-upload"] = ["name" => "plugin.zip", "error" => 0];
}

wp_upload_probe();
echo "|";
echo $_FILES["async-upload"]["name"];
echo ":";
echo $_FILES["async-upload"]["error"];
