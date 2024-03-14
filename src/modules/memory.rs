use super::custom::new_command;
use std::error::Error;

#[derive(Copy, Clone, Debug)]
pub enum RamOpts {
    Used,
    Free,
    PercUsed,
    PercFree,
}

pub fn memory_usage(opt: RamOpts) -> Result<String, Box<dyn Error>> {
    let output = new_command("free", "-m")?;
    let output = output.split_whitespace().collect::<Vec<&str>>();
    let total = output[7].parse::<f64>()?;
    let used = output[8].parse::<f64>()?;

    let output = match opt {
        RamOpts::PercUsed => (used / total) * 100.0,
        RamOpts::PercFree => ((total - used) / total) * 100.0,
        RamOpts::Used => used,
        RamOpts::Free => total - used,
    }
    .to_string()
    .split('.')
    .next()
    .ok_or("")?
    .into();

    Ok(output)
}
