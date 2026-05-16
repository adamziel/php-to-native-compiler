<?php
class Router {
    public function known() {
        return "known";
    }

    public function __call($method, $args) {
        echo "call:$method\n";
        return $method . ":" . $args[0] . ":" . $args[1];
    }
}

$router = new Router();
echo $router->known(), "\n";
echo $router->route("posts", 7);
