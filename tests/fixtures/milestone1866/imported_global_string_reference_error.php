<?php
$store = "abc";

function milestone1866_trigger() {
global $store;
$alias =& $store[0];
}
milestone1866_trigger();
