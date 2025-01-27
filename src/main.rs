use clap::{arg, Args, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use nvml_wrapper::Nvml;
use nvml_wrapper_sys::bindings::{nvmlDevice_t, NvmlLib};
use serde::Deserialize;
use std::{collections::HashMap, io};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Path to the config file
    #[arg(short, long, default_value = "/etc/nvidia_oc.json")]
    file: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Sets GPU parameters like frequency offset and power limit
    Set {
        /// GPU index
        #[arg(short, long)]
        index: u32,

        #[command(flatten)]
        sets: Sets,
    },
    /// Generate shell completion script
    Completion {
        /// The shell to generate the script for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Args, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[group(required = true, multiple = true)]
struct Sets {
    /// GPU frequency offset
    #[arg(short, long, allow_hyphen_values = true)]
    freq_offset: Option<i32>,
    /// GPU memory frequency offset
    #[arg(long = "mem-offset", allow_hyphen_values = true)]
    mem_offset: Option<i32>,
    /// GPU power limit in milliwatts
    #[arg(short, long)]
    power_limit: Option<u32>,
    /// GPU min clock
    #[arg(long = "min-clock")]
    min_clock: Option<u32>,
    /// GPU max clock
    #[arg(long = "max-clock")]
    max_clock: Option<u32>,
}

impl Sets {
    fn apply(&self, nvml: &NvmlLib, device: nvmlDevice_t) {
        if let Some(freq_offset) = self.freq_offset {
            set_gpu_frequency_offset(&nvml, device, freq_offset)
                .expect("Failed to set GPU frequency offset");
        }

        if let Some(mem_offset) = self.mem_offset {
            set_gpu_memory_frequency_offset(&nvml, device, mem_offset)
                .expect("Failed to set GPU memory frequency offset");
        }

        if let Some(limit) = self.power_limit {
            set_gpu_power_limit(&nvml, device, limit).expect("Failed to set GPU power limit");
        }

        if let (Some(min_clock), Some(max_clock)) = (self.min_clock, self.max_clock) {
            set_gpu_min_max_clock(&nvml, device, min_clock, max_clock)
                .expect("Failed to set GPU min and max clocks");
        }
    }
}

#[derive(Deserialize)]
struct Config {
    sets: HashMap<u32, Sets>,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Set { index, sets }) => {
            escalate_permissions().expect("Failed to escalate permissions");

            sudo2::escalate_if_needed()
                .or_else(|_| sudo2::doas())
                .or_else(|_| sudo2::pkexec())
                .expect("Failed to escalate privileges");

            let nvml = Nvml::init().expect("Failed to initialize NVML");

            let device = nvml.device_by_index(*index).expect("Failed to get GPU");

            unsafe {
                let raw_device_handle: nvmlDevice_t = device.handle();
                let nvml_lib =
                    NvmlLib::new("libnvidia-ml.so").expect("Failed to load NVML library");

                sets.apply(&nvml_lib, raw_device_handle);
            }
            println!("Successfully set GPU parameters.");
        }
        None => {
            let Ok(config_file) = std::fs::read_to_string(cli.file) else {
                panic!("Configuration file not found and no valid arguments were provided. Run `nvidia_oc --help` for more information.");
            };

            escalate_permissions().expect("Failed to escalate permissions");

            let config: Config =
                serde_json::from_str(&config_file).expect("Invalid configuration file");

            let nvml = Nvml::init().expect("Failed to initialize NVML");

            unsafe {
                let nvml_lib =
                    NvmlLib::new("libnvidia-ml.so").expect("Failed to load NVML library");

                for (index, sets) in config.sets {
                    let device = nvml.device_by_index(index).expect("Failed to get GPU");
                    sets.apply(&nvml_lib, device.handle());
                }
            }
            println!("Successfully set GPU parameters.");
        }
        Some(Commands::Completion { shell }) => {
            generate_completion_script(*shell);
        }
    }
}

fn escalate_permissions() -> Result<(), Box<dyn std::error::Error>> {
    if sudo2::running_as_root() {
        return Ok(());
    }

    if which::which("sudo").is_ok() {
        sudo2::escalate_if_needed()?;
    } else if which::which("doas").is_ok() {
        sudo2::doas()?;
    } else if which::which("pkexec").is_ok() {
        sudo2::pkexec()?;
    } else {
        return Err("Please install sudo, doas or pkexec and try again. Alternatively, run the program as root.".into());
    }

    Ok(())
}

fn set_gpu_frequency_offset(
    nvml_lib: &NvmlLib,
    handle: nvmlDevice_t,
    offset: i32,
) -> Result<(), String> {
    let result = unsafe { nvml_lib.nvmlDeviceSetGpcClkVfOffset(handle, offset) };
    if result != 0 {
        Err(format!("Error code: {}", result))
    } else {
        Ok(())
    }
}

fn set_gpu_memory_frequency_offset(
    nvml_lib: &NvmlLib,
    handle: nvmlDevice_t,
    offset: i32,
) -> Result<(), String> {
    let result = unsafe { nvml_lib.nvmlDeviceSetMemClkVfOffset(handle, offset) };
    if result != 0 {
        Err(format!("Error code: {}", result))
    } else {
        Ok(())
    }
}

fn set_gpu_power_limit(nvml_lib: &NvmlLib, handle: nvmlDevice_t, limit: u32) -> Result<(), String> {
    let result = unsafe { nvml_lib.nvmlDeviceSetPowerManagementLimit(handle, limit) };
    if result != 0 {
        Err(format!("Error code: {}", result))
    } else {
        Ok(())
    }
}

fn set_gpu_min_max_clock(
    nvml_lib: &NvmlLib,
    handle: nvmlDevice_t,
    minclock: u32,
    maxclock: u32,
) -> Result<(), String> {
    let result = unsafe { nvml_lib.nvmlDeviceSetGpuLockedClocks(handle, minclock, maxclock) };
    if result != 0 {
        Err(format!("Error code: {}", result))
    } else {
        Ok(())
    }
}

fn generate_completion_script<G: Generator>(gen: G) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(gen, &mut cmd, name, &mut io::stdout());
}
