use mev::{
    DeviceDesc,
    DeviceType,
    Features,
    Instance,
};

pub fn main() {
    let instance = Instance::load().expect("Failed to init graphics");
    println!("Mev instance created successfully.");

    let features = Features::SURFACE;
    let mut selected_device = None;
    let mut selected_device_is_discrete = false;

    instance.capabilities().devices.iter().enumerate().for_each(|(i, device)| {
        let all_features_supported = device.features.contains(features);
        let device_is_discrete = device.device_type == DeviceType::DiscreteGpu;
        let device_name = &device.name;
        println!("Device {i}:");
        println!("  Name: {device_name}");
        println!("  All features supported: {all_features_supported}");
        println!("  Is a discrete GPU: {device_is_discrete}");
        if selected_device.is_none() && all_features_supported {
            selected_device = Some(i);
            selected_device_is_discrete = device_is_discrete;
        } else if all_features_supported && device_is_discrete && !selected_device_is_discrete {
            selected_device = Some(i);
            selected_device_is_discrete = device_is_discrete;
        }
    });
    let device_index = selected_device.expect("No suitable device found.");
    if selected_device_is_discrete {
        println!("Selected device id: {device_index} and it is a discrete GPU.");
    } else {
        println!("Selected device id: {device_index} and it is not a discrete GPU.");
    }

    let (device, mut queues) = instance
        .new_device(DeviceDesc {
            idx: device_index,
            queues: &[0],
            features,
        })
        .expect("Failed to create a device.");
    println!("Device {:?} created successfully.", device);

    let queue = queues.pop().expect("No queue found.");
    println!("Queue {:?} popped successfully", queue);

    // Here we can create resources on the device and do graphics work.

    queue.wait_idle().unwrap();
}