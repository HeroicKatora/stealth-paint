use stealth_paint::buffer::{Descriptor, Whitepoint};
use stealth_paint::command::{self, CommandBuffer, Rectangle};
use stealth_paint::pool::{Pool, PoolKey};

#[path = "util.rs"]
mod util;

const BACKGROUND: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/input/background.png");
const FOREGROUND: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/input/foreground.png");

/// Integration test the whole pipeline against reference images.
#[test]
fn integration() {
    const ANY: wgpu::BackendBit = wgpu::BackendBit::all();
    // FIXME: this drop SEGFAULTs for me...
    let instance = core::mem::ManuallyDrop::new(wgpu::Instance::new(ANY));

    let background = image::open(BACKGROUND).expect("Background image opened");
    let foreground = image::open(FOREGROUND).expect("Background image opened");

    let mut pool = Pool::new();
    let pool_background = {
        let entry = pool.insert_srgb(&background);
        (entry.key(), entry.descriptor())
    };

    let pool_foreground = {
        let entry = pool.insert_srgb(&foreground);
        (entry.key(), entry.descriptor())
    };

    run_blending(
        &mut pool,
        instance.enumerate_adapters(ANY),
        pool_foreground.clone(),
        pool_background.clone(),
    );

    run_affine(
        &mut pool,
        instance.enumerate_adapters(ANY),
        pool_foreground.clone(),
        pool_background.clone(),
    );

    run_adaptation(
        &mut pool,
        instance.enumerate_adapters(ANY),
        pool_background.clone(),
    );
}

fn run_blending(
    pool: &mut Pool,
    adapters: impl Iterator<Item=wgpu::Adapter>,
    (fg_key, foreground): (PoolKey, Descriptor),
    (bg_key, background): (PoolKey, Descriptor),
) {
    let mut commands = CommandBuffer::default();

    let placement = Rectangle {
        x: 0,
        y: 0,
        max_x: foreground.layout.width(),
        max_y: foreground.layout.height(),
    };

    // Describe the pipeline:
    // 0: in (background)
    // 1: in (foreground)
    // 2: inscribe(0, placement, 1)
    // 3: out(2)
    let background = commands.input(background).unwrap();
    let foreground = commands.input(foreground).unwrap();

    let result = commands
        .inscribe(background, placement, foreground)
        .expect("Valid to inscribe");

    let (output, _outformat) = commands.output(result).expect("Valid for output");

    let plan = commands.compile().expect("Could build command buffer");
    let adapter = plan
        .choose_adapter(adapters)
        .expect("Did not find any adapter for executing the blend operation");

    let mut execution = plan
        .launch(pool)
        .bind(background, bg_key)
        .unwrap()
        .bind(foreground, fg_key)
        .unwrap()
        .launch(&adapter)
        .expect("Launching failed");

    while execution.is_running() {
        let _wait_point = execution.step().expect("Shouldn't fail but");
    }

    let mut retire = execution.retire_gracefully(pool);

    let image = retire
        .output(output)
        .expect("A valid image output");
    util::assert_reference(image, "composed.png.crc");
}

fn run_affine(
    pool: &mut Pool,
    adapters: impl Iterator<Item=wgpu::Adapter>,
    (fg_key, foreground): (PoolKey, Descriptor),
    (bg_key, background): (PoolKey, Descriptor),
) {
    let mut commands = CommandBuffer::default();

    let affine = command::Affine::new(command::AffineSample::Nearest)
        // Move the foreground center to origin.
        .shift(
            -((foreground.layout.width() / 2) as f32),
            -((foreground.layout.height() / 2) as f32),
        )
        // Rotate 45°
        .rotate(3.145159265 / 4.)
        // Move origin to the background center.
        .shift(
            (background.layout.width() / 2) as f32,
            (background.layout.height() / 2) as f32,
        );

    // Describe the pipeline:
    // 0: in (background)
    // 1: in (foreground)
    // 2: affine(0, affine, 1)
    // 3: out(2)
    let background = commands.input(background).unwrap();
    let foreground = commands.input(foreground).unwrap();

    let result_affine = commands
        .affine(background, affine, foreground)
        .expect("Valid to paint with affine transformation");

    let (output_affine, _outformat) = commands.output(result_affine).expect("Valid for output");

    let plan = commands.compile().expect("Could build command buffer");
    let adapter = plan
        .choose_adapter(adapters)
        .expect("Did not find any adapter for executing the blend operation");

    let mut execution = plan
        .launch(pool)
        .bind(background, bg_key)
        .unwrap()
        .bind(foreground, fg_key)
        .unwrap()
        .launch(&adapter)
        .expect("Launching failed");

    while execution.is_running() {
        let _wait_point = execution.step().expect("Shouldn't fail but");
    }

    let mut retire = execution.retire_gracefully(pool);

    let image_affine = retire
        .output(output_affine)
        .expect("A valid image output");
    util::assert_reference(image_affine, "affine.png.crc");
}

fn run_adaptation(
    pool: &mut Pool,
    adapters: impl Iterator<Item=wgpu::Adapter>,
    (bg_key, background): (PoolKey, Descriptor),
) {
    let mut commands = CommandBuffer::default();

    // Describe the pipeline:
    // 0: in (background)
    // 1: chromatic_adaptation(0, adapt)
    // 2: out(2)
    let background = commands.input(background).unwrap();

    let adapted = commands
        .chromatic_adaptation(
            background,
            command::ChromaticAdaptationMethod::VonKries,
            Whitepoint::D50,
        )
        .unwrap();

    let (output_affine, _outformat) = commands.output(adapted).expect("Valid for output");

    let plan = commands.compile().expect("Could build command buffer");
    let adapter = plan
        .choose_adapter(adapters)
        .expect("Did not find any adapter for executing the blend operation");

    let mut execution = plan
        .launch(pool)
        .bind(background, bg_key)
        .unwrap()
        .launch(&adapter)
        .expect("Launching failed");

    while execution.is_running() {
        let _wait_point = execution.step().expect("Shouldn't fail but");
    }

    let mut retire = execution.retire_gracefully(pool);

    let image_adapted = retire
        .output(output_affine)
        .expect("A valid image output");
    util::assert_reference(image_adapted, "adapted.png.crc");
}
