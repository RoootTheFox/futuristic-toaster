#!/bin/bash
echo "meow"

source .private

curl -X POST https://id.twitch.tv/oauth2/token -H "Content-Type: application/x-www-form-urlencoded" \
    -d "client_id="$twitch_client_id"&client_secret="$twitch_client_secret"&grant_type=client_credentials" > .twitch_access_token