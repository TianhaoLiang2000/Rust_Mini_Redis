use lazy_static::lazy_static;
use mini_redis::LogLayer;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};
use std::{net::SocketAddr, time::Instant};

lazy_static! {
    static ref CLIENT: volo_gen::volo::example::ItemServiceClient = {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .layer_outer(LogLayer)
            .address(addr)
            .build()
    };
    static ref MY_AOF_FILE: &'static str = "/Users/a1234/Desktop/Mini_Redis/src/AOF_FILE";
}

/*
这个是AOF的Always策略实现
use anyhow::Context;
pub struct Aof{
    file: File,
}

impl Aof {
    fn new(file_path: &str) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file_path)
            .with_context(|| format!("Failed to open AOF file: {}", file_path))?;

        Ok(Aof { file })
    }

    fn append(&mut self, data: &str) -> anyhow::Result<()> {
        self.file
            .write_all(data.as_bytes())
            .with_context(|| "Failed to append data to AOF file")?;
        self.file.flush()?;
        Ok(())
    }
}
*/

//这个是everysec策略
struct Cache {
    data: Vec<String>,
}

impl Cache {
    fn new() -> Self {
        Cache { data: Vec::new() }
    }

    fn append(&mut self, command: &str) {
        self.data.push(command.to_string());
    }
}

#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cache = Arc::new(Mutex::new(Cache::new()));

    // 复制缓存的 Arc<Mutex<Cache>> 到硬盘线程
    let cache_for_disk = cache.clone();

    let mut aof_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("/Users/a1234/Desktop/Mini_Redis/src/AOF_FILE")
        .expect("Failed to open AOF file");
    // 启动硬盘写入线程
    thread::spawn(move || {
        let mut last_write_time = Instant::now();
        loop {
            // 检查是否距上次写入已经过去了 1 秒
            if last_write_time.elapsed() >= Duration::from_secs(1) {
                let mut cache = cache_for_disk.lock().unwrap();
                // 获取当前文件大小（文件末尾）
                let file_size = aof_file
                    .seek(SeekFrom::End(0))
                    .expect("Failed to get file size");
                aof_file
                    .seek(SeekFrom::Start(file_size))
                    .expect("Failed to seek to end of file");
                // 将缓存中的操作写入 AOF 文件
                for command in &cache.data {
                    write!(aof_file, "{}", command).expect("Failed to write to AOF file");
                }
                aof_file.flush().expect("Failed to flush file");
                // 清空缓存
                cache.data.clear();
                last_write_time = Instant::now();
            }
        }
    });
    loop {
        println!("Select an operation:");
        println!("1. Set Item");
        println!("2. Get Item");
        println!("3. Delete Item");
        println!("4. Ping");
        println!("5. Recover");
        println!("0. Quit");

        let choice = read_user_choice();

        match choice {
            1 => {
                let id = read_user_input("Enter ID: ").parse().expect("Invalid ID");
                let title = read_user_input("Enter Title: ");
                let content = read_user_input("Enter Content: ");
                let command = format!("SET {:} {:} {:}\n", id, title, content);
                let mut cache = cache.lock().unwrap();
                cache.append(&command);
                //let mut my_aof = Aof::new(&MY_AOF_FILE).unwrap();
                //let _ = my_aof.append(&format!("SET {:} {:} {:}\n" ,id, title, content));
                let _result = set_item(id, &title, &content).await;
            }
            2 => {
                let id = read_user_input("Enter ID: ").parse().expect("Invalid ID");
                let _result = get_item(id).await;
            }
            3 => {
                let id = read_user_input("Enter ID: ").parse().expect("Invalid ID");
                let command = format!("DEL {:} - -\n", id);
                let mut cache = cache.lock().unwrap();
                cache.append(&command);
                //let mut my_aof = Aof::new(&MY_AOF_FILE).unwrap();
                //let _ = my_aof.append(&format!("DEL {:} - -\n" ,id));
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
                // 打开AOF文件并读取命令
                let file = File::open("/Users/a1234/Desktop/Mini_Redis/src/AOF_FILE").unwrap();
                let reader = BufReader::new(file);

                for line in reader.lines() {
                    let line = line.unwrap();
                    let parts: Vec<&str> = line.trim().splitn(4, ' ').collect();

                    if parts.len() != 4 {
                        eprintln!("Invalid command: {}", line);
                        continue;
                    }

                    let command = parts[0];
                    let id: i64 = parts[1].parse::<i64>().unwrap();
                    let title = parts[2];
                    let content = parts[2];

                    match command {
                        "SET" => set_item(id, &title, &content).await,
                        "DEL" => del_item(id).await,
                        _ => {
                            eprintln!("Unknown command: {}", command);
                        }
                    }
                }
            }
            0 => {
                println!("Goodbye!");
                break; // 退出循环
            }
            _ => println!("Invalid choice, please select a valid operation."),
        }
    }
}

fn read_user_choice() -> u32 {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input");
    input.trim().parse().expect("Invalid choice")
}

fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush(); // 强制刷新输出
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input");
    input.trim().to_string()
}

async fn ping() -> Result<(), Box<dyn std::error::Error>> {
    let req4 = volo_gen::volo::example::PingRequest {};
    let _resp4 = CLIENT.ping(req4).await;
    Ok(())
}

async fn set_item(id: i64, title: &str, content: &str) {
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

async fn get_item(id: i64) {
    let req = volo_gen::volo::example::GetItemRequest { id };
    let resp = CLIENT.get_item(req).await;

    match resp {
        Ok(info) => tracing::info!("{:?}", info),
        Err(e) => tracing::error!("{:?}", e),
    }
}

async fn del_item(id: i64) {
    let req = volo_gen::volo::example::DelItemRequest { id };
    let resp = CLIENT.del_item(req).await;

    match resp {
        Ok(info) => tracing::info!("{:?}", info),
        Err(e) => tracing::error!("{:?}", e),
    }
}
