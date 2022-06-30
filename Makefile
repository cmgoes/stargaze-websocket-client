watch:
	cargo watch -c -q -x 'run --bin websocket_client'

tunnel:
	ssh -L localhost:26657:localhost:26657 ${REMOTE_STARGAZE_NODE}
