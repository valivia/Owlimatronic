pub fn map(x: u16, in_min: u16, in_max: u16, out_min: u16, out_max: u16) -> u16 {
    let x = x as i32;
    let in_min = in_min as i32;
    let in_max = in_max as i32;
    let out_min = out_min as i32;
    let out_max = out_max as i32;

    if in_min == in_max {
        return out_min as u16; // Avoid division by zero
    }

    let result = (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
    result.clamp(u16::MIN as i32, u16::MAX as i32) as u16
}