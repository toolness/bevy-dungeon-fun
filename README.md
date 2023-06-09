In May 2023, I started taking the [Complete Blender Creator 3.2: Learn 3D Modelling for Beginners](https://www.gamedev.tv/p/complete-blender-creator-3-2-learn-3d-modelling-for-beginners) course on GameDev.tv and I decided to use its Modular Dungeon project as an opportunity to learn Bevy.

This project is the result of that experiment.

## Quick start

First, you will need to load `assets/dungeon.blend` in Blender 3.5 and export the scene as glTF.  The settings should already be configured for you.

If you don't have Blender, you can try using the files from the `gh-pages` branch of this repository, though they might be out of date compared to the project on the `main` branch.

You will also need Rust.

### Desktop

Just run the `run` script.

### Web

First install the dependency:

```
cargo install wasm-server-runner
```

Then run the `webrun` script.

## Distribution

Install the dependency:

```
cargo install wasm-bindgen-cli
```

Then run the `webdist.sh` script.

Then you can either:

1. Upload the `dist` directory to a webserver, or
2. Run `npm run deploy` to upload it to GitHub pages.

## License

Everything in this repository, including the Rust code and the Blender project, is licensed under [CC0 1.0 Universal](./LICENSE.md) (public domain).

## Credits

Portions of the first-person-controller code were taken from [bevy_flycam](https://github.com/sburris0/bevy_flycam).

Some snippets were provided by GitHub Copilot.

The rest of the Rust code, and the entire Blender scene, are by Atul Varma.
