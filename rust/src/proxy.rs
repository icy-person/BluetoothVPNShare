use std::{io, net::{IpAddr, Ipv4Addr, SocketAddr}};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream, UdpSocket}, sync::watch};

const V5:u8=5; const NO_AUTH:u8=0; const USERPASS:u8=2;
const CONNECT:u8=1; const UDP_ASSOCIATE:u8=3;
const IPV4:u8=1; const DOMAIN:u8=3; const IPV6:u8=4;

pub async fn run(port:u16, user:String, pass:String, mut stop:watch::Receiver<bool>)->io::Result<()>{
    let listener=TcpListener::bind(("0.0.0.0",port)).await?;
    loop{
        tokio::select!{
            _=stop.changed()=>{if *stop.borrow(){break;}},
            a=listener.accept()=>{let(s,_)=a?;let u=user.clone();let p=pass.clone();tokio::spawn(async move{let _=client(s,u,p).await;});}
        }
    }
    Ok(())
}

async fn client(mut s:TcpStream,user:String,pass:String)->io::Result<()>{
    let first=s.read_u8().await?;
    if first==V5{return socks5(s,user,pass).await;}
    http(s,first,user,pass).await
}

async fn socks5(mut s:TcpStream,user:String,pass:String)->io::Result<()>{
    let n=s.read_u8().await? as usize;let mut m=vec![0;n];s.read_exact(&mut m).await?;
    let auth=if user.is_empty(){NO_AUTH}else{USERPASS};
    if !m.contains(&auth){s.write_all(&[V5,0xff]).await?;return Ok(());} s.write_all(&[V5,auth]).await?;
    if auth==USERPASS{
        if s.read_u8().await?!=1{return Ok(());}let nu=s.read_u8().await? as usize;let mut ub=vec![0;nu];s.read_exact(&mut ub).await?;let np=s.read_u8().await? as usize;let mut pb=vec![0;np];s.read_exact(&mut pb).await?;
        let ok=String::from_utf8_lossy(&ub)==user && String::from_utf8_lossy(&pb)==pass;s.write_all(&[1,if ok{0}else{1}]).await?;if !ok{return Ok(());}
    }
    if s.read_u8().await?!=V5{return Ok(());}let cmd=s.read_u8().await?;let _=s.read_u8().await?;let atyp=s.read_u8().await?;
    let target=read_target(&mut s,atyp).await?;
    match cmd{CONNECT=>connect(s,target).await,UDP_ASSOCIATE=>udp(s).await,_=>reply(&mut s,7).await}
}

async fn read_target(s:&mut TcpStream,atyp:u8)->io::Result<String>{match atyp{
 IPV4=>{let mut b=[0;4];s.read_exact(&mut b).await?;let p=s.read_u16().await?;Ok(format!("{}.{}.{}.{}:{}",b[0],b[1],b[2],b[3],p))},
 DOMAIN=>{let n=s.read_u8().await? as usize;if n==0{return Err(io::Error::new(io::ErrorKind::InvalidData,"empty domain"));}let mut b=vec![0;n];s.read_exact(&mut b).await?;let p=s.read_u16().await?;Ok(format!("{}:{}",String::from_utf8_lossy(&b),p))},
 IPV6=>{let mut b=[0;16];s.read_exact(&mut b).await?;let p=s.read_u16().await?;Ok(format!("[{}]:{}",std::net::Ipv6Addr::from(b),p))},
 _=>Err(io::Error::new(io::ErrorKind::InvalidData,"bad atyp"))}}

async fn connect(mut c:TcpStream,target:String)->io::Result<()>{let r=match TcpStream::connect(target).await{Ok(x)=>x,Err(_)=>{reply(&mut c,1).await?;return Ok(());}};reply(&mut c,0).await?;let(mut cr,mut cw)=c.into_split();let(mut rr,mut rw)=r.into_split();tokio::try_join!(tokio::io::copy(&mut cr,&mut rw),tokio::io::copy(&mut rr,&mut cw))?;Ok(())}

async fn reply(s:&mut TcpStream,code:u8)->io::Result<()>{s.write_all(&[5,code,0,1,0,0,0,0,0,0]).await}

