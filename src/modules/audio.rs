use pulsectl::controllers::{DeviceControl, SinkController};

pub fn audio() -> Result<String, Box<dyn crate::Error>> {
    let mut handler = SinkController::create()?;

    let a = handler.get_default_device()?.index;
    let devices = handler.list_devices()?;
    Ok(devices
        .iter()
        .find_map(|dev| {
            if dev.index == a {
                let b = dev.volume.print();
                let c = b.split_whitespace().collect::<Vec<_>>()[1].replace('%', "");
                return Some(c);
            }

            None
        })
        .ok_or("")?)
}
