use lazy_static::lazy_static;
use mini_redis::LogLayer;
use std::{net::SocketAddr, time::Instant};
use clap::{App, Arg};

lazy_static! {
    static ref CLIENT: volo_gen::volo::example::ItemServiceClient = {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .layer_outer(LogLayer)
            .address(addr)
            .build()
    };
}

// 定义一个模拟的FastStr类型
#[derive(Debug)]
struct FastStrWrapper<'a>(&'a str);



#[volo::main]



async fn main() {
    tracing_subscriber::fmt::init();

    let matches = App::new("Your Client CLI")
        .version("1.0")
        .author("Your Name")
        .about("A client CLI for your service")
        .subcommand(App::new("set_item")
            .about("Set an item")
            .arg(Arg::with_name("id")
                .long("id")
                .takes_value(true)
                .required(true)
                )
            .arg(Arg::with_name("title")
                .long("title")
                .takes_value(true)
                .required(true)
                )
            .arg(Arg::with_name("content")
                .long("content")
                .takes_value(true)
                .required(true)
                ))
        .subcommand(App::new("get_item")
            .about("Get an item")
            .arg(Arg::with_name("id")
                .long("id")
                .takes_value(true)
                .required(true)
                ))
        .subcommand(App::new("del_item")
            .about("Delete an item")
            .arg(Arg::with_name("id")
                .long("id")
                .takes_value(true)
                .required(true)
                ))
        .subcommand(App::new("ping")
            .about("Ping the server"))
        .get_matches();

    match matches.subcommand() {
        ("set_item", Some(sub_m)) => {
            let id = sub_m.value_of("id").unwrap().parse::<i64>().unwrap();
            let title = sub_m.value_of("title").unwrap();
            let content = sub_m.value_of("content").unwrap();

            // 调用 set_item 函数并传递 FastStrWrapper 引用
            let _result = set_item(id,title,content).await;
        }
        ("get_item", Some(sub_m)) => {
            let id = sub_m.value_of("id").unwrap().parse::<i64>().unwrap();
            // 执行 get_item 操作，传递 id 到相应的函数
            // 示例：get_item(id).await;
            let _result = get_item(id).await;
        }
        ("del_item", Some(sub_m)) => {
            let id = sub_m.value_of("id").unwrap().parse::<i64>().unwrap();
            // 执行 del_item 操作，传递 id 到相应的函数
            // 示例：del_item(id).await;
            let _result = del_item(id).await;
        }
        ("ping", _) => {
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
        _ => {}
    }
}

async fn ping() -> Result<(), Box<dyn std::error::Error>> {
    let req4 = volo_gen::volo::example::PingRequest {};
    let _resp4 = CLIENT.ping(req4).await;
    Ok(())
}
// 定义一个模拟的 set_item 函数

async fn set_item<'a>(
    id: i64,
    title: &'a str,
    content: &'a str,
)  {
    // Convert title and content into owned String values
    let title_str = title.to_string();
    let content_str = content.to_string();

    // Create the SetItemRequest using owned String values
    let req = volo_gen::volo::example::SetItemRequest {
        id,
        title: title_str.into(),
        content: content_str.into(),
    };

    // Rest of your code...
    let resp = CLIENT.set_item(req).await;
    match resp {
        Ok(info) => tracing::info!("{:?}", info),
        Err(e) => tracing::error!("{:?}", e),
    }
}

async fn get_item<'a>(
    id: i64,
) {
    let req = volo_gen::volo::example::GetItemRequest {id,};
    // Rest of your code...
    let resp = CLIENT.get_item(req).await;

    match resp {
        Ok(info) => tracing::info!("{:?}", info),
        Err(e) => tracing::error!("{:?}", e),
    }

}

async fn del_item<'a>(
    id: i64,
) {
    let req = volo_gen::volo::example::DelItemRequest {id,};
    // Rest of your code...
    let resp = CLIENT.del_item(req).await;

    match resp {
        Ok(info) => tracing::info!("{:?}", info),
        Err(e) => tracing::error!("{:?}", e),
    }

}