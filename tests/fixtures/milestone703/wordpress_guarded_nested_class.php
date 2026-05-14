<?php
if (!class_exists("WP_Privacy_Requests_Table")) {
    class WP_Privacy_Requests_Table {
        public static function label() {
            return "privacy";
        }
    }
}

echo WP_Privacy_Requests_Table::label(), "\n";
echo "after";
