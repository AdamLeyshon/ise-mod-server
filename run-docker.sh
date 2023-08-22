STACK_FILE="stack.yml"
DEPLOY_ENV="dev"

if [ "$1" = "prod" ]; then
  DEPLOY_ENV="prod"
fi

if [ "$2" = "proxy" ]; then
  STACK_FILE="stack-proxied.yml"
fi

docker-compose -f "./deploy/${STACK_FILE}" -f "./deploy/${DEPLOY_ENV}-${STACK_FILE}" -p "ise_${DEPLOY_ENV}" down
docker-compose -f "./deploy/${STACK_FILE}" -f "./deploy/${DEPLOY_ENV}-${STACK_FILE}" -p "ise_${DEPLOY_ENV}" up -d
