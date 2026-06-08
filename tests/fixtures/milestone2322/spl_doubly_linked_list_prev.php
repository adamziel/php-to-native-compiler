<?php
$list = new SplDoublyLinkedList();
$list->push(1);
$list->push(2);
$list->push(3);
$list->push(4);

$list->rewind();
$list->prev();
var_dump($list->current());
$list->rewind();
var_dump($list->current());
$list->next();
var_dump($list->current());
$list->next();
$list->next();
var_dump($list->current());
$list->prev();
var_dump($list->current());

$list->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO);
$list->rewind();
var_dump($list->current());
$list->next();
var_dump($list->current());
$list->prev();
var_dump($list->current());

$empty = new SplDoublyLinkedList();
$empty->rewind();
$empty->prev();
echo var_export($empty->current(), true);
