pub mod moon {

    // 2.551442882×10^6 seconds
    const AVG_SYNODIC_MONTH_SECS: u64 = 2_551_443;

    // https://aa.usno.navy.mil/calculated/moon/fraction?year=2025&task=00&tz=0.00&tz_sign=-1&tz_label=false&submit=Get+Data
    const REFERENCE_PHASE: f64 = 0.0;
    const REFERENCE_TIMESTAMP: u64 = 1763596800; // Nov 20, 2025 midnight UTC

    /// Gets the phase of the moon as a float [0, 1), representing the fraction of
    /// the way through the current lunar cycle from new to full to new again. Note
    /// that this is *not* the same as the fractional illumination.
    pub fn get_phase(timestamp: u64) -> f64 {
        let second_since_reference = timestamp - REFERENCE_TIMESTAMP;
        let remainder = second_since_reference % AVG_SYNODIC_MONTH_SECS;
        let phase_offset = (remainder as f64) / (AVG_SYNODIC_MONTH_SECS as f64);
        let mut phase = REFERENCE_PHASE + phase_offset;
        if phase >= 1.0 {
            phase -= 1.0;
        }
        phase
    }

    pub fn get_phase_label(phase: f64) -> &'static str {
        match phase {
            _ if phase <= 1.0 / 16.0 => "new moon",
            _ if phase <= 3.0 / 16.0 => "waxing crescent",
            _ if phase <= 5.0 / 16.0 => "first quarter",
            _ if phase <= 7.0 / 16.0 => "waxing gibbous",
            _ if phase <= 9.0 / 16.0 => "full moon",
            _ if phase <= 11.0 / 16.0 => "waning gibbous",
            _ if phase <= 13.0 / 16.0 => "last quarter",
            _ if phase <= 15.0 / 16.0 => "waning crescent",
            _ => "new moon",
        }
    }

    #[rustfmt::skip]
    const ILLUMINATION_TABLE: [f64; 31] = [
        0.0,
        0.0032,
        0.0213,
        0.0565,
        0.1075,
        0.1725,
        0.2499,
        0.3375,
        0.4331,
        0.5338,
        0.6362,
        0.7361,
        0.8280,
        0.9058,
        0.9631,
        0.9942,
        0.9954,
        0.9662,
        0.9093,
        0.8302,
        0.7354,
        0.6318,
        0.5254,
        0.4212,
        0.3233,
        0.2347,
        0.1578,
        0.0948,
        0.0472,
        0.0161,
        0.0024,
    ];

    /// Calculates the illumination percent from the moon's phase using a lookup
    /// table.
    pub fn get_illumination(phase: f64) -> f64 {
        let fractional_index = phase * ILLUMINATION_TABLE.len() as f64;

        let low_index = fractional_index as usize % ILLUMINATION_TABLE.len();
        let high_index = (low_index + 1) % ILLUMINATION_TABLE.len();

        let low_val = ILLUMINATION_TABLE[low_index];
        let high_val = ILLUMINATION_TABLE[high_index];
        let a = fractional_index - (low_index as f64);

        low_val * (1.0 - a) + high_val * a
    }
}
