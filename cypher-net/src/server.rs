use tokio::net::UdpSocket;

pub async fn listen(addr: &str, port: u16) -> std::io::Result<()> {
    let socket = UdpSocket::bind(format!("{addr}:{port}")).await?;
    let mut buffer = [0; 2048];

    loop {
        let (len, addr) = socket.recv_from(&mut buffer).await?;
        println!("{:?} bytes received from {:?}", len, addr);
    }
}