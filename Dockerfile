FROM rust:latest as build

RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-pack

WORKDIR /mandelbrot-wasm-app
COPY . . 

RUN wasm-pack build --target web

FROM nginx:latest

COPY ./nginx.conf /etc/nginx/nginx.conf
COPY --from=build /mandelbrot-wasm-app/pkg /usr/share/nginx/html/pkg
COPY ./static /usr/share/nginx/html