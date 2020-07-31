#!/bin/sh

DIR=`dirname $0`

rm /tmp/cava.fifo
mkfifo /tmp/cava.fifo
(
	trap 'kill 0' SIGINT
	cava -p "${DIR}/cava.conf" &
	${DIR}/target/release/equalizer-tx
)
