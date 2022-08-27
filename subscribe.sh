#!/bin/bash

source .private

twitch_access_token=$(cat .twitch_access_token | jq .access_token | sed 's/"//g')

meow_secret=$(cat .meow_secret)

if [ -z "$twitch_access_token" ]; then
    echo "Access token is empty"
    exit 1
fi

if [ -z "$meow_secret" ]; then
    echo "meow_secret is empty"
    exit 1
fi

curl -X POST 'https://api.twitch.tv/helix/eventsub/subscriptions' \
-H 'Authorization: Bearer '$twitch_access_token \
-H 'Client-Id: '$twitch_client_id \
-H 'Content-Type: application/json' \
-d '{"type":"stream.online","version":"1","condition":{"broadcaster_user_id":"576291377"},"transport":{"method":"webhook","callback":"https://futuristic-toaster.backend.rooot.gay/twitch/sub/callback","secret":"'$meow_secret'"}}'