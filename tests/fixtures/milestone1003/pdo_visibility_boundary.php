<?php
echo extension_loaded("pdo") ? "pdo" : "missing";
echo "|";
echo extension_loaded("pdo_mysql") ? "pdo-mysql" : "missing";
echo "|";
echo class_exists("PDO") ? "class" : "missing";
echo "|";
echo class_exists("PDOStatement") ? "statement" : "missing";
