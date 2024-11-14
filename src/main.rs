use rppal::i2c::I2c;
use std::thread::sleep;
use std::time::{Duration, Instant};

// ADS1115 default I2C address
const ADS1115_ADDR: u16 = 0x48;

// ADS1115 registers
const CONVERSION_REG: u8 = 0x00;
const CONFIG_REG: u8 = 0x01;

// ADS1115 configuration bits
const A0_SINGLE_ENDED: u16 = 0x4000;
const FSR_4_096V: u16 = 0x0200;
const MODE_SINGLE_SHOT: u16 = 0x0100;
const DATA_RATE_128SPS: u16 = 0x0080;
const OS_START_SINGLE: u16 = 0x8000;

// Parameters for BPM calculation
const SAMPLE_INTERVAL_MS: u64 = 10;
const BPM_CALCULATION_PERIOD: Duration = Duration::from_secs(5);
const THRESHOLD: f32 = 2.7;

fn main() {
    let mut i2c = I2c::new().expect("I2C Init");
    i2c.set_slave_address(ADS1115_ADDR)
        .expect("Set slave address");

    let mut beat_count = 0;
    let mut last_beat_time = Instant::now();
    let mut start_time = Instant::now();

    loop {
        let config =
            OS_START_SINGLE | A0_SINGLE_ENDED | FSR_4_096V | MODE_SINGLE_SHOT | DATA_RATE_128SPS;

        let config_bytes = config.to_be_bytes();
        i2c.write(&[CONFIG_REG, config_bytes[0], config_bytes[1]])
            .expect("Set config bits");

        sleep(Duration::from_millis(10));

        let mut buffer = [0; 2];
        i2c.write_read(&[CONVERSION_REG], &mut buffer)
            .expect("Read bytes to buffer");
        let value = i16::from_be_bytes(buffer);

        let voltage = value as f32 * 4.096 / 32768.0;

        if voltage > THRESHOLD && last_beat_time.elapsed() > Duration::from_millis(300) {
            beat_count += 1;
            last_beat_time = Instant::now();
            println!("Beat detected! Voltage: {:.3} V", voltage);
        }

        if start_time.elapsed() >= BPM_CALCULATION_PERIOD {
            let bpm = (beat_count as f32 / BPM_CALCULATION_PERIOD.as_secs_f32()) * 60.0;
            println!("BPM: {:.1}", bpm);

            beat_count = 0;
            last_beat_time = Instant::now();
            start_time = Instant::now();
        }

        sleep(Duration::from_millis(SAMPLE_INTERVAL_MS));
    }
}
