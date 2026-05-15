<?php
define("OBJECT", "OBJECT");
define("ARRAY_A", "ARRAY_A");

class wpdb {
    public $dbh;
    public $result;
    public $last_result;
    public $num_rows;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_result = [];
        $this->num_rows = 0;
    }

    public function _do_query($query) {
        $this->result = mysqli_query($this->dbh, $query);
    }

    public function get_results($query, $output = OBJECT) {
        $this->last_result = [];
        $this->num_rows = 0;
        $this->_do_query($query);

        if ($output === ARRAY_A) {
            $row = mysqli_fetch_assoc($this->result);
            while ($row) {
                $this->last_result[] = $row;
                $this->num_rows++;
                $row = mysqli_fetch_assoc($this->result);
            }
        } else {
            $row = mysqli_fetch_object($this->result);
            while ($row) {
                $this->last_result[] = $row;
                $this->num_rows++;
                $row = mysqli_fetch_object($this->result);
            }
        }

        mysqli_free_result($this->result);
        while (mysqli_more_results($this->dbh)) {
            mysqli_next_result($this->dbh);
        }

        return $this->last_result;
    }
}

$wpdb = new wpdb();
$rows = $wpdb->get_results("SELECT ID, post_title FROM wp_posts WHERE ID = 1", ARRAY_A);
echo count($rows), "\n";
echo $wpdb->num_rows, "\n";
echo $rows[0]["ID"], "\n";
echo $rows[0]["post_title"], "\n";
echo $wpdb->last_result[0]["post_title"];
