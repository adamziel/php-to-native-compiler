<?php
echo defined("PDO::ATTR_ERRMODE") ? "defined" : "missing";
echo "|";
echo PDO::ATTR_ERRMODE;
echo ":";
echo PDO::ERRMODE_EXCEPTION;
echo ":";
echo PDO::FETCH_ASSOC;
echo ":";
echo PDO::FETCH_NUM;
echo ":";
echo PDO::FETCH_BOTH;
echo ":";
echo PDO::MYSQL_ATTR_INIT_COMMAND;
echo "|";
echo constant("PDO::ATTR_DEFAULT_FETCH_MODE");
