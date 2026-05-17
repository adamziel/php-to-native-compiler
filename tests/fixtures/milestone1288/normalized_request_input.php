<?php
echo isset($_GET["user_login"]) ? $_GET["user_login"] : "empty";
echo "|";
echo isset($_GET["remember_me"]) ? $_GET["remember_me"] : "empty";
echo "|";
echo isset($_GET["nested_key"]["child space"]) ? $_GET["nested_key"]["child space"] : "empty";
echo "|";
echo isset($_POST["action_name"]) ? $_POST["action_name"] : "empty";
echo "|";
echo isset($_POST["form_name"]["inner.dot"]) ? $_POST["form_name"]["inner.dot"] : "empty";
echo "|";
echo isset($_REQUEST["dup_key"]) ? $_REQUEST["dup_key"] : "empty";
