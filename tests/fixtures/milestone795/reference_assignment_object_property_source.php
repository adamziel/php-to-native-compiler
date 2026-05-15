<?php
class Query
{
    public $posts;
}

$query = new Query();
$query->posts = array('first');
$GLOBALS['posts'] =& $query->posts;
echo $GLOBALS['posts'][0];
