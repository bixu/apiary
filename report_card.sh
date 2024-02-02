#!/bin/sh

ids=$(apiary --api-key ${HONEYCOMB_API_KEY} --dataset="theta" --endpoint="slos" | jq --raw-output '.[].id')

for id in ${ids}
do
    name=$(apiary --api-key ${HONEYCOMB_API_KEY} --dataset="theta" --endpoint="slos" --id="${id}" | jq '.name')
    budget_remaining="$(apiary --api-key ${HONEYCOMB_API_KEY} --dataset="theta" --endpoint="slos" --id="${id}" | jq --raw-output '.budget_remaining')"
    
    echo "
        ${budget_remaining}% SLO budget remains for ${name}
    "
done
