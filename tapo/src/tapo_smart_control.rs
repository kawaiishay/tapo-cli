/// P306/P115 Example -- Shayna H. Jackson
use std::{thread, time::Duration};
use tapo::{ApiClient, Plug};
use argh::FromArgs;

pub type Result<T> = std::result::Result<T, anyhow::Error>;

#[derive(FromArgs)]
#[derive(Clone)]
#[argh(help_triggers("-h", "--help", "help"))]
/// all device information necessary to run commands
struct DeviceInfo {

    /// the tapo username
    #[argh(positional, short = 'u')]
    tapo_username: String,

    /// the tapo username
    #[argh(positional, short = 'p')]
    tapo_password: String,


    /// the device type [plug,strip]
    #[argh(positional, short = 't')]
    device_type: String,

    /// the ip of the device
    #[argh(positional, short = 'i')]
    device_ip: String,
    
    /// the command to run on the device
    #[argh(positional, short = 'c')]
    device_command: String,

    /// the device nickname in the tapo app
    #[argh(option, short = 'n')]
    device_nickname: Option<String>,
}

/* 
fn parse_args() -> Result<Args, anyhow::Error> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 || args.len() > 4 {
        eprintln!("Syntax: tapo_extension_strip <device_command> [on, off, reboot, info] <ip_address> <device_nickname>");
        std::process::exit(1);
    } else if args.len() == 4
    {
        Ok(Args {
            device_command: args[1].parse::<String>()?, device_ip: args[2].parse::<String>()?, device_nickname: args[3].parse::<String>()?,
        })
    }
    else {
        Ok(Args {
            device_command: "".to_string(), device_ip: args[2].parse::<String>()?, device_nickname: args[1].parse::<String>()?,
        })
    }
}
*/


async fn on(device: DeviceInfo) -> Result<()> {

    let (plug, strip) = get_device_info(device.clone()).await;

    match plug 
    {
        Some(found_plug) => 
        {
            println!("Turning device [with IP: {}] on...", device.clone().device_ip);
            found_plug.on().await?;
        }
        _ => {}
    }

    match strip 
    {
        Some(strip_plug) => 
        {
            println!("Turning {} device on...", device.clone().device_nickname.unwrap());
            strip_plug.on().await?;
        }
        _ => {}
    }

    Ok(())
}

async fn off(device: DeviceInfo) -> Result<()> {
   
    let (plug, strip) = get_device_info(device.clone()).await;

    match plug 
    {
        Some(found_plug) => 
        {
            println!("Turning device [with IP: {}] off...", device.clone().device_ip);
            found_plug.off().await?;
        }
        _ => {}
    }

    match strip 
    {
        Some(strip_plug) => 
        {
            println!("Turning {} device off...", device.clone().device_nickname.unwrap());
            strip_plug.off().await?;
        }
        _ => {}
    }

    Ok(())
}

async fn reboot(device: DeviceInfo) -> Result<()> {
   
    let (plug, strip) = get_device_info(device.clone()).await;

    match plug 
    {
        Some(found_plug) => 
        {
            println!("Rebooting device [with IP: {}]...", device.clone().device_ip);
            found_plug.off().await?;
            thread::sleep(Duration::from_secs(1));
            found_plug.on().await?;
        }
        _ => {}
    }

    match strip 
    {
        Some(strip_plug) => 
        {
            println!("Rebooting {} device...", device.clone().device_nickname.unwrap());
            strip_plug.off().await?;
            thread::sleep(Duration::from_secs(1));
            strip_plug.on().await?;
        }
        _ => {}
    }

    Ok(())
}

async fn info(device: DeviceInfo) -> Result<()> {
   
    let (plug, strip) = get_device_info(device.clone()).await;

    match plug 
    {
        Some(found_plug) => 
        {
            let device_info = found_plug.get_device_info().await?;
            println!("Device info: {device_info:?}", );
        }
        _ => {}
    }

    match strip 
    {
        Some(strip_plug) => 
        {
            let device_info = strip_plug.get_device_info().await?;
            println!("Device info: {device_info:?}", );
        }
        _ => {}
    }

    Ok(())
}

async fn get_device_plug(device: DeviceInfo) ->  std::result::Result<tapo::PlugEnergyMonitoringHandler, tapo::Error> {
    return ApiClient::new(device.tapo_username, device.tapo_password)
    .p115(device.device_ip)
    .await;
}

async fn get_device_strip(device: DeviceInfo) ->  std::result::Result<tapo::PowerStripPlugHandler, tapo::Error> {
    let power_strip = ApiClient::new(device.tapo_username, device.tapo_password)
        .p300(device.device_ip)
        .await.unwrap();
        
    // println!("Getting child devices...");
    let child_device_list = power_strip.get_child_device_list().await.unwrap();

    for child in child_device_list {
        if device.device_command == "info"
        {
            println!(
                "Found plug with nickname: {}, id: {}, state: {}.",
                child.nickname, child.device_id, child.device_on,
            );

            ()

        } else {
            if child.nickname == device.device_nickname.clone().unwrap()
            {
                return power_strip.plug(Plug::ByDeviceId(child.device_id)).await;
            }
        }
    }

    panic!("No matching outlet on strip with device nickname: {}", device.device_nickname.clone().unwrap());
}

async fn get_device_info(device: DeviceInfo) -> (Option<tapo::PlugEnergyMonitoringHandler>, Option<tapo::PowerStripPlugHandler>)   {

    return match device.device_type.as_str() 
    {
         "plug" => (Some(get_device_plug(device).await.unwrap()), None),
         "strip" => (None, Some(get_device_strip(device).await.unwrap())),
         _ =>  panic!("Invalid device type passed! -- {}", device.device_type)
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    // let args = parse_args()?;

    // println!("Device command: {}", args.device_command);
    // println!("Device nickname: {}", args.device_nickname);

    let device_information: DeviceInfo = argh::from_env();

    match device_information.device_command.as_str() {
        "on" => on(device_information).await,
        "off" => off(device_information).await,
        "reboot" => reboot(device_information).await,
        "info" => info(device_information).await,
        _ => panic!("Invalid command passed: {} -- use `tapo-cli --help`", device_information.device_command)
    }
}

