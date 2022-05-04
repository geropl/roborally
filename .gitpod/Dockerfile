FROM gitpod/workspace-full

ENV TRIGGER_UPDATE=3

USER gitpod

# Install gatsby CLI
RUN bash -lc "npm install -g gatsby-cli"

# Install rust 1.51
RUN bash -lc "unset CARGO_HOME && rustup install 1.51 && \
    rustup default 1.51 && \
    rustup component add \
        rustfmt \
        rls-preview \
        rust-analysis \
        rust-src \
        clippy"

USER root

# protoc (protobuf compiler)
RUN set -ex && \
    tmpdir=$(mktemp -d) && \
    curl -L -o $tmpdir/protoc.zip https://github.com/protocolbuffers/protobuf/releases/download/v3.7.1/protoc-3.7.1-linux-x86_64.zip && \
    mkdir -p /usr/lib/protoc && cd /usr/lib/protoc && unzip $tmpdir/protoc.zip && \
    chmod -R 755 /usr/lib/protoc/include/google && \
    ln -s /usr/lib/protoc/bin/* /usr/bin && \
    rm -rf $tmpdir

# grpc-web
RUN wget https://github.com/grpc/grpc-web/releases/download/1.0.6/protoc-gen-grpc-web-1.0.6-linux-x86_64 && \
    mv protoc-gen-grpc-web-1.0.6-linux-x86_64 /usr/local/bin/protoc-gen-grpc-web && \
    chmod +x /usr/local/bin/protoc-gen-grpc-web

# grpc-web-proxy (https://github.com/improbable-eng/grpc-web/tree/master/go/grpcwebproxy)
RUN cd $(mktemp -d) && \
    wget https://github.com/improbable-eng/grpc-web/releases/download/v0.11.0/grpcwebproxy-v0.11.0-linux-x86_64.zip -O grpcwebproxy.zip && \
    unzip grpcwebproxy.zip && \
    mv dist/grpcwebproxy-v0.11.0-linux-x86_64 /usr/local/bin/grpcwebproxy
