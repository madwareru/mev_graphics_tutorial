use mev::{AttachmentDesc, Buffer, BufferDesc, BufferUsage, ClearColor, DeviceDesc, DeviceType, Extent2, Features, Image, ImageDesc, ImageExtent, ImageUsage, Instance, Offset3, PipelineStages, PixelFormat, Queue, RenderPassDesc, Surface};
use mev_graphics_tutorial::software_buffer::{Color24, SoftwareBuffer};

const NICE_BLUE: ClearColor = ClearColor(0.39, 0.32, 0.81, 1.0);

pub fn main() {
    // Stage 0: Create a device and a queue.
    // Since queue is Deref<Target = Device>,
    // we don't need a device itself and all
    // its methods will be called through the queue.
    let mut queue = create_device_queue();

    // Stage 1: Create a render target image and a buffer
    // to which the image will be copied.
    let extent = Extent2::new(640, 480);
    let format = PixelFormat::Bgra8Unorm;
    let (render_target_image, mut dst_buffer) = create_render_target_and_dst_buffer(&mut queue, extent, format);

    // Stage 2: Create a fake surface to render to.
    // Effectively, we are just wrapping the image
    // to be able to render to it in the same manner
    // as it would be done with a real surface.
    let mut fake_surface = queue.new_fake_surface(render_target_image.clone()).expect("Failed to create fake surface");
    fake_surface.preferred_format(format);
    fake_surface.preferred_extent(extent);

    // Stage 3: Submit a frame to the queue.
    submit_frame(&mut queue, extent, &render_target_image, &mut dst_buffer, &mut fake_surface);

    // Stage 4: Read the buffer and print it as a ppm image.
    let mut soft_buffer = SoftwareBuffer::new(extent.width() as _, extent.height() as _);
    let mut flat_buffer = vec![0u8; extent.width() as usize * extent.height() as usize * 4];
    dst_buffer.read(0, &mut flat_buffer[..]).expect("Failed to read bytes from a buffer");
    for j in 0..extent.height() as usize {
        for i in 0..extent.width() as usize {
            let offset = (j * extent.width() as usize + i) * 4;
            let b = flat_buffer[offset + 0];
            let g = flat_buffer[offset + 1];
            let r = flat_buffer[offset + 2];
            // `a` is ignored
            soft_buffer.set_pixel(i as u16, j as u16, Color24 { r, g, b });
        }
    }
    soft_buffer.print_as_ppm();
}

fn submit_frame(queue: &mut Queue, extent: Extent2, render_target_image: &Image, dst_buffer: &mut Buffer, fake_surface: &mut Surface) {
    let mut frame = fake_surface.next_frame().expect("Failed to get next frame");

    queue.sync_frame(&mut frame, PipelineStages::COLOR_OUTPUT);
    let mut encoder = queue.new_command_encoder();
    encoder.init_image(PipelineStages::empty(), PipelineStages::COLOR_OUTPUT, frame.image());

    { // Just clear the image with a blue color.
        let _ = encoder.render(RenderPassDesc {
            name: "main",
            color_attachments: &[
                AttachmentDesc::new(frame.image()).clear(NICE_BLUE)
            ],
            depth_stencil_attachment: None,
        });
        encoder.present(frame, PipelineStages::COLOR_OUTPUT);
    }

    { // Copy the image to the buffer.
        let mut copy = encoder.copy();
        copy.barrier(PipelineStages::COLOR_OUTPUT, PipelineStages::TRANSFER);
        copy.copy_image_to_buffer(
            &render_target_image,
            Offset3::new(0, 0, 0),
            extent.to_3d(),
            0u32..1u32,
            0,
            &dst_buffer,
            0,
            extent.width() as usize * 4,
            extent.width() as usize * extent.height() as usize * 4,
        );
        copy.barrier(PipelineStages::TRANSFER, PipelineStages::empty());
    }
    let command_buffer = encoder.finish();

    queue.submit_checkpoint([command_buffer]).expect("Failed to submit checkpoint");

    queue.wait_idle().unwrap();
}

fn create_render_target_and_dst_buffer(
    queue: &mut Queue,
    extent: Extent2,
    format: PixelFormat
) -> (Image, Buffer) {
    let image_usage = ImageUsage::TARGET | ImageUsage::TRANSFER_SRC;
    let buffer_usage = BufferUsage::TRANSFER_DST | BufferUsage::HOST_READ;
    let buffer_size = extent.width() as usize * extent.height() as usize * 4;
    let image_extent = ImageExtent::D2(extent);
    (
        // Images used for the storage of textures.
        // They can be used as render targets,
        // depth-stencil attachments, or sampled textures.
        // There is also a possibility to submit a command
        // to copy part of an image to another image.
        queue.new_image(ImageDesc {
            extent: image_extent,
            format,
            layers: 1,
            levels: 1,
            usage: image_usage,
            name: "rt"
        }),
        // Buffers are universal storages for data on a GPU.
        // You can load and unload data from and into the CPU using buffers.
        // You can also transfer data between buffers and images.
        queue.new_buffer(BufferDesc {
            size: buffer_size,
            usage: buffer_usage,
            name: "dst",
        })
    )
}

fn create_device_queue() -> Queue {
    let instance = Instance::load().expect("Failed to init graphics");

    let features = Features::SURFACE;
    let mut selected_device = None;
    let mut selected_device_is_discrete = false;

    instance.capabilities().devices.iter().enumerate().for_each(|(i, device)| {
        let all_features_supported = device.features.contains(features);
        let device_is_discrete = device.device_type == DeviceType::DiscreteGpu;
        if selected_device.is_none() && all_features_supported {
            selected_device = Some(i);
            selected_device_is_discrete = device_is_discrete;
        } else if all_features_supported && device_is_discrete && !selected_device_is_discrete {
            selected_device = Some(i);
            selected_device_is_discrete = device_is_discrete;
        }
    });
    let device_index = selected_device.expect("No suitable device found.");

    let (_, mut queues) = instance
        .new_device(DeviceDesc {
            idx: device_index,
            queues: &[0],
            features,
        })
        .expect("Failed to create a device.");

    queues.pop().expect("No queue found.")
}