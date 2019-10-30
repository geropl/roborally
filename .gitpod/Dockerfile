FROM gitpod/workspace-full

USER gitpod

# Install gatsby CLI
RUN bash -lc "npm install -g gatsby-cli"

# Install rust beta
RUN bash -lc "rustup install beta && \
    rustup default beta && \
    rustup component add rustfmt"
