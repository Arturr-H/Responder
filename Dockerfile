FROM --platform=linux/amd64 rust:1.66.0

# Install dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev

# Create a new project
WORKDIR /usr/src/app

# Copy deps (responder)
COPY . /usr/src/app
EXPOSE 6102

# Build the project
RUN cargo build --release --lib

# Run the project (not responder, we only start the test-server therefore change workdir)
WORKDIR /usr/src/app/test-server
CMD ["cargo", "run", "--release"]
