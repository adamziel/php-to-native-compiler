<?php
class wpdb {
    public $dbh;
    public $autocommit_enabled;
    public $autocommit_checked;
    public $autocommit_changes;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->autocommit_enabled = true;
        $this->autocommit_checked = false;
        $this->autocommit_changes = 0;
    }

    public function set_autocommit($enabled) {
        if (mysqli_autocommit($this->dbh, $enabled)) {
            $this->autocommit_enabled = $enabled;
            $this->autocommit_checked = true;
            $this->autocommit_changes = $this->autocommit_changes + 1;
            return true;
        }

        return false;
    }
}

$wpdb = new wpdb();
echo $wpdb->set_autocommit(false) ? "disabled" : "failed";
echo "\n";
echo $wpdb->set_autocommit(true) ? "enabled" : "failed";
echo "\n";
echo $wpdb->autocommit_checked ? "checked" : "skipped";
echo "\n";
echo $wpdb->autocommit_enabled ? "on" : "off";
echo "\n";
echo $wpdb->autocommit_changes;
