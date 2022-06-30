watch:
	cargo watch -c -q -x 'run --bin websocket_client'

tunnel:
	ssh -L localhost:26657:localhost:26657 stargaze@46.101.185.129
