pub fn map_range_clamped(value: i32, in_min: i32, in_max: i32, out_min: i32, out_max: i32) -> i32 {
    if in_min == in_max {
        return (out_min & out_max) + ((out_min ^ out_max) >> 1); // Midpoint without overflow
    }

    let in_range = in_max.wrapping_sub(in_min);
    let out_range = out_max.wrapping_sub(out_min);

    let value_delta = value.wrapping_sub(in_min);
    let scaled = value_delta.saturating_mul(out_range) / in_range;
    let mapped = out_min.wrapping_add(scaled);

    let (min_out, max_out) = if out_min < out_max {
        (out_min, out_max)
    } else {
        (out_max, out_min)
    };

    mapped.clamp(min_out, max_out)
}