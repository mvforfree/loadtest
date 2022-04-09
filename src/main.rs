use clap::Parser;
use tokio;
use std::time::Duration;

use futures;
use futures::StreamExt;
use reqwest::{Client, Error, Response};

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::string::String;
use reqwest::header::HeaderMap;
use rand::{Rng, thread_rng};

#[derive(Debug, Clone)]
struct RevRequest {
    headers: String,
    request: String,
    req_type: String,
}


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    url: String,

    #[clap(short, long, default_value_t = 20)]
    threads: usize,

    #[clap(short, long, default_value_t = 5)]
    connection_timeout: u64,

    #[clap(short, long, default_value = "")]
    headers: String,

    #[clap(short, long, default_value = "")]
    request: String,

    #[clap(short, long, default_value = "GET")]
    req_type: String,

    #[clap(short, long, default_value = "")]
    http_proxies_file: String,

    #[clap(short, long, default_value = "")]
    socks4_proxies_file: String,

    #[clap(short, long, default_value = "")]
    socks5_proxies_file: String,

    #[clap(short, long)]
    verbose: bool,

    #[clap(short, long)]
    replace: bool,
}

impl RevRequest {
    fn new(headers: String, request: String, generate_rand: bool, req_type: String) -> RevRequest {
        let mut req_body = request;
        if generate_rand {
            req_body = RevRequest::replace(req_body);
        }
        let req = RevRequest {headers, request: req_body, req_type };
        req
    }

    fn replace(request_prepared: String) -> String {

        let names = ["Adrian", "Akim", "Alek", "Alexei", "Andrey", "Anatoly", "Arkadi", "Armen", "Artyom", "Boris", "Damien", "Danyl", "Denis", "Dima", "Dimitri", "Eriks", "Hedeon", "Gennady", "Igor", "Ilias", "Ioann", "Ivan", "Jeremie", "Karlin", "Kirill", "Konstantin", "Lev", "Leonid", "Ludis", "Maxim", "Mikhail", "Nikolai", "Oleg", "Olezka", "Pavel", "Rodion", "Rolan", "Ruslan", "Sacha", "Sergei", "Simeon", "Tima", "Vas", "Vasili", "Viktor", "Vladimir", "Vova", "Vyacheslav", "Yevgeny", "Yuri"];
        let mut request_raw = request_prepared;

        let mut rng = thread_rng();
        let v = rng.gen_range(0..49);
        let phone: u64 = rng.gen_range(1000000000..9999999999);

        let mut request_raw = request_raw.replace("%%random_name%%",names[v]);
        let mut request_raw = request_raw.replace("%%random_phone%%",phone.to_string().as_str());

        request_raw
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let url = format!("{}", args.url);
    let http_proxies_file = args.http_proxies_file.as_str();
    let socks4_proxies_file = args.socks4_proxies_file.as_str();
    let socks5_proxies_file = args.socks5_proxies_file.as_str();
    let headers_raw = args.headers.as_str();
    let request_raw = args.request.as_str();

    let req = RevRequest::new(headers_raw.to_string(), request_raw.to_string(), args.replace, args.req_type);

    let mut client_builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(args.connection_timeout));
    let verbose = args.verbose;

    if Path::new(http_proxies_file).exists() {
        if let Ok(lines) = read_lines(args.http_proxies_file) {
            for line in lines {
                if let Ok(proxy_url) = line {
                    let proxy = reqwest::Proxy::http(proxy_url).unwrap();
                    client_builder = client_builder.proxy(proxy);
                }
            }
        }
    }

    if Path::new(socks4_proxies_file).exists() {
        if let Ok(lines) = read_lines(socks4_proxies_file) {
            for line in lines {
                if let Ok(proxy_url) = line {
                    let proxy = reqwest::Proxy::http(proxy_url).unwrap();
                    client_builder = client_builder.proxy(proxy);
                }
            }
        }
    }

    if Path::new(socks5_proxies_file).exists() {
        if let Ok(lines) = read_lines(socks5_proxies_file) {
            for line in lines {
                if let Ok(proxy_url) = line {
                    let proxy = reqwest::Proxy::http(proxy_url).unwrap();
                    client_builder = client_builder.proxy(proxy);
                }
            }
        }
    }


    let client = client_builder.build().unwrap().clone();

    let fetches = futures::stream::iter(
        (0..)
            .into_iter()
            .map(|_| tokio::spawn(process(client.clone(),url.clone(), req.clone(), verbose))),
    )
        .for_each_concurrent(args.threads, |f| async move { f.await.unwrap() });

    fetches.await;
}

async fn process(client: Client, url: String, req: RevRequest, verbose: bool) {
    if verbose {
        println!("request body: {}",req.request.clone());
    }

    let request = match req.req_type.as_str() {
        "POST" => client.post(url).body(req.request.clone()),
        _ => client.get(url),
    };

    let res = request.header("content-type" ,"multipart/form-data").send().await;

    let status= match res {
        Ok(r) => r.text().await.unwrap(),
        _ => {String::from("error") }
    };

    if verbose {
        println!("status: {}", status);
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}