async fn udp(mut control:TcpStream)->io::Result<()>{
    let relay=UdpSocket::bind(("0.0.0.0",0)).await?;let a=relay.local_addr()?;let ip=match a.ip(){IpAddr::V4(x)=>x.octets(),_=>[0,0,0,0]};control.write_all(&[5,0,0,1,ip[0],ip[1],ip[2],ip[3],(a.port()>>8)as u8,a.port()as u8]).await?;
    let mut b=vec![0;65535];let mut cb=[0;1];loop{tokio::select!{n=control.read(&mut cb)=>{if n.unwrap_or(0)==0{break;}},r=relay.recv_from(&mut b)=>{let(n,peer)=r?;if let Some((dst,payload))=parse_udp(&b[..n]).await?{let out=UdpSocket::bind(("0.0.0.0",0)).await?;out.send_to(payload,dst).await?;let mut rb=vec![0;65535];if let Ok((rn,src))=out.recv_from(&mut rb).await{let mut q=vec![0,0,0,1];if let IpAddr::V4(v)=src.ip(){q.extend_from_slice(&v.octets());}else{continue;}q.extend_from_slice(&src.port().to_be_bytes());q.extend_from_slice(&rb[..rn]);let _=relay.send_to(&q,peer).await?;}}}}}Ok(())}

async fn parse_udp(b:&[u8])->io::Result<Option<(SocketAddr,&[u8])>>{if b.len()<4||b[0]!=0||b[1]!=0||b[2]!=0{return Ok(None)}let mut p=4;let d=match b[3]{1=>{if b.len()<p+6{return Ok(None)}let ip=Ipv4Addr::new(b[p],b[p+1],b[p+2],b[p+3]);p+=4;let port=u16::from_be_bytes([b[p],b[p+1]]);p+=2;SocketAddr::new(IpAddr::V4(ip),port)},3=>{if b.len()<p+1{return Ok(None)}let n=b[p]as usize;p+=1;if b.len()<p+n+2{return Ok(None)}let host=String::from_utf8_lossy(&b[p..p+n]).to_string();p+=n;let port=u16::from_be_bytes([b[p],b[p+1]]);p+=2;match tokio::net::lookup_host((host,port)).await?.next(){Some(x)=>x,None=>return Ok(None)}},4=>return Ok(None),_=>return Ok(None)};Ok(Some((d,&b[p..])))}

async fn http(mut s:TcpStream,first:u8,user:String,pass:String)->io::Result<()>{
    let mut data=vec![first];let mut buf=[0;4096];loop{let n=s.read(&mut buf).await?;if n==0{break;}data.extend_from_slice(&buf[..n]);if data.windows(4).any(|x|x==b"\r\n\r\n"){break;}if data.len()>65536{return Ok(());}}
    let text=String::from_utf8_lossy(&data);let mut lines=text.split("\r\n");let req=lines.next().unwrap_or("");let mut parts=req.split_whitespace();let method=parts.next().unwrap_or("");let uri=parts.next().unwrap_or("");
    if !user.is_empty(){let expected=format!("{}:{}",user,pass);let mut ok=false;for l in lines.clone(){if l.to_ascii_lowercase().starts_with("proxy-authorization:"){let v=l.split_whitespace().nth(2).unwrap_or("");let want=base64_simple(expected.as_bytes());ok=v==want;}}if !ok{s.write_all(b"HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic realm=proxy\r\n\r\n").await?;return Ok(());}}
    let target=if method.eq_ignore_ascii_case("CONNECT"){uri.to_string()}else{let u=uri.parse::<url::Url>().map_err(|_|io::Error::new(io::ErrorKind::InvalidData,"bad url"))?;format!("{}:{}",u.host_str().unwrap_or(""),u.port_or_known_default().unwrap_or(80))};
    let mut r=match TcpStream::connect(target).await{Ok(x)=>x,Err(_)=>{s.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await?;return Ok(());}};
    if method.eq_ignore_ascii_case("CONNECT"){s.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;}else{s.write_all(&data).await?;}
    let(mut sr,mut sw)=s.into_split();let(mut rr,mut rw)=r.into_split();tokio::try_join!(tokio::io::copy(&mut sr,&mut rw),tokio::io::copy(&mut rr,&mut sw))?;Ok(())
}

fn base64_simple(b:&[u8])->String{const T:&[u8]=b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";let mut o=String::new();let mut i=0;while i<b.len(){let a=b[i]as u32;let c=if i+1<b.len(){b[i+1]as u32}else{0};let d=if i+2<b.len(){b[i+2]as u32}else{0};o.push(T[(a>>2)as usize]as char);o.push(T[((a&3)<<4|c>>4)as usize]as char);if i+1<b.len(){o.push(T[((c&15)<<2|d>>6)as usize]as char)}else{o.push('=')}if i+2<b.len(){o.push(T[(d&63)as usize]as char)}else{o.push('=')}i+=3;}o}
