<?php
class wpdb {
    public $dbh;
    public $options;

    public function __construct($prefix) {
        $this->options = $prefix . 'options';
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function prepare_two($query, $one, $two) {
        return array($query, array($one, $two));
    }

    public function get_col($query) {
        $result = mysqli_query($this->dbh, $query);
        $values = array();
        while (($value = mysqli_fetch_column($result)) !== false) {
            $values[] = $value;
        }
        return $values;
    }

    public function get_col_prepared($prepared) {
        $stmt = mysqli_prepare($this->dbh, $prepared[0]);
        $one = $prepared[1][0];
        $two = $prepared[1][1];
        mysqli_stmt_bind_param($stmt, 'ss', $one, $two);
        mysqli_stmt_execute($stmt);
        $result = mysqli_stmt_get_result($stmt);
        $values = array();
        while (($value = mysqli_fetch_column($result)) !== false) {
            $values[] = $value;
        }
        return $values;
    }

    public function get_col_prepared_query($query) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_execute($stmt);
        $result = mysqli_stmt_get_result($stmt);
        $values = array();
        while (($value = mysqli_fetch_column($result)) !== false) {
            $values[] = $value;
        }
        return $values;
    }
}

$wpdb = new wpdb('wp_');
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($wpdb->dbh, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");

$autoload = $wpdb->prepare_two(
    'SELECT `option_name` FROM `' . $wpdb->options . '` WHERE `autoload` IN (?, ?)',
    'yes',
    'on'
);
echo 'autoload=' . implode(',', $wpdb->get_col_prepared($autoload));

echo '|';
$named = $wpdb->get_col("SELECT option_name FROM wp_options WHERE option_name IN ('theme_mods','missing','home')");
echo 'named=' . implode(',', $named);

echo '|';
$all = $wpdb->get_col_prepared_query('SELECT option_name FROM wp_options');
echo 'all_prepared=' . implode(',', $all);
