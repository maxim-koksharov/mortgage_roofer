FROM almalinux:9

RUN dnf install -y --allowerasing \
    curl \
    git \
    procps-ng \
    gcc \
    g++ \
    make \
    pkgconfig \
    openssl-devel \
    fontconfig-devel \
    freetype-devel \
    python3 \
    vim-common \
    && dnf clean all

RUN dnf install -y epel-release && dnf install -y ripgrep && dnf clean all

# Install gosu for dropping privileges at runtime
RUN curl -o /usr/local/bin/gosu -fsSL "https://github.com/tianon/gosu/releases/download/1.17/gosu-amd64" \
    && chmod +x /usr/local/bin/gosu \
    && gosu nobody true

# Install Rust system-wide so the non-root container user can use it
ENV CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH="/usr/local/cargo/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --no-modify-path \
    && rustup component add rust-analyzer

# Cache Rust dependencies with a dummy build
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY mortgage_core/Cargo.toml mortgage_core/Cargo.toml
COPY mortgage_cli/Cargo.toml mortgage_cli/Cargo.toml
COPY mortgage_tui/Cargo.toml mortgage_tui/Cargo.toml
COPY mortgage_gui/Cargo.toml mortgage_gui/Cargo.toml
RUN mkdir -p mortgage_core/src mortgage_cli/src mortgage_tui/src mortgage_gui/src \
    && echo "" > mortgage_core/src/lib.rs \
    && echo "fn main() {}" > mortgage_cli/src/main.rs \
    && echo "fn main() {}" > mortgage_tui/src/main.rs \
    && echo "fn main() {}" > mortgage_gui/src/main.rs
RUN cargo build --release 2>&1
RUN rm -rf /build

# Install opencode globally
RUN curl -fsSL https://opencode.ai/install | bash \
    && mv /root/.opencode/bin/opencode /usr/local/bin/opencode \
    && chmod +x /usr/local/bin/opencode \
    && rm -rf /root/.opencode

COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

WORKDIR /workspace

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["/bin/bash"]
