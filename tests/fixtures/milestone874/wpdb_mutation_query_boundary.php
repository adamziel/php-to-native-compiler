<?php
class wpdb {
    public $dbh;
    public $last_query;
    public $rows_affected;
    public $insert_id;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_query = "";
        $this->rows_affected = -1;
        $this->insert_id = -1;
    }

    public function query($query) {
        $this->last_query = $query;
        echo $this->last_query, "\n";

        mysqli_query($this->dbh, $query);
        $this->rows_affected = mysqli_affected_rows($this->dbh);
        $this->insert_id = mysqli_insert_id($this->dbh);

        return $this->rows_affected;
    }
}

$wpdb = new wpdb();
$wpdb->query("UPDATE wp_options SET option_value = '1' WHERE option_name = 'blog_public'");
