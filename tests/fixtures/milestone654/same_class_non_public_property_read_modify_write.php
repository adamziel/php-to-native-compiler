<?php
class Counter {
    private $count;
    protected $score;

    public function seed($count, $score) {
        $this->count = $count;
        $this->score = $score;
    }

    public function bump($other) {
        $this->count += 4;
        $this->score *= 3;
        echo $this->count, ":", $this->score, "\n";
        echo $other->count++, "\n";
        echo ++$other->score, "\n";
        $other->count .= "!";
    }

    public function describe() {
        return $this->count . ":" . $this->score;
    }
}

$first = new Counter();
$second = new Counter();
$first->seed(6, 2);
$second->seed(10, 20);
$first->bump($second);
echo $second->describe();
