<img align="left" alt="" src="vulkanus.gif" height="150"/>

# Vulkanus

[Vulkan](https://en.wikipedia.org/wiki/Vulkan)

[Vulcanus](https://en.wikipedia.org/wiki/Vulcan_(mythology))

[ASCII art](https://en.wikipedia.org/wiki/ASCII_art)

Vulkanus is a small pedagogical project which aims to bring Vulkan-based 3D rendering to the console.

## Setup

Vulkanus is written in Rust. We prefer to use Anaconda (specifically miniconda) for managing the build environment.

From the cloned repository:

    conda env create -f environment.yml
    conda activate vulkanus
    cargo run