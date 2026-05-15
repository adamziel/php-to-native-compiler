<?php
class wpdb {
    public $dbh;
    public $rows_affected;
    public $insert_id;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->rows_affected = -1;
        $this->insert_id = -1;
    }

    public function query($query) {
        mysqli_query($this->dbh, $query);
        $this->rows_affected = mysqli_affected_rows($this->dbh);
        $this->insert_id = mysqli_insert_id($this->dbh);

        while (mysqli_more_results($this->dbh)) {
            mysqli_next_result($this->dbh);
        }

        return $this->rows_affected;
    }
}

$wpdb = new wpdb();
$result = $wpdb->query("SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
echo $result, "\n";
echo $wpdb->rows_affected, "\n";
echo $wpdb->insert_id;
