FROM rust:bookworm

# Create a non-root user
RUN useradd -ms /bin/bash msfuser

# Install necessary tools and dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    build-essential \
    git \
    python3 \
    apt-transport-https \
    ca-certificates

# Install additional dependencies
RUN apt-get install -y \
    libpcap-dev \
    libssl-dev \
    zlib1g-dev \
    ruby-full \
    arp-scan \
    nmap \
    dnsrecon \
    postgresql postgresql-contrib \
    dnsutils \
    iputils-ping \
    libutempter-dev \
    cmake

# Install Metasploit
RUN curl https://raw.githubusercontent.com/rapid7/metasploit-omnibus/master/config/templates/metasploit-framework-wrappers/msfupdate.erb > /tmp/msfinstall && \
    chmod 755 /tmp/msfinstall && \
    /tmp/msfinstall

# Add Metasploit binaries to the PATH
ENV PATH=$PATH:/opt/metasploit-framework/bin

# Set working directory
WORKDIR /msf
RUN cargo install cargo-watch

COPY . .

# Ensure root ownership for certain files (if needed)
RUN chown -R root:root /msf

# Switch back to non-root user for security (optional)
USER msfuser

# Initialize the Metasploit Database
RUN msfdb init && msfdb start
