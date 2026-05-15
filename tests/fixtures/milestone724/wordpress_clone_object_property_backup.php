<?php
class WP_Query_Like {
    public $query_vars;
}

$wp_query = new WP_Query_Like();
$wp_query->query_vars = "main";
$query = clone $wp_query;
$query->query_vars = "backup";

var_dump($wp_query === $query);
echo $wp_query->query_vars, ":", $query->query_vars;
