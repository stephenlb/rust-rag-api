#!/bin/zsh
HOST="0.0.0.0:3000"

## Test basic prompt
PROMPT="hello how is it going"
curl $HOST/ -d '{"prompt":"'$PROMPT'"}'

## Test Document loading
DOCUMENT="day is going well thank you"
curl $HOST/doc -d '{"":"'$DOCUMENT'"}'
