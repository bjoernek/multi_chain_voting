ifdef DFX_NETWORK
	NETWORK := $(DFX_NETWORK)
else
	NETWORK := local
endif

ifeq (${NETWORK}, ic)
	KEY := test_key_1
else
	KEY := dfx_test_key
endif

create-canisters:
	dfx canister create --all --network "${NETWORK}"

deploy-provider:
	dfx deploy evm_rpc --with-cycles 1t --network "${NETWORK}" --argument "( \
		record { \
			nodesInSubnet = 25; \
		} \
	)"
	dfx deploy ic_siwe_provider --with-cycles 1t --network "${NETWORK}" --argument "( \
	    record { \
	        domain = \"icp0.io\"; \
	        uri = \"https://$$(dfx canister id --network "${NETWORK}" frontend).icp0.io\"; \
	        salt = \"salt\"; \
	        chain_id = opt 11155111; \
	        scheme = opt \"http\"; \
	        statement = opt \"Login to the SIWE/IC demo app\"; \
	        sign_in_expires_in = opt 300000000000; /* 5 minutes */ \
	        session_expires_in = opt 604800000000000; /* 1 week */ \
	        targets = opt vec { \
	            \"$$(dfx canister id --network "${NETWORK}" ic_siwe_provider)\"; \
	            \"$$(dfx canister id --network "${NETWORK}" backend)\"; \
	        }; \
	    } \
	)"

upgrade-provider:
	dfx canister install ic_siwe_provider --mode upgrade --upgrade-unchanged --argument "( \
	    record { \
	        domain = \"icp0.io\"; \
	        uri = \"https://$$(dfx canister id --network "${NETWORK}" ic_siwe_provider).icp0.io\"; \
	        salt = \"salt\"; \
	        chain_id = opt 1; \
	        scheme = opt \"http\"; \
	        statement = opt \"Login to the SIWE/IC demo app\"; \
	        sign_in_expires_in = opt 300000000000; /* 5 minutes */ \
	        session_expires_in = opt 604800000000000; /* 1 week */ \
	        targets = opt vec { \
	            \"$$(dfx canister id --network "${NETWORK}" ic_siwe_provider)\"; \
	            \"$$(dfx canister id --network "${NETWORK}" backend)\"; \
	        }; \
	    } \
	)"

deploy-backend:
	CANISTER_CANDID_PATH_EVM_RPC=../../src/evm_rpc.did dfx deploy backend --network "${NETWORK}" --argument "(\"${KEY}\")"

deploy-frontend:
	npm install
	CANISTER_CANDID_PATH_EVM_RPC=../../src/evm_rpc.did dfx deploy frontend --network "${NETWORK}"

deploy-all: create-canisters deploy-provider deploy-backend deploy-frontend

run-frontend:
	npm install
	npm run dev

clean:
	rm -rf .dfx
	rm -rf dist
	rm -rf node_modules
	rm -rf src/declarations
	rm -f .env
	cargo clean
