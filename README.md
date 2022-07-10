# vulkanus

Vulkanus is a small pedagogical project which aims to bring [Vulkan](https://en.wikipedia.org/wiki/Vulkan)-based 3D rendering to the console.

## Setup

Vulkanus is written in Rust. We prefer to use Anaconda (specifically miniconda) for managing the build environment.

From the cloned repository:

    conda env create -f environment.yml
    conda activate vulkanus
    cargo run