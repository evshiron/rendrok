#!/bin/sh

export AUTH="rendrok:$RENDROK_PASS"
#echo $AUTH
/app/chisel server --reverse --backend http://127.0.0.1:3000
