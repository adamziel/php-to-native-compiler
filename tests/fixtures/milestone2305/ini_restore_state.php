<?php
echo ini_get("user_agent") === "" ? "default" : "unexpected";
echo "|";
echo ini_set("user_agent", "phpc");
echo "|";
echo ini_get("user_agent");
echo "|";
echo ini_restore("user_agent") === null ? "null" : "other";
echo "|";
echo ini_get("user_agent") === "" ? "restored" : ini_get("user_agent");
echo "|";
echo ini_restore("missing.option") === null ? "missing-null" : "missing-other";
