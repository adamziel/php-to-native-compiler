<?php
function counter() {
    static $count = next_value();
}
