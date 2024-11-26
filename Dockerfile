FROM rust:slim-bookworm AS build

# Add wasm target
RUN rustup target add wasm32-unknown-unknown

# Install wasm-pack for testing
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install trunk
ADD https://github.com/thedodd/trunk/releases/download/v0.21.4/trunk-x86_64-unknown-linux-gnu.tar.gz ./tmp
RUN cd /tmp && tar xf trunk-x86_64-unknown-linux-gnu.tar.gz && chmod +x trunk && mv trunk /bin

WORKDIR /app
COPY . .

RUN trunk build --release

FROM nginx:1.21.5-alpine AS production

COPY --from=build /app/dist /usr/share/nginx/html
