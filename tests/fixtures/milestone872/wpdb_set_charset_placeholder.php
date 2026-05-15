<?php
class wpdb {
    public $dbh;
    public $charset;
    public $collate;
    public $charset_set;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->charset = "utf8mb4";
        $this->collate = "utf8mb4_unicode_520_ci";
        $this->charset_set = false;
    }

    public function set_charset($charset, $collate) {
        $this->charset = $charset;
        $this->collate = $collate;
        $this->charset_set = mysqli_set_charset($this->dbh, $charset);

        return $this->charset_set;
    }
}

$wpdb = new wpdb();
echo $wpdb->set_charset("utf8mb4", "utf8mb4_unicode_520_ci") ? "set" : "failed", "\n";
echo $wpdb->charset, "\n";
echo $wpdb->collate, "\n";
echo $wpdb->charset_set ? "recorded" : "missing";
