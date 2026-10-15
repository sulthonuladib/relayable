#!/usr/bin/env bash
# DOC-002 fetch batch. Writes ONLY under /tmp/doc002/.
set -u
BATCH_START=$(date -u +%Y-%m-%dT%H:%M:%SZ)
echo "BATCH_START=$BATCH_START"
declare -a RESULT_LINES
fail_count=0
while IFS=' ' read -r slug url; do
  [ -z "$slug" ] && continue
  echo "########## SLUG=$slug ##########"
  echo "--- date_before ---"
  D0=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  echo "$D0"
  echo "--- curl ---"
  curl -sS -L --max-time 30 --connect-timeout 15 -o "/tmp/doc002/raw/$slug" -w 'HTTP=%{http_code} SIZE=%{size_download} URL=%{url_effective}\n' "$url"
  rc=$?
  echo "curl_exit=$rc"
  echo "--- date_after ---"
  D1=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  echo "$D1"
  echo "--- sha256 ---"
  if [ -f "/tmp/doc002/raw/$slug" ]; then
    sha256sum "/tmp/doc002/raw/$slug"
  else
    echo "NO_OUTPUT_FILE"
  fi
  echo "SLUGRECORD|$slug|$url|$D0|$D1|$rc"
  if [ "$rc" -ne 0 ]; then
    fail_count=$((fail_count+1))
    if [ "$fail_count" -ge 2 ]; then
      echo "NETWORK_FAILURE_STOP: two curl non-zero exits; aborting remaining fetches."
      break
    fi
  fi
done < /tmp/doc002/urls.txt
BATCH_END=$(date -u +%Y-%m-%dT%H:%M:%SZ)
echo "BATCH_END=$BATCH_END"
