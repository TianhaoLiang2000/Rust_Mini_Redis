use lazy_static::lazy_static;
use mini_redis::LogLayer;
use std::{net::SocketAddr, time::Instant};
use std::io::Write;
use std::io;

lazy_static! {
    static ref CLIENT: volo_gen::volo::example::ItemServiceClient = {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .layer_outer(LogLayer)
            .address(addr)
            .build()
    };
}
#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();

    loop {
        println!("Select an operation:");
        println!("1. Set Item");
        println!("2. Get Item");
        println!("3. Delete Item");
        println!("4. Ping");
        println!("5. Quit");

        let choice = read_user_choice();

        match choice {
            1 => {
                let id = read_user_input("Enter ID: ").parse().expect("Invalid ID");
                let title = read_user_input("Enter Title: ");
                let content = read_user_input("Enter Content: ");
                let _result = set_item(id,&title,&content).await;
            }
            2 => {
                let id = read_user_input("Enter ID: ").parse().expect("Invalid ID");
                let _result = get_item(id).await;
            }
            3 => {
                let id = read_user_input("Enter ID: ").parse().expect("Invalid ID");
                let _result = del_item(id).await;
            }
            4 => {
                // 执行 ping 操作
            // 在发送 ping 请求之前记录时间戳
            let send_time = Instant::now();
            // 执行 ping 操作，等待响应
            let resp4 = ping().await;
            // 在接收到 ping 响应后记录时间戳
            let receive_time = Instant::now();
            // 计算往返时间（Round-Trip Time）
            let rtt = receive_time.duration_since(send_time);
            match resp4 {
                Ok(info) => {
                    tracing::info!("{:?}", info);
                    // 打印往返时间
                    tracing::info!("Round-Trip Time (RTT): {:?}", rtt);
                }
                Err(e) => tracing::error!("{:?}", e),
            }
            }
            5 => {
                println!("Goodbye!");
                break; // 退出循环
            }
            _ => println!("Invalid choice, please select a valid operation."),
        }
    }
}

fn read_user_choice() -> u32 {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read user input");
    input.trim().parse().expect("Invalid choice")
}

fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush(); // 强制刷新输出
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read user input");
    input.trim().to_string()
}

async fn ping() -> Result<(), Box<dyn std::error::Error>> {
    let req4 = volo_gen::volo::example::PingRequest {};
    let _resp4 = CLIENT.ping(req4).await;
    Ok(())
}

async fn set_item(
    id: i64,
    title: &str,
    content: &str,
)  {
    let title_str = title.to_string();
    let content_str = content.to_string();
    let req = volo_gen::volo::example::SetItemRequest {
        id,
        title: title_str.into(),
        content: content_str.into(),
    };
    let resp = CLIENT.set_item(req).await;
    match resp {
        Ok(info) => tracing::info!("{:?}", info),
        Err(e) => tracing::error!("{:?}", e),
    }
}

async fn get_item(
    id: i64,
) {
    let req = volo_gen::volo::example::GetItemRequest {id,};
    let resp = CLIENT.get_item(req).await;

    match resp {
        Ok(info) => tracing::info!("{:?}", info),
        Err(e) => tracing::error!("{:?}", e),
    }

}

async fn del_item(
    id: i64,
) {
    let req = volo_gen::volo::example::DelItemRequest {id,};
    let resp = CLIENT.del_item(req).await;

    match resp {
        Ok(info) => tracing::info!("{:?}", info),
        Err(e) => tracing::error!("{:?}", e),
    }

}