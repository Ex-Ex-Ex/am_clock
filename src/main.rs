extern crate chrono;
extern crate clap;
extern crate crc8_rs;
extern crate serialport;

use chrono::{Datelike, FixedOffset, Local, TimeZone, Timelike};
use clap::Parser;
use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use std::process;

const PID: u16 = 598;
const VID: u16 = 1452;
const BAUD_RATE: u32 = 9600;
const TIMEOUT: u64 = 500;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "am_clock updates the time of a USB-connected Angry Miao Cyberboard to the local PC time"
)]
struct Args {
    /// Print status and error information (default is silent)
    #[arg(short = 'v')]
    verbose: bool,

    /// Use fake AM / PM mode (needs to be called at midday and midnight)
    #[arg(short = 'a')]
    am_pm: bool,
}

fn crc8_am(data: &[u8], initial_start: u8) -> u8 {
    let table: [u8; 256] = [
        0, 7, 14, 9, 28, 27, 18, 21, 56, 63, 54, 49, 36, 35, 42, 45, 112, 119, 126, 121, 108, 107,
        98, 101, 72, 79, 70, 65, 84, 83, 90, 93, 224, 231, 238, 233, 252, 251, 242, 245, 216, 223,
        214, 209, 196, 195, 202, 205, 144, 151, 158, 153, 140, 139, 130, 133, 168, 175, 166, 161,
        180, 179, 186, 189, 199, 192, 201, 206, 219, 220, 213, 210, 255, 248, 241, 246, 227, 228,
        237, 234, 183, 176, 185, 190, 171, 172, 165, 162, 143, 136, 129, 134, 147, 148, 157, 154,
        39, 32, 41, 46, 59, 60, 53, 50, 31, 24, 17, 22, 3, 4, 13, 10, 87, 80, 89, 94, 75, 76, 69,
        66, 111, 104, 97, 102, 115, 116, 125, 122, 137, 142, 135, 128, 149, 146, 155, 156, 177,
        182, 191, 184, 173, 170, 163, 164, 249, 254, 247, 240, 229, 226, 235, 236, 193, 198, 207,
        200, 221, 218, 211, 212, 105, 110, 103, 96, 117, 114, 123, 124, 81, 86, 95, 88, 77, 74, 67,
        68, 25, 30, 23, 16, 5, 2, 11, 12, 33, 38, 47, 40, 61, 58, 51, 52, 78, 73, 64, 71, 82, 85,
        92, 91, 118, 113, 120, 127, 106, 109, 100, 99, 62, 57, 48, 55, 34, 37, 44, 43, 6, 1, 8, 15,
        26, 29, 20, 19, 174, 169, 160, 167, 178, 181, 188, 187, 150, 145, 152, 159, 138, 141, 132,
        131, 222, 217, 208, 215, 194, 197, 204, 203, 230, 225, 232, 239, 250, 253, 244, 243,
    ];

    let mut sum = initial_start;

    if !data.is_empty() {
        for byte in data {
            sum = table[(sum ^ byte) as usize];
        }
    }

    sum
}

fn find_serial_port_with_pid_vid(pid: u16, vid: u16, verbose: bool) -> Option<Box<dyn SerialPort>> {
    let ports: Vec<SerialPortInfo> = serialport::available_ports().unwrap();

    for port in ports {
        if let SerialPortType::UsbPort(usb_info) = port.port_type {
            if usb_info.vid == vid && usb_info.pid == pid {
                //let port_result = serialport::new(port.port_name, BAUD_RATE).timeout(std::time::Duration::from_millis(TIMEOUT)).open();
                match serialport::new(port.port_name, BAUD_RATE)
                    .timeout(std::time::Duration::from_millis(TIMEOUT))
                    .open()
                {
                    Ok(serialport) => {
                        return Some(serialport);
                    }
                    Err(err) => {
                        if verbose {
                            eprintln!("Failed to open serial port: {}", err);
                        }
                    }
                };
            }
        }
    }

    None
}

fn get_message(am_pm: bool) -> [u8; 64] {
    let mut buff: [u8; 64] = [0; 64];
    buff[0] = 1;
    buff[1] = 3;

    let local_time = Local::now();
    let hour = if am_pm {
        match local_time.hour() {
            0 => 12,
            13..=23 => local_time.hour() - 12,
            _ => local_time.hour(),
        }
    } else {
        local_time.hour()
    };

    let fake_bejing_time = FixedOffset::east_opt(3600 * 8)
        .unwrap()
        .with_ymd_and_hms(
            local_time.year(),
            local_time.month(),
            local_time.day(),
            hour,
            local_time.minute(),
            local_time.second(),
        )
        .unwrap();
    let timestamp = fake_bejing_time.timestamp() as u32;

    buff[2] = ((timestamp >> 24) & 0xFF) as u8;
    buff[3] = ((timestamp >> 16) & 0xFF) as u8;
    buff[4] = ((timestamp >> 8) & 0xFF) as u8;
    buff[5] = (timestamp & 0xFF) as u8;

    buff[63] = crc8_am(&buff[0..63], 0);

    buff
}

fn main() {
    // Get command line parameters
    let args = Args::parse();

    // Find the serial port with the desired PID and VID
    let mut cyberboard_device;

    if let Some(device) = find_serial_port_with_pid_vid(PID, VID, args.verbose) {
        cyberboard_device = device;
        if args.verbose {
            println!(
                "Potential Cyberboard found: {}",
                cyberboard_device.name().unwrap()
            );
        }
    } else {
        if args.verbose {
            eprintln!("Cyberboard not found");
        }
        process::exit(1);
    }

    // Get the message to send
    let message = get_message(args.am_pm);

    // Send the message
    cyberboard_device.write_all(&message).unwrap();

    // Read the reply
    let mut serial_buf: Vec<u8> = vec![0; 64];
    match cyberboard_device.read(serial_buf.as_mut_slice()) {
        Ok(_) => {
            if args.verbose {
                println!("Reply received: {:?}", serial_buf);
            }
        }
        Err(err) => {
            if args.verbose {
                eprintln!("Error: {}", err);
            }
            process::exit(1);
        }
    }

    //.expect("Error: Board did not reply");

    if args.verbose {
        println!("Time updated successfully!");
    }
}
