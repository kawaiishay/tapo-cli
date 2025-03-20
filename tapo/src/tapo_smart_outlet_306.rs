/// P306 Example -- Shayna H. Jackson
use std::{env, thread, time::Duration};
use tapo::{ApiClient, Plug};


struct Args {
    device_command: String,
    device_nickname: String,
}

fn parse_args() -> Result<Args, anyhow::Error> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Syntax: tapo_306_strip <device_command> [on, off, reboot, info] <device_nickname>");
        std::process::exit(1);
    } else if args.len() == 3
    {
        Ok(Args {
            device_command: args[1].parse::<String>()?, device_nickname: args[2].parse::<String>()?,
        })
    }
    else {
        Ok(Args {
            device_command: "".to_string(), device_nickname: args[1].parse::<String>()?,
        })
    }
}


async fn on(plug: &tapo::PowerStripPlugHandler, plug_nickname: &String) -> Result<(), anyhow::Error> {
   
    println!("Turning {} device on...", plug_nickname);
    plug.on().await?;

    Ok(())
}

async fn off(plug: &tapo::PowerStripPlugHandler, plug_nickname: &String) -> Result<(), anyhow::Error> {
   
    println!("Turning {} device off...", plug_nickname);
    plug.off().await?;

    Ok(())
}

async fn reboot(plug: &tapo::PowerStripPlugHandler, plug_nickname: &String) -> Result<(), anyhow::Error> {
   
    off(plug, plug_nickname).await?;
    thread::sleep(Duration::from_secs(1));
    on(plug, plug_nickname).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    // println!("Device command: {}", args.device_command);
    // println!("Device nickname: {}", args.device_nickname);

    let tapo_username = env::var("TAPO_USERNAME")?;
    let tapo_password = env::var("TAPO_PASSWORD")?;
    let ip_address = env::var("IP_ADDRESS")?;

    let power_strip = ApiClient::new(tapo_username, tapo_password)
        .p300(ip_address)
        .await?;

    let device_info = power_strip.get_device_info().await?;
    if args.device_command == "info"
    {
        println!("Device info: {device_info:?}");
    }

    // println!("Getting child devices...");
    let child_device_list = power_strip.get_child_device_list().await?;

    for child in child_device_list {
        if args.device_command == "info"
        {
            println!(
                "Found plug with nickname: {}, id: {}, state: {}.",
                child.nickname, child.device_id, child.device_on,
            )
        }else {
            if child.nickname == args.device_nickname
            {
                let plug: tapo::PowerStripPlugHandler = power_strip.plug(Plug::ByDeviceId(child.device_id)).await?;
                let plug_nickname: &String = &child.nickname;

                match args.device_command.as_str() {
                    "on" => on(&plug, plug_nickname).await?,
                    "off" => off(&plug, plug_nickname).await?,
                    "reboot" => reboot(&plug, plug_nickname).await?,
                    _ => panic!("No valid commands found -- {} [on, off, reboot, info]", args.device_command),
                }
                break;
            }
        }
    }

    Ok(())
}
