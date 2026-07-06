#!/bin/zsh

HOST="0.0.0.0:3000"
PROMPT="What does Carbon offset programs do?"
PROMPT="programs"
echo $PROMPT
curl $HOST -d '{"prompt":"'$PROMPT'"}'
