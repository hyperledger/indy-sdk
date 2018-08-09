#!/bin/bash

set -e

if [ ! -d "/var/lib/indy/sandbox/keys" ]; then
  echo "Ledger does not exist - recreating..."

  echo generate_indy_pool_transactions --nodes 4 --clients 5 --nodeNum 1 2 3 4 --ips="$POOL_IP,$POOL_IP,$POOL_IP,$POOL_IP"
  su -c "generate_indy_pool_transactions --nodes 4 --clients 5 --nodeNum 1 2 3 4 --ips=\"$POOL_IP,$POOL_IP,$POOL_IP,$POOL_IP\"" indy

  echo "Generated genesis transaction file:"
  echo "/home/indy/.indy-cli/networks/sandbox/pool_transactions_genesis"
  cat /home/indy/.indy-cli/networks/sandbox/pool_transactions_genesis

else
  echo "Ledger exists - using..."
fi
echo "/usr/bin/supervisord"
su -c "/usr/bin/supervisord" indy