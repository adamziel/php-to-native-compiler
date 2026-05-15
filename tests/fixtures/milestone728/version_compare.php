<?php
echo version_compare("7.2.24", "8.3.0"), "\n";
echo version_compare("8.3", "8.3.0"), "\n";
echo version_compare("8.3.1", "8.3.0"), "\n";
echo version_compare("7.2.24", PHP_VERSION, "<") ? "lt" : "ge";
echo "|", version_compare(PHP_VERSION, PHP_VERSION, ">=") ? "ge" : "lt";
echo "|", version_compare("8-3-0", "8.3.0", "=") ? "eq" : "ne";
