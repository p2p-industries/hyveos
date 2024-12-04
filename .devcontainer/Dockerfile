FROM ros:jazzy

RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y \
    build-essential \
    curl \
    git \
    libclang-dev \
    python3-pip \
    protobuf-compiler
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install cargo-ament-build
RUN pip install --break-system-packages git+https://github.com/colcon/colcon-cargo.git git+https://github.com/colcon/colcon-ros-cargo.git

WORKDIR /home/p2p-industries/ros

CMD ["/bin/bash"]
