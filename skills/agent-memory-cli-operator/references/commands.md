# Command Cookbook

## Identity bootstrap
```bash
agent-memory-cli admin migrate --db data/agent-memory.db
agent-memory-cli --db data/agent-memory.db user create --name "Yongseong"
agent-memory-cli --db data/agent-memory.db scope create --id private:test --type private
```

## Request pattern logging
```bash
echo '{"pattern":"restaurant_reco"}' > /tmp/request.json
agent-memory-cli --db data/agent-memory.db ingest event \
  --uid <uid> --scope private:test \
  --type request.logged --file /tmp/request.json \
  --idempotency-key req-001
```

## Read back
```bash
agent-memory-cli --db data/agent-memory.db query topk \
  --uid <uid> --scope private:test --topic request_pattern --limit 3

agent-memory-cli --db data/agent-memory.db query metric \
  --uid <uid> --scope private:test --prefix counter:request_pattern:
```
