<img align="right" alt="" src="vulkanus.gif"/>

# Vulkanus

[Vulkan](https://en.wikipedia.org/wiki/Vulkan)

[Vulcanus](https://en.wikipedia.org/wiki/Vulcan_(mythology))

[ASCII art](https://en.wikipedia.org/wiki/ASCII_art)

Vulkanus is a small pedagogical project which aims to bring Vulkan-based 3D rendering to the command line. It is currently in a minimum working example state.

## Setup

Vulkanus is written in Rust. We prefer to use Anaconda (specifically miniconda) for managing the build environment. You will also need [Microsoft Build Tools for Visual Studio (C++ desktop)](https://visualstudio.microsoft.com/downloads/) to compile a few Rust dependencies.

From the cloned repository:

    conda env create -f environment.yml
    conda activate vulkanus
    cargo run
