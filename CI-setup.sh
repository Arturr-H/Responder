## This file is called using ./CI-setup.sh, however
## if it doesn't work, do chmod 777 CI-setup.sh first

# Setup responder test server for CI
docker-compose build
docker-compose up -d

# Test
cargo test

# Stop responder test server
docker-compose down
