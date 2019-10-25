Here what follows is terrible english and absolutely horrendous rust codes. You may copy, modify, redistribute modified or verbatim copies according to one or both of the licenses mentioned below.

# Moksha

This is an experimental video game written in rust. Building an editor of some sorts is the first priority; The actual game will follow. Keep note that this can be used as a library but is not intended as such since the code quality and feature set has a long way to go before being library worthy.

After many attempts and blunders, I have finally settled for wasm-bindgen. My first attempt was to try three-rs (<https://gitlab.com/nrayamajhee/moksha-three>). It was a good starting point, but I eventually learnt some webgl and GLSLS, which led me to abandon three-rs's three-js like abstraction. I then thought, I would instead write my own vulkan based renderer (<https://gitlab.com/nrayamajhee/moksha-vk>), which turned out to be an agonizing journery that was well beyond my capabilities. Hence, I am here, using wasm-bindgen's webgl binding. Hopefully someday WebGPU kicks off and drags me back to vulkan like code base. 

## How to?

### Setup:

To install rust. Follow the custom setup and install nightly because I use `maud` for templating which uses procedural macros.

```bash
curl https://sh.rustup.rs -sSf | sh
```

(Alternative) If you run Arch Linux, the following will do.

```bash
pacman -S rustup
rustup default nightly
```

To install wasm-pack:

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh 
```

### Run the game:
```bash
git clone https://gitlab.com/nrayamajhee/moksha.git
cd moksha
./moksha build
./mksha serve
```
### Run all tests:
```bash
wasm-pack test --headless --firefox
```

or

```bash
wasm-pack test --headless --chrome
```

### Generate docs:

```bash
cargo doc --open
```

## Preview 

The entire project is deployed at <http://moksha.rayamajhee.com>. This is continuously deployed with every commit in `master`, so expect it to be broken.

This was the initial setup for moksha-three. My goal is to eventually write abstractions (editors, shdaers, etc.) to be able to recreate this:

![screenshot](data/img/flight.png)

This is what the editor currently looks like:

![screenshot](data/img/editor.png)

## Rust-lang usages

Although, I have been learning rust for a while and was writing some C/C++ in the past, I am in no capability to judge how idiomatic my code is nor how healthy its memory management is. This program, although robust to my eyes does have some memory issues. I can't quite figure out if its my code, or firefox's WebGL driver, but I suspect there's some memory leaks as firefox's memory usage only climbs when run. Chrome, on the other hand, works perfectly fine.

Due to wasm-bingen's lack of support for rust lifetime annotations, I make heavy use of Rc<RefCell> to pass around the various structs to event handlers. Although, I can ditch wasm-bindgen endpoint and use lifetimes notations, I think it is better to expose all my structs and functions to javascript so that if anyone wants to use this library from javascript, it is as feature rich and complete.

# Done

- System
    - Storage to hold all the data
    - Reorganize VAOs into Storage

- Scene
    - Scene tree that allows node creations
    - Add primitive meshes
    
- Renderer
    - Rendering component that holds, compiles, and binds shaders
    - Unshaded Color
    - Vertext Color
    - Per pixel shading (Flat + Smooth)
    - Albedo  Map
    - Barycentric Wireframe
    - Lights (Ambient, Point, Directional, Spot)
	- Add menu to add mesh and lights
    
 - Editor
    - Custom logging screen
    - Node tree viewer
	- Implment open/collapse and render toggle to Node tree view
    - Zoom and Switch Perspective controls
    - Menu for adding objects

- Viewport
    - Perspective Projection
    - Orthographic Projection
    - Third Person Controls

# Doing

- Scene
  - load obj (geometry + albedo texure)

- Editor
  - Implement drag and drop for node parent/child relations.

- Renderer
  - Fix rendering order for depth

# ToDo
- System
    - Cap the framerate for performance.
    - Debug firefox's memory leaks.

- Scene
    - load obj
    - load gltf

- Renderer
    - Normal Map
    - Metallic/Roughness Map
    - Occlusion Map
    - Reflection and HDR Cubemaps
    - Volumetrics
    - Procedulal Texures (Cloud, Fbm, Perlin, Voronoi, etc.)
    - Fancy Wireframe (Points + Line + Depth Fade)
  
- Editor
    - Fancy mesh outline while selecting
    - Configuration Editor
    - Create a fps meter
    - Implement translation gizmo.
    - Implement rotational gizmo.
    - IMplement scaling gizmo.
    - Implement a pan guide gizmo. (This might need rendering on a separate framebuffer)
  
  
- Viewport
    - First Person Constrol
    - Allow camera animations
    
- Controller
    - Fly Navigation: accelerate, Deaccelerate, Roll, Pitch, and yaw movements
    - Walk Navigation: Acceleraete, Deaccelerate, Turn, Jump, Roll, Crouch, Crawl
    - Drive Navigation: Acceleraete, Deaccelerate, Turn

- World
    - Load milkyway skybox
    - Render Sky Model for Day/Night cycle
    - Displace the icosphere vertices with noise function
    - Implement level of detail for icosphere vertices.
    - Add plane model.
    - Volumetric Clouds
    - Instanced Trees

- Physics
    - Gravity
    - Collision with surfaces
    
## License

Licensed under either of the following terms at your choice:

  - Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
  - MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

Data contained in the repository (images, gltf, blend, etc.) are licensed under Creative Commons Attribution 4.0 International License (CC BY 4.0).
Refer to https://creativecommons.org/licenses/by/4.0/ for the details.

Please refer to the links at /data/credits for individual attributions for the blend file used to generate the skybox, and the deep star map from NASA.

Although, I doubt these are even relevant at this point, nontheless here they are. The entire rust files are licensed like rust-lang and other rust libs. For savage dummies who don't read legal stuffs, these simply mean if you use this source base, please kindly recognize my copyright. You may use it permissively (aka. non-copyleft). In other words, if you ever get dellusional and believe that you've somehow found gold in this repo and know a way to trade it for millions in petty human dollars, I don't give a shit about your efforts, nor do I demand your modifications back. A humble acknowledgement will suffice.

This is merely a past-time, an attempt at libration from this monotonous life, a dream from a mind full of darkness, a sink to all my erratic musing, and obviously incompetant scribbles. If you want to collaborate, or have questions, hit me up, or create issues, or send a pull request: do what you usually do. 
