<?php
echo ini_get("user_agent"), "|";
echo ini_set("user_agent", "phpc"), "|";
echo ini_get("user_agent"), "|";
echo ini_set("user_agent", "native"), "|";
echo ini_get("user_agent"), "|";
echo ini_set("missing.option", "x") === false ? "false" : "not-false";
