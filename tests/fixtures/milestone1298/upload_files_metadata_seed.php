<?php
echo isset($_FILES["async-upload"]["name"]) ? $_FILES["async-upload"]["name"] : "empty";
echo "|";
echo isset($_FILES["async-upload"]["type"]) ? $_FILES["async-upload"]["type"] : "empty";
echo "|";
echo isset($_FILES["async-upload"]["tmp_name"]) ? $_FILES["async-upload"]["tmp_name"] : "empty";
echo "|";
echo isset($_FILES["async-upload"]["error"]) ? $_FILES["async-upload"]["error"] : "empty";
echo "|";
echo isset($_FILES["async-upload"]["size"]) ? $_FILES["async-upload"]["size"] : "empty";
echo "|";
echo isset($_FILES["async-upload"]["full_path"]) ? $_FILES["async-upload"]["full_path"] : "empty";
echo "|";

function upload_summary() {
    if (!isset($_FILES["async-upload"]["name"])) {
        return "no-upload";
    }
    return $_FILES["async-upload"]["name"] . ":" . $_FILES["async-upload"]["size"];
}

echo upload_summary();
